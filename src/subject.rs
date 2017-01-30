use std::fmt;
use std::collections::HashMap;
use memo::Memo;
use memoref::MemoRef;
use context::Context;
use slab::Slab;
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
    pub fn new ( context: Context, vals: HashMap<String, String> ) -> Result<Subject,String> {

        let slab = context.get_slab();
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
        Memo::create( &slab, subject_id, vec![], vals );

        Ok(subject)
    }
    pub fn new_kv ( context: Context, key: &str, value: &str) -> Result<Subject,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals )
    }
    pub fn set_kv (&mut self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let shared = self.shared.lock().unwrap();
        let slab = shared.context.get_slab();
        let head = shared.head.clone();

        Memo::create( &slab, self.id, head, vals );

        true
    }
    pub fn get_value ( &self, key: &str ) -> Option<String> {
        //self.context.get_subject_value(self.id, key)

        //let mut memos = self.context.get_subject_head(self.id);
        for memo in self.memo_iter() {
            let values = memo.get_values();
            if let Some(v) = values.get(key) {
                return Some(v.clone());
            }
        }
        None
    }
    pub fn append_memorefs (&mut self, memorefs: &[MemoRef]){

        let mut shared = self.shared.lock().unwrap();

        // TODO: prune the head to remove any memos which are referenced by these memos
        shared.head.append(&mut memorefs.to_vec());
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
        }

        return None;
    }
}

impl Drop for SubjectShared {
    fn drop (&mut self) {
        println!("Drop {:?}", &self);
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
/*
Record.prototype.set = function(vals){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */

    var memo = new memo_cls.create( this.slab,this.id, this.getHeadMemoIDs(), this.context.getPresentContext(), vals );
    this.context.addMemos([memo]);
    // TODO - return promise which is fulfilled on transaction commit
}

var memosort = function(a,b){
    // TODO - implement sorting by beacon-offset-millisecond LWW or node id as required to achieve desired determinism

    if ( a.id < b.id )
        return -1;
    if ( a.id > b.id )
        return 1;

    return 0;
}


Record.prototype.get = function(field){
    // TODO: implement promises for get
    return this.getFreshOrNull(field);
}

Record.prototype.getHeadMemoIDs = function(){
    return this.slab.getHeadMemoIDsForRecord( this.id );
}


Record.prototype.getMemoIDs = function(){
    return Object.keys(this.memos_by_id);
};
*/
