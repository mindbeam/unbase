use std::collections::HashMap;
use memo::{Memo,MemoBody};
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
            MemoBody::FullyMaterialized {v: values, r: HashMap::new() }
        );

        let memorefs = slab.put_memos(&MemoOrigin::SameSlab, vec![ memo.clone() ]);

        MemoRefHead::from_memoref(memorefs[0].clone())
    }

}
