use std::collections::HashMap;

use crate::{
    head::Head,
    slab::{
        EdgeSet,
        EntityType,
        MemoBody,
        RelationSet,
        SlabHandle,
    },
};

pub struct SystemCreator;

impl SystemCreator {
    pub fn generate_root_index_seed(slab: &SlabHandle) -> Head {
        let mut values = HashMap::new();
        values.insert("tier".to_string(), 0.to_string());

        let memoref = slab.new_memo_noparent(Some(slab.generate_entity_id(EntityType::IndexNode)),
                                             MemoBody::FullyMaterialized { v: values,
                                                                           r: RelationSet::empty(),
                                                                           e: EdgeSet::empty(),
                                                                           t: EntityType::IndexNode, });

        memoref.to_head()
    }
}
