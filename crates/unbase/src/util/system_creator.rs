use std::collections::HashMap;

use crate::{
    memorefhead::MemoRefHead,
    slab::{
        EdgeSet,
        MemoBody,
        RelationSet,
        SlabHandle,
        SubjectType,
    }
};

pub struct SystemCreator;

impl SystemCreator {

    pub fn generate_root_index_seed( slab: &SlabHandle ) -> MemoRefHead {

        let mut values = HashMap::new();
        values.insert("tier".to_string(),0.to_string());

        let memoref = slab.new_memo_noparent(
            Some(slab.generate_subject_id(SubjectType::IndexNode)),
            MemoBody::FullyMaterialized {
                v: values,
                r: RelationSet::empty(),
                e: EdgeSet::empty(),
                t: SubjectType::IndexNode
            }
        );

        memoref.to_head()
    }

}