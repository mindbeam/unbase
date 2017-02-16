use std::fmt;
use std::collections::HashMap;
use memo::*;
use memoref::MemoRef;
use context::Context;
use slab::*;
use std::sync::{Arc,Mutex,Weak};

pub type SubjectId     = u64;
pub type SubjectField  = String;

#[derive(Clone)]
pub struct Subject {
    pub id:  SubjectId,
    shared: Arc<Mutex<SubjectShared>>
}

pub struct WeakSubject {
    pub id:  SubjectId,
    shared: Weak<Mutex<SubjectShared>>
}

pub struct SubjectShared {
    id:      SubjectId,
    head:    Vec<MemoRef>,
    context: Context,
}

impl Subject {
    pub fn new ( context: &Context, vals: HashMap<String, String> ) -> Result<Subject,String> {

        let slab : &Slab = context.get_slab();
        let subject_id = slab.generate_subject_id();

        let shared = SubjectShared{
            id: subject_id,
            head: Vec::new(),
            context: context.clone()
        };

        let subject = Subject {
            id:      subject_id,
            shared: Arc::new(Mutex::new(shared))
        };

        context.subscribe_subject( &subject );

        slab.put_memos(MemoOrigin::Local, vec![
            Memo::new_basic_noparent( slab.gen_memo_id(), subject_id, MemoBody::Edit(vals) )
        ]);

        Ok(subject)
    }
    pub fn reconstitute (context: &Context, subject_id: SubjectId, head: Vec<MemoRef>) -> Subject {
        let shared = SubjectShared{
            id: subject_id,
            head: head,
            context: context.clone()
        };

        let subject = Subject {
            id:      subject_id,
            shared: Arc::new(Mutex::new(shared))
        };

        context.subscribe_subject( &subject );

        subject
    }
    pub fn new_kv ( context: &Context, key: &str, value: &str) -> Result<Subject,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals )
    }
    pub fn set_kv (&self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let slab;
        let head;
        {
            let shared = self.shared.lock().unwrap();
            slab = shared.context.get_slab().clone();
            head = shared.head.clone();
        }
        //println!("Subject({}).set_kv({},{}) -> Starting head.len {}",self.id,key,value,self.shared.lock().unwrap().head.len() );

        slab.put_memos(MemoOrigin::Local, vec![
            Memo::new_basic( slab.gen_memo_id(), self.id, head, MemoBody::Edit(vals) )
        ]);
        //println!("Subject({}).set_kv({},{}) -> Ending head.len {}",self.id,key,value,self.shared.lock().unwrap().head.len() );

        //println!("Subject({}).set_kv({},{}) -> {:?}",self.id,key,value,self.shared.lock().unwrap().head );
        true
    }
    pub fn get_value ( &self, key: &str ) -> Option<String> {
        println!("Subject({}).get_value({})",self.id,key);
        for memo in self.memo_iter() {
            println!("\t Memo {:?}", memo );
            let values = memo.get_values();
            if let Some(v) = values.get(key) {
                return Some(v.clone());
            }
        }
        None
    }
    pub fn set_relation (&self, key: u8, relation: Self) {
        let mut memoref_map = HashMap::new();
        memoref_map.insert(key, (relation.id, relation.get_head().clone()) );

        let slab;
        let head;
        {
            let shared = self.shared.lock().unwrap();
            slab = shared.context.get_slab().clone();
            head = shared.head.clone();
        }


        slab.put_memos(MemoOrigin::Local, vec![
            Memo::new(
                slab.gen_memo_id(),
                self.id,
                head,
                MemoBody::Relation(memoref_map)
            )
        ]);
    }
    pub fn get_relation ( &self, key: u8 ) -> Option<Subject> {
        println!("Subject({}).get_relation({})",self.id,key);

        let shared = self.shared.lock().unwrap();
        let context = &shared.context;

        for memo in self.memo_iter() {
            println!("\t Memo {:?}", memo );
            let relations : HashMap<u8, (SubjectId, Vec<MemoRef>)> = memo.get_relations();

            if let Some(r) = relations.get(&key) {
                if let Ok(relation) = context.get_subject_with_head(r.0,r.1.clone()) {
                    return Some(relation)
                }else{
                    return None;
                    //return Err("subject retrieval error")
                }
            }
        }
        None
    }
    pub fn update_head (&mut self, head: &[MemoRef]){
        let mut shared = self.shared.lock().unwrap();

        // TODO: prune the head to remove any memos which are referenced by these memos
        shared.head = head.to_vec();
    }
    pub fn get_head (&self) -> Vec<MemoRef> {
        let shared = self.shared.lock().unwrap();
        shared.head.clone()
    }
    fn memo_iter (&self) -> SubjectMemoIter {
        let shared = self.shared.lock().unwrap();

        SubjectMemoIter::from_head(&shared.head, shared.context.get_slab() )
    }
    pub fn weak (&self) -> WeakSubject {
        WeakSubject {
            id: self.id,
            shared: Arc::downgrade(&self.shared)
        }
    }
}

pub struct SubjectMemoIter {
    queue: Vec<MemoRef>,
    slab:  Slab
}

/*
  Plausible Memo Structure:
          /- E -> C -\
     G ->              -> B -> A
head ^    \- F -> D -/
     Desired iterator sequence: G, E, C, F, D, B, A ( Why? )
     Consider:                  [G], [E,C], [F,D], [B], [A]
     Arguably this should not be an iterator at all, but rather a recursive function
     Going with the iterator for now in the interest of simplicity
*/
impl SubjectMemoIter {
    pub fn from_head ( head: &Vec<MemoRef>, slab: &Slab) -> Self {
        SubjectMemoIter {
            queue: head.clone(),
            slab: slab.clone()
        }
    }
}
impl Iterator for SubjectMemoIter {
    type Item = Memo;

    fn next (&mut self) -> Option<Memo> {
        // iterate over head memos
        // Unnecessarly complex because we're not always dealing with MemoRefs
        // Arguably heads should be stored as Vec<MemoRef> instead of Vec<Memo>
        if self.queue.len() > 0 {
            let mut memoref = self.queue.remove(0);
            // this is wrong - Will result in G, E, F, C, D, B, A

            if let Ok(memo) = memoref.get_memo( &self.slab ){
                self.queue.append(&mut memo.get_parent_refs());
                return Some(memo)
            }
            //TODO: memoref.get_memo needs to be able to fail
        }

        return None;
    }
}

impl Drop for SubjectShared {
    fn drop (&mut self) {
        println!("Subject({}).drop", &self.id);
        self.context.unsubscribe_subject(self.id);
    }
}
impl fmt::Debug for SubjectShared {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}

impl fmt::Debug for Subject {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.shared.lock().unwrap();

        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &shared.head)
            .finish()
    }
}

impl WeakSubject {
    pub fn upgrade (&self) -> Option<Subject> {
        match self.shared.upgrade() {
            Some(s) => Some( Subject { id: self.id, shared: s } ),
            None    => None
        }
    }
}
