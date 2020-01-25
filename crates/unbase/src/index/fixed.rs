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
        LocalBoxFuture
    }
};

use std::{
    collections::HashMap,
    fmt,
};

use tracing::debug;

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

        self.recurse_set(context, 0, key, &mut self.root, subject.clone()).await
    }
    // Temporarily managing our own bubble-up
    // TODO: finish moving the management of this to context / context::subject_graph
    fn recurse_set <'a> (&'a self, context: &'a Context, tier: usize, key: u64, node: &'a mut Subject, subject: Subject) -> LocalBoxFuture<'a, Result<(),WriteError>>{
        async move {
            // TODO: refactor this in a way that is generalizable for strings and such
            // Could just assume we're dealing with whole bytes here, but I'd rather
            // allow for SUBJECT_MAX_RELATIONS <> 256. Values like 128, 512, 1024 may not be entirely ridiculous
            let exponent: u32 = (self.depth as u32 - 1) - tier as u32;
            let x = SUBJECT_MAX_RELATIONS.pow(exponent as u32);
            let y = ((key / (x as u64)) % SUBJECT_MAX_RELATIONS as u64) as RelationSlotId;

            //println!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                //println!("]]] end of the line");
                node.set_edge(context, y as RelationSlotId, &subject).await
            } else {
                match node.get_edge(context, y).await? {
                    Some(ref mut n) => {
                        self.recurse_set(context, tier + 1, key, n, subject).await
                    }
                    None => {
                        let mut values = HashMap::new();
                        values.insert("tier".to_string(), tier.to_string());

                        let mut new_node = Subject::new(context, SubjectType::IndexNode, values).await?;

                        node.set_edge(context, y, &new_node).await?;

                        self.recurse_set(context, tier + 1, key, &mut new_node, subject).await
                    }
                }
            }
        }.boxed_local()
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
        self.scan(&context, |r| {
            if let Some(v) = r.get_value(key) {
                Ok(v == value)
            }else{
                Ok(false)
            }
        }).await
    }
    pub async fn scan<F> ( &mut self, context: &Context, f: F ) -> Result<Option<SubjectHandle>, RetrieveError>
        where F: Fn( &SubjectHandle ) -> Result<bool,RetrieveError> {

        self.scan_recurse( context, &mut self.root, 0, &f ).await
    }

    fn scan_recurse <'a, F> ( &'a self, context: &'a Context, node: &'a mut Subject, tier: usize, f: &'a F ) -> LocalBoxFuture<'a, Result<Option<SubjectHandle>, RetrieveError>>
        where F: Fn( &SubjectHandle ) -> Result<bool,RetrieveError> {
        async move {

            // for _ in 0..tier+1 {
            //     print!("\t");
            // }

            if tier as u8 == self.depth - 1 {
                //println!("LAST Non-leaf node   {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..SUBJECT_MAX_RELATIONS {
                    if let Some(mrh) = node.get_edge_head(context, slot_id as RelationSlotId).await? {
                        let sh = context.get_subject_handle_with_head(mrh).await?;
                        if f(&sh)? {
                            return Ok(Some(sh))
                        }
                    }
                }
            } else {
                //println!("RECURSE {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..SUBJECT_MAX_RELATIONS {
                    if let Some(child) = node.get_edge(context, slot_id as RelationSlotId).await? {
                        if let Some(mrh) = self.scan_recurse(context, &mut child, tier + 1, f).await? {
                            return Ok(Some(mrh))
                        }
                    }
                }
            }

            Ok(None)

        }.boxed_local()
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