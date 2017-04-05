use std::collections::HashMap;
use slab::Memo;
use slab::memo::MemoBody;
use memorefhead::MemoRefHead;
use slab::{SlabShared,MemoOrigin};

pub struct SystemCreator;

impl SystemCreator {

    pub fn generate_root_index_seed( slab_inner: &SlabShared ) -> MemoRefHead {

        let mut values = HashMap::new();
        values.insert("tier".to_string(),0.to_string());

        let memoref = slab_inner.new_memo_basic_noparent(
            Some(slab_inner.generate_subject_id()),
            MemoBody::FullyMaterialized {v: values, r: HashMap::new() }
        );

        MemoRefHead::from_memoref(memoref)
    }

}
