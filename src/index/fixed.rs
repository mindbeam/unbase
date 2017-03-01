use context::Context;
use subject::*;
use super::Index;
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
            root: Subject::new( context, HashMap::new() ).unwrap(),
            depth: depth
        }
    }
    pub fn insert <'a> (&self, key: u64, subject: &Subject) {

        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = SUBJECT_MAX_RELATION as u64 + 1;

        //let mut n;
        for exp in (0..self.depth).rev() {
            let x = max.pow(exp as u32);
            let y = ((key / (x as u64)) % max) as u8;
            println!("Tier {}, {}, {}", exp, x, y );

            if let Some(n) = node.get_relation(y){
                node = n;
            }else{
                if exp == 0{
                    println!("]]] end of the line");
                    node.set_relation(y as u8,subject.clone()); // TODO: should accept a borrow
                }else{
                    let new_node = Subject::new( &self.context, HashMap::new() ).unwrap();
                    node.set_relation(y as u8,new_node.clone()); // TODO: should accept a borrow
                    node = new_node;
                }
            }
        }

    }
    pub fn get (&self, key: u64 ) -> Option<Subject> {

        //TODO: this is dumb, figure out how to borrow here
        //      and replace with borrows for nested subjects
        let mut node = self.root.clone();
        let max = SUBJECT_MAX_RELATION as u64 + 1;

        //let mut n;
        for exp in (0..self.depth).rev() {
            let x = max.pow(exp as u32);
            let y = ((key / (x as u64)) % max) as u8;
            println!("Tier {}, {}, {}", exp, x, y );

            if let Some(n) = node.get_relation(y){
                node = n;
            }else{
                return None;
            }

            if exp == 0 {
                return node.get_relation(y as u8);
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
