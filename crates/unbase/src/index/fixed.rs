use crate::{
    context::Context,
    error::{
        RetrieveError,
        WriteError,
    },
    memorefhead::MemoRefHead,
    slab::{
        RelationSlotId
    },
    subject::{
        Subject,
        SubjectType,
        SUBJECT_MAX_RELATIONS,
    },
    SubjectHandle,
};

use futures::{
    future::{
        FutureExt,
        TryFutureExt,
        LocalBoxFuture
    },
    Future,
};

use std::{
    collections::HashMap,
    cell::RefCell,
    fmt,
};

use tracing::debug;
use crate::slab::{MemoBody, RelationSet, EdgeSet};

pub struct IndexFixed {
    root: Subject,
    depth: u8
}

impl IndexFixed {
    pub async fn new (context: &Context, depth: u8) -> Result<IndexFixed,WriteError> {
        Ok(Self {
            root: Subject::new( context, SubjectType::IndexNode, HashMap::new() ).await?,
            depth: depth
        })
    }
    pub fn new_from_memorefhead (context: &Context, depth: u8, memorefhead: MemoRefHead ) -> IndexFixed {
        Self {
            root: Subject::reconstitute( context, memorefhead ).unwrap(),
            depth: depth
        }
    }
    pub async fn insert_subject(&mut self, key: u64, subjecthandle: &SubjectHandle) -> Result<(),WriteError> {
        self.insert(&subjecthandle.context, key, &subjecthandle.subject).await
    }
    pub (crate) async fn insert <'a> (&mut self, context: &Context, key: u64, subject: &Subject) -> Result<(),WriteError> {
        debug!("IndexFixed.insert({}, {:?})", key, subject );

        // TODO: optimize index node creation so we're not changing relationship as an edit
        // after the fact if we don't strictly have to. That said, this gives us a great excuse
        // to work on the consistency model, so I'm doing that first.

        let mut tier = 0;
//        let mut new_node : RefCell<Option<Subject>> = RefCell::new(None);
        let mut node = self.root.clone();

