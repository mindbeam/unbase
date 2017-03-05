use std::mem;
use std::fmt;
use std::slice;
use std::collections::VecDeque;
use memo::*;
use memoref::*;
use slab::*;

#[derive(Clone)]
pub struct MemoRefHead (Vec<MemoRef>);

impl MemoRefHead {
    pub fn new () -> Self {
        MemoRefHead( Vec::with_capacity(5) )
    }
    pub fn from_memoref (memoref: MemoRef) -> Self {
        MemoRefHead( vec![memoref] )
    }
    pub fn apply_memoref(&mut self, mut new: MemoRef, slab: &Slab ) -> bool {
        println!("# MemoRefHead({:?}).apply_memoref({})", self.memo_ids(), &new.id);

        // Conditionally add the new memoref only if it descends any memorefs in the head
        // If so, any memorefs that it descends must be removed

        // Not suuuper in love with these flag names
        let mut new_is_descended = false;
        let mut new_descends  = false;

        let mut applied  = false;
        let mut replaced  = false;

        // I imagine it's more efficient to iterate in reverse, under the supposition that
        // new items are more likely to be at the end, and that's more likely to trigger
        // the cheapest case: (existing descends new)

        'existing: for i in (0..self.0.len()).rev() {
            let mut remove = false;
            {
                let ref mut existing = self.0[i];
                if *existing == new {
                    return false; // we already had this

                } else if existing.descends(&new,&slab) {
                    new_is_descended = true;

                    // IMPORTANT: for the purposes of the boolean return,
                    // the new memo does not get "applied" in this case

                    // If any memo in the head already descends the newcomer,
                    // then it doesn't get applied at all punt the whole thing
                    break 'existing;

                } else if new.descends(&existing, &slab) {
                    new_descends = true;
                    applied = true; // descends

                    if replaced {
                        remove = true;
                    }else{
                        // Lets try real hard not to remove stuff in the middle of the vec
                        // But we only get to do this trick once, because we don't want to add duplicates
                        mem::replace( existing, new.clone() );
                        replaced = true;
                    }

                }
            }

            if remove {
                // because we're descending, we know the offset of the next items won't change
                self.0.remove(i);
            }
        }

        if !new_descends && !new_is_descended  {
            // if the new memoref neither descends nor is descended
            // then it must be concurrent

            self.0.push(new);
            applied = true; // The memoref was "applied" to the MemoRefHead
        }

        // This memoref was applied if it was concurrent, or descends one or more previous memos

        if applied {
            println!("# \t\\ Was applied - {:?}", self.memo_ids());
        }else{
            println!("# \t\\ NOT applied - {:?}", self.memo_ids());
        }

        applied
    }
    pub fn apply_memorefs (&mut self, new_memorefs: Vec<MemoRef>, slab: &Slab) {
        for new in new_memorefs{
            self.apply_memoref(new, slab);
        }
    }
    pub fn apply (&mut self, other: &MemoRefHead, slab: &Slab){
        for new in other.iter(){
            self.apply_memoref(new.clone(), slab );
        }
    }
    pub fn memo_ids (&self) -> Vec<MemoId> {
        self.0.iter().map(|m| m.id).collect()
    }
    pub fn to_vecdeque (&self) -> VecDeque<MemoRef> {
        VecDeque::from(self.0.clone())
    }
    pub fn len (&self) -> usize {
        self.0.len()
    }
    pub fn iter (&self) -> slice::Iter<MemoRef> {
        self.0.iter()
    }
}

impl fmt::Debug for MemoRefHead{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        fmt.debug_struct("MemoRefHead")
           .field("memo_ids", &self.memo_ids() )
           .finish()
    }
}
