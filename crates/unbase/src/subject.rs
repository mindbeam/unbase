//TODO MERGE topic/topo-compaction3

use std::{
    collections::HashMap,
    fmt
};
use tracing::debug;
use futures::{
    StreamExt,
    channel::mpsc,
};
use crate::{
    context::Context,
    error::{
        RetrieveError,
        WriteError,
    },
    memorefhead::{
        MemoRefHead,
    },
    slab::{
        EdgeSet,
        MemoBody,
        MemoId,
        RelationSet,
        RelationSlotId,
        SlabHandle,
    }
};

pub const SUBJECT_MAX_RELATIONS : usize = 256;
#[derive(Copy,Clone,Eq,PartialEq,Ord,PartialOrd,Hash,Debug,Serialize,Deserialize)]
pub enum SubjectType {
    IndexNode,
    Record,
}
#[derive(Copy,Clone,Eq,PartialEq,Ord,PartialOrd,Hash,Debug,Serialize,Deserialize)]
pub struct SubjectId {
    pub id:    u64,
    pub stype: SubjectType,
}
impl <'a> core::cmp::PartialEq<&'a str> for SubjectId {
    fn eq (&self, other: &&'a str) -> bool {
        self.concise_string() == *other
    }
}

impl SubjectId {
    pub fn test(test_id: u64) -> Self{
        SubjectId{
            id:    test_id,
            stype: SubjectType::Record
        }
    }
    pub fn index_test(test_id: u64) -> Self{
        SubjectId{
            id:    test_id,
            stype: SubjectType::IndexNode
        }
    }
    pub fn concise_string (&self) -> String {
        use self::SubjectType::*;
        match self.stype {
            IndexNode => format!("I{}", self.id),
            Record    => format!("R{}", self.id)
        }
    }
}

impl fmt::Display for SubjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}-{}", self.stype, self.id)
    }
}

pub(crate) struct Subject {
    pub id:     SubjectId,
    pub (crate) head: MemoRefHead
}

impl Subject {
    // TODO POSTMERGE - consider merging Subject and MemoRefHead ( SubjectHandle could potentially be then renamed to Subject? )
    pub async fn new (context: &Context, stype: SubjectType, vals: HashMap<String,String> ) -> Result<Self,WriteError> {
        if let SubjectType::IndexNode = stype {
            panic!("now allowed to use Subject::new for IndexNode");
            // Perhaps this means that IndexNode should be its own thing?
        }

        let slab: &SlabHandle = &context.slab;
        let id = slab.generate_subject_id(stype);
        debug!("Subject({}).new()",id);

        let head = slab.new_memo(
            Some(id),
            MemoRefHead::Null,
            MemoBody::FullyMaterialized {v: vals, r: RelationSet::empty(), e: EdgeSet::empty(), t: stype.clone() }
        ).to_head();

        let mut subject = Subject{ id, head };

        //slab.subscribe_subject( &subject );

        subject.update_referents( context ).await?;

        Ok(subject)
    }
    /// Notify whomever needs to know that a new subject has been created
    async fn update_referents (&mut self, context: &Context) -> Result<(),WriteError> {
        match self.id.stype {
            SubjectType::IndexNode => {
//                context.apply_head( &self.head ).await?;
                panic!("not allowed to use this for index nodes")
            },
            SubjectType::Record    => {
                // TODO: Consider whether this should accept head instead of subject
                context.insert_into_root_index( self.id, &self ).await?;
            }
        }

        Ok(())
    }
    pub fn reconstitute (_context: &Context, head: MemoRefHead) -> Result<Subject,RetrieveError> {

        // TODO: consolidate this with new_from_memorefhead
        // TODO: do we need to be calling update_referents from here?
        //println!("Subject.reconstitute({:?})", head);
        // Arguably we shouldn't ever be reconstituting a subject

        if let Some(id) = head.subject_id(){
            let subject = Subject{ id, head };

            // TODO3 - Should a resident subject be proactively updated? Or only when it's being observed?
            //context.slab.subscribe_subject( &subject );

            Ok(subject)

        }else{
            Err(RetrieveError::InvalidMemoRefHead)
        }
    }
    pub async fn get_value ( &mut self, context: &Context, key: &str ) -> Result<Option<String>, RetrieveError> {
        //println!("# Subject({}).get_value({})",self.id,key);

        // TODO3: Consider updating index node ingress to mark relevant subjects as potentially dirty
        //        Use the lack of potential dirtyness to skip index traversal inside get_relevant_subject_head
        let chead = context.get_relevant_subject_head(self.id).await?;
        //println!("\t\tGOT: {:?}", chead.memo_ids() );

        self.head.apply_mut( &chead, &context.slab ).await?;
        self.head.project_value(&context.slab, key).await
    }
    pub async fn get_relation ( &mut self, context: &Context, key: RelationSlotId ) -> Result<Option<Subject>, RetrieveError> {
        //println!("# Subject({}).get_relation({})",self.id,key);
        self.head.apply_mut( &context.get_resident_subject_head(self.id), &context.slab ).await?;

        match self.head.project_relation(&context.slab, key).await? {
            Some(subject_id) => context.get_subject(subject_id).await,
            None             => Ok(None),
        }
    }
    pub async fn get_edge ( &mut self, context: &Context, key: RelationSlotId ) -> Result<Option<Subject>, RetrieveError> {
        match self.get_edge_head(context,key).await? {
            Some(head) => {
                Ok( Some( context.get_subject_with_head(head).await? ) )
            },
            None => {
                Ok(None)
            }
        }
    }
    pub async fn get_edge_head ( &mut self, context: &Context, key: RelationSlotId ) -> Result<Option<MemoRefHead>, RetrieveError> {
        //println!("# Subject({}).get_relation({})",self.id,key);
        self.head.apply_mut( &context.get_resident_subject_head(self.id), &context.slab ).await?;
        self.head.project_edge(&context.slab, key).await
    }

