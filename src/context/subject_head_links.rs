use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::*;

pub struct SubjectHeadLinks {
    forward: HashMap<SubjectId, Vec<SlotSubjectLink>>,
    backward: HashMap<SubjectId, Vec<SlotSubjectLink>>
}

impl SubjectHeadLinks {
    pub fn new () -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new()
        }
    }
    pub fn update (&mut self, slab: &Slab, subject_id: SubjectId, head: &MemoRefHead ){
        // TODO: make this incremental. Rather than extracting every relation every
        // time, we should only do it for the relation(s) that were just updated

        let relations : Vec<u8,(SubjectId,&MemoRefHead)> = head.get_all_relations( slab );

        let forward : Vec<SlotSubjectLink> = self.forward.entry(subject_id).or_insert(Vec::new());

        for slotlink in forward {
            // my_slot,other_subject_id

            match relations.entry( slotlink.slot ) {
                Entry::vacant(e) => {
                    //delete slot from foward
                }
                Entry::occupied(e) => {
                    e.set(slotlink.subject_id); // just in case it's changed
                    e.remove();
                }
            }
        }

        for (slot,(to_subject_id, _)) in relations {
            forward.push(SlotSubjectLink{ slot: slot, subject_id: to_subject_id });
            match self.backward.entry(to_subject_id) {
                Entry::Occupied(e) => {
                    for back in e.get() { // Vec of reverse links to all slots
                        // Yeah, this algo is stupid
                    }
                }
            }
        }


        for (slot, (to_subject_id, to_head)) in relations.iter() {


        }


                for (parent_subject_id,head) in shared.subject_heads.iter() {
                    // Left off here
                    let all_relations = head.get_all_relations( &slab );
                    // parent -> child
                    all_relations.map(|(slot,(to_subject_id, _))| {
                        // HERE HERE HERE - check this
                        refs.insert(parent_subject_id, BidirectionalRef{
                            parents:vec![],
                            children: (slot, parent_subject_id)
                        } );
                    });

                    // child -> parent
                    for (slot, child_subject_id, head) in all_relations.iter() {
                        // child -> (by_parent_subject, ref_slot)
                        refs.entry(*child_subject_id).or_insert().set(slot, parent_subject_id)
                    }
                }

                //let subject_ids = shared.subject_heads.keys().map(|k| k.to_owned()).collect();

    }
    pub fn remove (&mut self, subject_id: SubjectId) {

    }
}
