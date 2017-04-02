use std::collections::HashMap;
use slab::Memo;
use slab::memo::MemoBody;
use memorefhead::MemoRefHead;
use slab::{Slab,MemoOrigin};

pub struct SystemCreator;

impl SystemCreator {

    pub fn generate_root_index_seed( slab: &Slab ) -> MemoRefHead {

        let mut values = HashMap::new();
        values.insert("tier".to_string(),0.to_string());

        let memo = Memo::new_basic_noparent(
            slab.gen_memo_id(),
            slab.generate_subject_id(),
            MemoBody::FullyMaterialized {v: values, r: HashMap::new() },
            slab
        );

        let memoref = slab.put_memo(&MemoOrigin::SameSlab, memo);

        MemoRefHead::from_memoref(memoref)
    }

}
