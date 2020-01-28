use crate::{
    context::Context,
    error::{
        RetrieveError,
        WriteError,
    },
    memorefhead::MemoRefHead,
    slab::{
        RelationSlotId,
        SubjectType,
        MAX_SLOTS,
        MemoBody,
        RelationSet,
        EdgeSet
    },
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

pub struct IndexFixed {
    root: MemoRefHead,
    depth: u8
}

impl IndexFixed {
    pub fn new (context: &Context, depth: u8) -> IndexFixed {
        let mut debug_info = HashMap::new();
        debug_info.insert("tier".to_string(),  "root".to_string());

        Self {
            root: MemoRefHead::new_index(&context.slab, debug_info),
            depth: depth
        }
    }
    pub fn new_from_head(context: &Context, depth: u8, memorefhead: MemoRefHead ) -> IndexFixed {
        Self {
            root: memorefhead,
            depth: depth
        }
    }
    pub (crate) async fn insert <'a> (&mut self, context: &Context, key: u64, target: MemoRefHead) -> Result<(),WriteError> {
        debug!("IndexFixed.insert({}, {:?})", key, target);

        // TODO: optimize index node creation so we're not changing relationship as an edit
        // after the fact if we don't strictly have to. That said, this gives us a great excuse
        // to work on the consistency model, so I'm doing that first.

        let mut tier = 0;
        let mut node = self.root.clone();

        loop{
            // TODO: refactor this in a way that is generalizable for strings and such
            // Could just assume we're dealing with whole bytes here, but I'd rather
            // allow for SUBJECT_MAX_RELATIONS <> 256. Values like 128, 512, 1024 may not be entirely ridiculous
            let exponent: u32 = (self.depth as u32 - 1) - tier as u32;
            let x = MAX_SLOTS.pow(exponent as u32);
            let y = ((key / (x as u64)) % MAX_SLOTS as u64) as RelationSlotId;

            //println!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                // Leaf node
                //println!("]]] end of the line");

                // TODO- this MIGHT not be necessary, because context.apply_head might be doing the same thing.
                context.mut_update_head_for_consistency( &mut node ).await?;

                node.set_edge(&context.slab, y as RelationSlotId, target);

                // Apply the updated head to the context
                context.apply_head( &node ).await?;

                return Ok(());
            } else {
                // TODO- this MIGHT not be necessary, because context.apply_head might be doing the same thing.
                context.mut_update_head_for_consistency( &mut node ).await?;

                match node.get_edge(&context.slab, y).await? {
                    Some(n) => {
                        node = n;
                        tier += 1;
                    }
                    None => {
                        let mut debug_info = HashMap::new();
                        debug_info.insert("tier".to_string(), tier.to_string());

                        let next_node = MemoRefHead::new_index( &context.slab, debug_info );

                        // apply the new_node head to the context
                        // TODO POSTMERGE - determine if we can skip this apply_head because we're about to do it for the updated parent node
                        context.apply_head( &next_node ).await?;

                        node.set_edge(&context.slab, y, next_node.clone());
                        // Apply the updated head to the context
                        context.apply_head( &node ).await?;

                        node = next_node;
                        tier += 1;
                    }
                }
            }
        }
    }
    /// Convenience method for the test suite
    #[doc(hidden)]
    #[cfg(test)]
    pub (crate) async fn test_get_subject_handle(&self, context: &Context, key: u64 ) -> Result<Option<crate::subjecthandle::SubjectHandle>,RetrieveError> {
        use crate::subjecthandle::SubjectHandle;
        match self.get(context,key).await? {
            Some(mut head) => {

                let subject = context.get_subject_from_head( head ).await?;

                Ok(Some(subject))
            },
            None => Ok(None)
        }
    }
    #[tracing::instrument]
    pub async fn get ( &self, context: &Context, key: u64 ) -> Result<Option<MemoRefHead>, RetrieveError> {

        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = MAX_SLOTS as u64;

        //let mut n;
        for tier in 0..self.depth {
            let exponent = (self.depth - 1) - tier;
            let x = max.pow(exponent as u32);
            let y = ((key / (x as u64)) % max) as RelationSlotId;
            debug!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                // Leaf node
                debug!("]]] end of the line");

                context.mut_update_head_for_consistency( &mut node ).await?;

                return node.get_edge( &context.slab, y as RelationSlotId).await;

            }else{
                // branch

                context.mut_update_head_for_consistency( &mut node ).await?;

                match node.get_edge( &context.slab, y).await? {
                    Some(n) => node = n,
                    None    => return Ok(None),
                }
            }

        };

        panic!("Sanity error");

    }
    pub async fn scan_kv( &mut self, context: &Context, key: &str, value: &str ) -> Result<Option<MemoRefHead>, RetrieveError> {
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
    pub async fn scan<F, Fut> ( &mut self, context: &Context, f: F ) -> Result<Option<MemoRefHead>, RetrieveError>
        where
            F: Fn( &mut MemoRefHead ) -> Fut,
            Fut: Future<Output=Result<bool,RetrieveError>>
    {

        let mut stack : Vec<(MemoRefHead,usize)> = vec![(self.root.clone(),0)];

        while let Some((mut node,tier)) = stack.pop() {
            if tier as u8 == self.depth - 1 {

                // TODO NEXT / WIP: finish converting this to stack based recursion.
                // Seems the compiler doesn't like something here. Most likely has to do with

                //println!("LAST Non-leaf node   {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..MAX_SLOTS {

                    context.mut_update_head_for_consistency( &mut node ).await?;
                    if let Some(mut head) = node.get_edge(&context.slab, slot_id as RelationSlotId).await? {

                        if f(&mut head).await? {
                            return Ok(Some(head))
                        }
                    }
                }
            } else {
                //println!("RECURSE {}, {}, {}", node.id, tier, self.depth );
                for slot_id in 0..MAX_SLOTS {

                    context.mut_update_head_for_consistency( &mut node ).await?;
                    if let Some(child) = node.get_edge(&context.slab, slot_id as RelationSlotId).await? {
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

        let mut index = IndexFixed::new(&context_a, 5);

        // First lets do a single index test
        let i = 12345;
        let record = SubjectHandle::new_kv(&context_a, "record number", &format!("{}",i)).unwrap();
        index.insert(&context_a, i,record.head.clone()).unwrap();

        assert_eq!(index.test_get_subject_handle(&context_a, 12345).unwrap().unwrap().get_value("record number").unwrap(), "12345");

        //Ok, now lets torture it a little
        for i in 0..500 {
            let record = SubjectHandle::new_kv(&context_a, "record number", &format!("{}",i)).unwrap();
            index.insert(&context_a, i, record.head.clone()).unwrap();
        }

        for i in 0..500 {
            assert_eq!(index.test_get_subject_handle(&context_a, i).unwrap().unwrap().get_value("record number").unwrap(), i.to_string() );
        }

        let maybe_rec = index.scan_kv(&context_a, "record number","12345").unwrap();
        assert!( maybe_rec.is_some(), "Index scan for record 12345" );
        assert_eq!( maybe_rec.unwrap().get_value("record number").unwrap(), "12345", "Is correct record");

        let maybe_rec = index.scan_kv(&context_a, "record number","275").unwrap();
        assert!( maybe_rec.is_some(), "Index scan for record 275" );
        assert_eq!( maybe_rec.unwrap().get_value("record number").unwrap(), "275", "Is correct record");
    }
}