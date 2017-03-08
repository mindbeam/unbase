use context::Context;
use subject::*;
use memorefhead::MemoRefHead;
use error::RetrieveError;
use std::collections::HashMap;


pub struct IndexFixed {
    context: Context,
    root: Subject,
    depth: u8
}

impl IndexFixed {
    pub fn new (context: &Context, depth: u8) -> IndexFixed {
        Self {
            context: context.clone(),
            root: Subject::new( context, HashMap::new(), true ).unwrap(),
            depth: depth
        }
    }
    pub fn new_from_memorefhead (context: &Context, depth: u8, memorefhead: MemoRefHead ) -> IndexFixed {
        Self {
            context: context.clone(),
            root: Subject::reconstitute( context, memorefhead ),
            depth: depth
        }
    }
    pub fn insert <'a> (&self, key: u64, subject: &Subject) {
        println!("IndexFixed.insert({}, {:?})", key, subject );
        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = SUBJECT_MAX_RELATIONS as u64;

        // TODO: optimize index node creation so we're not changing relationship as an edit
        // after the fact if we don't strictly have to. That said, this gives us a great excuse
        // to work on the consistency model, so I'm doing that first.

        for tier in 0..self.depth {

            // TODO: refactor this in a way that is generalizable for strings and such
            // Could just assume we're dealing with whole bytes here, but I'd rather
            // allow for SUBJECT_MAX_RELATIONS <> 256. Values like 128, 512, 1024 may not be entirely ridiculous
            let exponent = (self.depth - 1) - tier;
            let x = max.pow(exponent as u32);
            let y = ((key / (x as u64)) % max) as u8;

            println!("Tier {}, {}, {}", tier, x, y );

            match node.get_relation(y) {
                Ok(n) => {
                    node = n;
                }
                Err(e) => {
                    match e {
                        RetrieveError::NotFound => {
                            if exponent == 0 {
                                // BUG: move this clause up
                                println!("]]] end of the line");
                                node.set_relation(y as u8,subject.clone()); // TODO: should accept a borrow
                            }else{
                                let new_node = Subject::new( &self.context, HashMap::new(), true ).unwrap();
                                node.set_relation(y as u8,new_node.clone()); // TODO: should accept a borrow
                                node = new_node;
                            }
                        }
                        _ => {
                            panic!("unhandled error")
                        }
                    }
                }
            }
        }

    }
    pub fn get (&self, key: u64 ) -> Result<Subject, RetrieveError> {

        println!("IndexFixed.get({})", key );
        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = SUBJECT_MAX_RELATIONS as u64;

        //let mut n;
        for tier in 0..self.depth {
            let exponent = (self.depth - 1) - tier;
            let x = max.pow(exponent as u32);
            let y = ((key / (x as u64)) % max) as u8;
            println!("Tier {}, {}, {}", tier, x, y );

            if exponent == 0 {
                println!("]]] end of the line");
                return node.get_relation(y as u8);

            }else{
                if let Ok(n) = node.get_relation(y){
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