    pub async fn set_value (&mut self, context: &Context, key: &str, value: &str) -> Result<bool,WriteError> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        // TODO - do this in a single swap? May require unsafe
        let mut head = MemoRefHead::Null;
        std::mem::swap(&mut head,&mut self.head);

        let mut new_head = context.slab.new_memo(
            Some(self.id),
            head,
            MemoBody::Edit(vals)
        ).to_head();

        std::mem::swap(&mut self.head, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.head.apply_memoref(&memoref, &slab).await?;

        self.update_referents( context ).await?;

        Ok(true)
    }
    pub async fn set_relation (&mut self, context: &Context, key: RelationSlotId, relation: &Self) -> Result<(),WriteError> {
        //println!("# Subject({}).set_relation({}, {})", &self.id, key, relation.id);
        let mut relationset = RelationSet::empty();
        relationset.insert( key, relation.id );

        // TODO - do this in a single swap? May require unsafe
        let mut head = MemoRefHead::Null;
        std::mem::swap(&mut head,&mut self.head);

        let mut new_head = context.slab.new_memo(
            Some(self.id),
            head,
            MemoBody::Relation(relationset)
        ).to_head();

        std::mem::swap(&mut self.head, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.head.apply_memoref(&memoref, &slab).await?;

        self.update_referents( context ).await?;

        Ok(())
    }
    pub fn set_edge (&mut self, context: &Context, key: RelationSlotId, edge: &Self) {
        //println!("# Subject({}).set_edge({}, {})", &self.id, key, relation.id);
        let mut edgeset = EdgeSet::empty();
        edgeset.insert( key, edge.get_head() );

        // TODO - do this in a single swap? May require unsafe
        let mut head = MemoRefHead::Null;
        std::mem::swap(&mut head,&mut self.head);

        let mut new_head = context.slab.new_memo(
                Some(self.id),
                head,
                MemoBody::Edge(edgeset)
            ).to_head();

        std::mem::swap(&mut self.head, &mut new_head);

        // We shouldn't need to apply the new memoref. It IS the new head
        // self.head.apply_memoref(&memoref, &slab).await?;
    }
    // // TODO: get rid of apply_head and get_head in favor of Arc sharing heads with the context
    // pub fn apply_head (&self, context: &Context, new: &MemoRefHead){
    //     //println!("# Subject({}).apply_head({:?})", &self.id, new.memo_ids() );

    //     let slab = context.slab.clone(); // TODO: find a way to get rid of this clone

    //     //println!("# Record({}) calling apply_memoref", self.id);
    //     self.head.write().unwrap().apply(&new, &slab);
    // }
    pub fn get_head (&self) -> MemoRefHead {
        self.head.clone()
    }
    // pub fn get_contextualized_head(&self, context: &Context) -> MemoRefHead {
    //     let mut head = self.head.read().unwrap().clone();
    //     head.apply( &context.get_resident_subject_head(self.id), &context.slab );
    //     head
    // }
    #[tracing::instrument]
    pub async fn get_all_memo_ids ( &self, slab: SlabHandle ) -> Result<Vec<MemoId>,RetrieveError> {
        let mut memostream = self.head.causal_memo_stream( slab );

        let mut memo_ids = Vec::new();
        while let Some(memo) = memostream.next().await {
            memo_ids.push(memo?.id);
        }
        Ok(memo_ids)
    }
    // pub fn is_fully_materialized (&self, context: &Context) -> bool {
    //     self.head.read().unwrap().is_fully_materialized(&context.slab)
    // }
    // pub fn fully_materialize (&self, _slab: &Slab) -> bool {
    //     unimplemented!();
    //     //self.shared.lock().unwrap().head.fully_materialize(slab)
    // }

    pub fn observe (&self, slab: &SlabHandle) -> mpsc::Receiver<MemoRefHead> {
        // get an initial value, rather than waiting for the value to change
        let (tx, rx) = mpsc::channel(1000);
//        tx.send( self.head.clone() ).wait().unwrap();

        // BUG HERE - not applying MRH to our head here, but double check as to what we were expecting from indexes
        slab.observe_subject( self.id, tx );

        rx
    }
}

impl Clone for Subject {
    fn clone (&self) -> Subject {
        Self{
            id: self.id,
            head: self.head.clone()
        }
    }
}
impl Drop for Subject {
    fn drop (&mut self) {
        //println!("# Subject({}).drop", &self.id);
        // TODO: send a drop signal to the owning context via channel
        // self.drop_channel.send(self.id);
    }
}
impl fmt::Debug for Subject {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Subject")
            .field("subject_id", &self.id)
            .field("head", &self.head)
            .finish()
    }
}