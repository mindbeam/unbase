use std::fmt;
use std::collections::HashMap;
use slab::Slab;
use memo::Memo;
use memoref::MemoRef;
use error::RetrieveError;

use subject::*;
use std::sync::{Mutex,Arc,Weak};

pub struct ContextShared {
    head: Vec<Memo>,
    subjects: HashMap<SubjectId, WeakSubject>,
}

pub struct ContextInner {
    slab: Slab,
    shared: Mutex<ContextShared>
}
#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>
}

pub struct WeakContext {
    inner: Weak<ContextInner>
}


impl Context{
    pub fn new ( slab: &Slab ) -> Context {
        Context {
            inner: Arc::new(ContextInner {
                slab: slab.clone(),
                shared: Mutex::new(ContextShared {
                    head: vec![],
                    subjects: HashMap::new()
                })
            })
        }
    }
    pub fn get_slab (&self) -> &Slab {
        &self.inner.slab
    }
    pub fn subscribe_subject (&self, subject: &Subject) {
        {
            let mut shared = self.inner.shared.lock().unwrap();
            shared.subjects.insert( subject.id, subject.weak() );
        }
        self.inner.slab.subscribe_subject(subject.id, self);
    }
    pub fn unsubscribe_subject (&self, subject_id: SubjectId ){

        {

            let mut shared = self.inner.shared.lock().unwrap();
            shared.subjects.remove( &subject_id );
        }
        self.inner.slab.unsubscribe_subject(subject_id, self);
    }
    pub fn get_subject (&self, subject_id: SubjectId) -> Result<Subject, RetrieveError> {
        {
            let mut shared = self.inner.shared.lock().unwrap();
            // First - Check to see if I have the subject resident in this context
            if let Some(weaksub) = shared.subjects.get_mut(&subject_id) {
                if let Some(subject) = weaksub.upgrade() {
                    return Ok(subject);
                }else{
                    return Err(RetrieveError::NotFound);
                }
            }
        }

        // Else - Perform an index lookup on the primary subject index to construct the subject head
        match self.inner.slab.lookup_subject_head(subject_id) {
            Ok(head) => {
                println!("Reconstituting from slab {} subject {} head {:?}", self.inner.slab.id, subject_id, head );
                return Ok(Subject::reconstitute(self,subject_id,head));
            },
            Err(e) => {
                return Err(e)
            }
        }
    }

    pub fn update_subject_head (&self, subject_id: SubjectId, head: &[MemoRef]){
        if let Ok(mut subject) = self.get_subject(subject_id) {
            subject.update_head(head)
        }
    }

    pub fn cmp (&self, other: &Self) -> bool{
        // stable way:
        &*(self.inner) as *const _ != &*(other.inner) as *const _

        // unstable way:
        //Arc::ptr_eq(&self.inner,&other.inner)
    }
    pub fn weak (&self) -> WeakContext {
        WeakContext {
            inner: Arc::downgrade(&self.inner)
        }
    }
}

impl Drop for ContextShared {
    fn drop (&mut self) {
        println!("ContextShared.drop");
    }
}
impl fmt::Debug for ContextShared {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("ContextShared")
            .field("head", &self.head)
            // TODO: restore Debug for WeakSubject
            //.field("subjects", &self.subjects)
            .finish()
    }
}
impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let shared = self.inner.shared.lock().unwrap();

        fmt.debug_struct("Context")
            .field("inner", &shared)
            .finish()
    }
}

impl WeakContext {
    pub fn upgrade (&self) -> Option<Context> {
        match self.inner.upgrade() {
            Some(i) => Some( Context { inner: i } ),
            None    => None
        }
    }
    pub fn cmp (&self, other: &WeakContext) -> bool{
        if let Some(context) = self.upgrade() {
            if let Some(other) = other.upgrade(){
                // stable way:
                &*(context.inner) as *const _ != &*(other.inner) as *const _

                // unstable way:
                //Arc::ptr_eq(&context.inner,&other.inner)
            }else{
                false
            }
        }else {
            false
        }


    }
}
