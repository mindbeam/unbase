use crate::context::ContextRef;
use crate::subject::*;
use crate::memorefhead::{MemoRefHead,RelationSlotId};
use crate::error::RetrieveError;
use std::collections::HashMap;
use futures::future::{FutureExt, LocalBoxFuture};
use std::sync::{Arc,Mutex};
use std::ops::Deref;
use std::fmt;

use tracing::debug;

#[derive(Clone)]
pub struct IndexFixed (Arc<Inner>);
impl Deref for IndexFixed {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &*self.0
    }
}

pub struct Inner {
    pub contextref: Mutex<ContextRef>,
    pub root: Mutex<Subject>,
    pub depth: u8
}

impl IndexFixed {
    pub async fn new (contextref: &ContextRef, depth: u8) -> IndexFixed {

        IndexFixed(Arc::new(Inner {
                contextref: Mutex::new(contextref.clone()),
                root: Mutex::new(Subject::new_with_contextref(contextref.clone(), HashMap::new(), true).await.unwrap()),
                depth: depth
            })
        )
    }
    pub fn new_from_memorefhead (contextref: ContextRef, depth: u8, memorefhead: MemoRefHead ) -> IndexFixed {
        IndexFixed(Arc::new(Inner {
                contextref: Mutex::new(contextref.clone()),
                root: Mutex::new(Subject::reconstitute(contextref, memorefhead)),
                depth: depth
            })
        )
    }
    pub fn get_root_id (&self) -> SubjectId {
        self.root.lock().unwrap().id
    }
    #[tracing::instrument]
    pub async fn insert <'a> (&self, key: u64, subject: &Subject) {
        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let node = {
            self.root.lock().unwrap().clone()
        };

        // TODO: optimize index node creation so we're not changing relationship as an edit
        // after the fact if we don't strictly have to. That said, this gives us a great excuse
        // to work on the consistency model, so I'm doing that first.

        self.recurse_set(0, key, &node, subject).await;
    }
    // Temporarily managing our own bubble-up
    // TODO: finish moving the management of this to context / context::subject_graph
    fn recurse_set(&self, tier: usize, key: u64, node: &Subject, subject: &Subject) -> LocalBoxFuture<()> {

        let me = (*self).clone();
        let node = (*node).clone();
        let subject = (*subject).clone();
        async move {
            // TODO: refactor this in a way that is generalizable for strings and such
            // Could just assume we're dealing with whole bytes here, but I'd rather
            // allow for SUBJECT_MAX_RELATIONS <> 256. Values like 128, 512, 1024 may not be entirely ridiculous
            let exponent: u32 = (me.depth as u32 - 1) - tier as u32;
            let x = SUBJECT_MAX_RELATIONS.pow(exponent as u32);
            let y = ((key / (x as u64)) % SUBJECT_MAX_RELATIONS as u64) as RelationSlotId;

            debug!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                // BUG: move this clause up
                debug!("]]] end of the line");
                node.set_relation(y as RelationSlotId, &subject).await;
            } else {
                match node.get_relation(y).await {
                    Ok(n) => {
                        me.recurse_set(tier + 1, key, &n, &subject).await;

                        //TEMPORARY - to be replaced by automatic context compaction
                        node.set_relation(y, &n).await;
                    }
                    Err(RetrieveError::NotFound) => {
                        let mut values = HashMap::new();
                        values.insert("tier".to_string(), tier.to_string());

                        let context = {
                            me.contextref.lock().unwrap().clone()
                        };
                        let new_node = Subject::new_with_contextref(context, values, true).await.unwrap();
                        node.set_relation(y, &new_node).await;

                        me.recurse_set(tier + 1, key, &new_node, &subject).await;

                        //TEMPORARY - to be replaced by automatic context compaction
                        node.set_relation(y, &new_node).await;
                    }
                    _ => {
                        panic!("unhandled error")
                    }
                }
            }
        }.boxed_local()
    }
    #[tracing::instrument]
    pub async fn get (&self, key: u64 ) -> Result<Subject, RetrieveError> {

        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = {
            self.root.lock().unwrap().clone()
        };
        let max = SUBJECT_MAX_RELATIONS as u64;

        //let mut n;
        for tier in 0..self.depth {
            let exponent = (self.depth - 1) - tier;
            let x = max.pow(exponent as u32);
            let y = ((key / (x as u64)) % max) as RelationSlotId;
            debug!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                debug!("]]] end of the line");
                return node.get_relation(y as RelationSlotId).await;

            }else{
                if let Ok(n) = node.get_relation(y).await {
                    node = n;
                }else{
                    return Err(RetrieveError::NotFound);
                }
            }

        };

        panic!("Sanity error");

    }
}

/*
    let idx_node = Subject::new_kv(&context_b, "dummy","value").unwrap();
    idx_node.set_relation( 0, rec_b1 );
    rec_b2.set_relation( 1, rec_b1 );
*/
impl fmt::Debug for IndexFixed {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("IndexFixed")
            .finish()
    }
}