        loop{
            // TODO: refactor this in a way that is generalizable for strings and such
            // Could just assume we're dealing with whole bytes here, but I'd rather
            // allow for SUBJECT_MAX_RELATIONS <> 256. Values like 128, 512, 1024 may not be entirely ridiculous
            let exponent: u32 = (self.depth as u32 - 1) - tier as u32;
            let x = SUBJECT_MAX_RELATIONS.pow(exponent as u32);
            let y = ((key / (x as u64)) % SUBJECT_MAX_RELATIONS as u64) as RelationSlotId;

            //println!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                //println!("]]] end of the line");
                node.set_edge(context, y as RelationSlotId, &subject);
                // Apply the updated head to the context
                context.apply_head( &node.head ).await?;

                return Ok(());
            } else {
                match node.get_edge(context, y).await? {
                    Some(n) => {
                        node = n;
                        tier += 1;
                    }
                    None => {
                        let mut values = HashMap::new();
                        values.insert("tier".to_string(), tier.to_string());

                        // Manually implementing the relevant parts of Subject::new to avoid recursive async functions / boxed futures
                        // we can get away with this because Subject::new only inserts other SubjectTypes into the root index, not ::IndexNode
                        let id = context.slab.generate_subject_id(SubjectType::IndexNode);
                        let head = context.slab.new_memo(
                            Some(id),
                            MemoRefHead::Null,
                            MemoBody::FullyMaterialized {v: values, r: RelationSet::empty(), e: EdgeSet::empty(), t: SubjectType::IndexNode }
                        ).to_head();

                        // apply the new_node head to the context
                        // TODO POSTMERGE - determine if we can skip this apply_head because we're about to do it for the updated parent node
                        context.apply_head( &head ).await?;
                        let next_node = Subject{ id, head };

                        node.set_edge(context, y, &next_node);
                        // Apply the updated head to the context
                        context.apply_head( &node.head ).await?;

                        node = next_node;
                        tier += 1;
                    }
                }
            }
        }
    }
    pub fn get_root_subject_handle(&self, context: &Context) -> Result<SubjectHandle,RetrieveError> {
        Ok(SubjectHandle{
            id: self.root.id,
            subject: self.root.clone(),
            context: context.clone()
        })
    }
    pub async fn get_subject_handle(&self, context: &Context, key: u64 ) -> Result<Option<SubjectHandle>,RetrieveError> {
        match self.get(context,key).await? {
            Some(subject) => {
                Ok(Some(SubjectHandle{
                    id: subject.id,
                    subject: subject,
                    context: context.clone()
                }))
            },
            None => Ok(None)
        }
    }
    #[tracing::instrument]
    pub async fn get ( &self, context: &Context, key: u64 ) -> Result<Option<Subject>, RetrieveError> {
        match self.get_head( context, key ).await? {
            Some(mrh) => Ok(Some( context.get_subject_with_head( mrh ).await? )),
            None      => Ok(None)
        }
    }
    #[tracing::instrument]
    pub async fn get_head ( &self, context: &Context, key: u64 ) -> Result<Option<MemoRefHead>, RetrieveError> {

        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = SUBJECT_MAX_RELATIONS as u64;

        //let mut n;
        for tier in 0..self.depth {
            let exponent = (self.depth - 1) - tier;
            let x = max.pow(exponent as u32);
            let y = ((key / (x as u64)) % max) as RelationSlotId;
            debug!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                debug!("]]] end of the line");
                return node.get_edge_head( context, y as RelationSlotId).await;

            }else{
                match node.get_edge( context, y).await? {
                    Some(n) => node = n,
                    None    => return Ok(None),
                }
            }

        };

        panic!("Sanity error");

    }
    pub async fn scan_kv( &mut self, context: &Context, key: &str, value: &str ) -> Result<Option<SubjectHandle>, RetrieveError> {
        // TODO - make scan_concurrent or something like that.
        // The problem with concurrent scanning is: how do we want to manage output ordering?
        // Presumably scan should be generic over output Vec<T>
        // That way, closure execution won't be (deterministically/lexicographically) ordered, but scan() -> Vec<T> will be

        // TODO MERGE - uncomment ( crap, I think we probably do need async closures )
        self.scan(&context, move |r| {
//            async {
//                if let Some(v) = r.get_value(key).await? {
//                    Ok(v == value)
//                } else {
//                    Ok(false)
//                }
//            }
            futures::future::ready(Ok(false) )
//            unimplemented!()
        }).await;

        Ok(None)
    }
    pub async fn scan<F, Fut> ( &mut self, context: &Context, f: F ) -> Result<Option<SubjectHandle>, RetrieveError>
        where
            F: Fn( &mut SubjectHandle ) -> Fut,
            Fut: Future<Output=Result<bool,RetrieveError>>
    {

        let mut stack : Vec<(Subject,usize)> = vec![(self.root.clone(),0)];

        while let Some((mut node,tier)) = stack.pop() {
            if tier as u8 == self.depth - 1 {

                // TODO NEXT / WIP: finish converting this to stack based recursion.
                // Seems the compiler doesn't like something here. Most likely has to do with

                //println!("LAST Non-leaf node   {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..SUBJECT_MAX_RELATIONS {
                    if let Some(mrh) = node.get_edge_head(context, slot_id as RelationSlotId).await? {
                        let mut sh = context.get_subject_handle_with_head(mrh).await?;
                        if f(&mut sh).await? {
                            return Ok(Some(sh))
                        }
                    }
                }
            } else {
                //println!("RECURSE {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..SUBJECT_MAX_RELATIONS {
                    if let Some(child) = node.get_edge(context, slot_id as RelationSlotId).await? {
                        stack.push( (child, tier + 1 ))
                    }
                }
            }
        }


        Ok(None)
    }
}

impl fmt::Debug for IndexFixed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("IndexFixed")
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::{Network, Slab, SubjectHandle};
    use crate::index::IndexFixed;

    #[test]
    fn index_construction() {

        let net = Network::create_new_system();

        let context_a = Slab::new(&net).create_context();

        let index = IndexFixed::new(&context_a, 5).unwrap();

        // First lets do a single index test
        let i = 12345;
        let record = SubjectHandle::new_kv(&context_a, "record number", &format!("{}",i)).unwrap();
        index.insert_subject(i, &record).unwrap();

        assert_eq!( index.get_subject_handle(&context_a,12345).unwrap().unwrap().get_value("record number").unwrap(), "12345");

        //Ok, now lets torture it a little
        for i in 0..500 {
            let record = SubjectHandle::new_kv(&context_a, "record number", &format!("{}",i)).unwrap();
            index.insert_subject(i, &record).unwrap();
        }

        for i in 0..500 {
            assert_eq!( index.get_subject_handle(&context_a,i).unwrap().unwrap().get_value("record number").unwrap(), i.to_string() );
        }

        let maybe_rec = index.scan_kv(&context_a, "record number","12345").unwrap();
        assert!( maybe_rec.is_some(), "Index scan for record 12345" );
        assert_eq!( maybe_rec.unwrap().get_value("record number").unwrap(), "12345", "Is correct record");

        let maybe_rec = index.scan_kv(&context_a, "record number","275").unwrap();
        assert!( maybe_rec.is_some(), "Index scan for record 275" );
        assert_eq!( maybe_rec.unwrap().get_value("record number").unwrap(), "275", "Is correct record");
    }
}