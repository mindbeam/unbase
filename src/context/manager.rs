#![warn(bad_style, missing_docs,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]

use super::*;
use memorefhead::{RelationSlotId,RelationLink};

use std::collections::HashSet;
use std::collections::hash_map::Entry;

//TODO: farm the guts of this out to it's own topo-sort accumulator crate
//      using gratuitous Arc<Mutex<>> for now which will later be converted to unsafe Mutex<Rc<Item>>

pub struct SubjectHead {
    pub subject_id: SubjectId,
    pub head:       MemoRefHead,
    pub from_subject_ids: Vec<SubjectId>
}

type ItemId = usize;

#[derive(Clone)]
struct Item {
    subject_id: SubjectId,
    indirect_references: isize,
    head: Option<MemoRefHead>,
    relations: Vec<Option<ItemId>>
}

/// Performs topological sorting.
pub struct ContextManager {
    items: Vec<Option<Item>>,
    vacancies: Vec<ItemId>
}

impl Item {
    fn new( subject_id: SubjectId, maybe_head: Option<MemoRefHead> ) -> Self {
        Item {
            subject_id: subject_id,
            head: maybe_head,
            indirect_references: 0,
            relations: Vec::new(),
        }
    }
}

impl ContextManager {
    pub fn new() -> ContextManager {
        ContextManager {
            items:     Vec::with_capacity(30),
            vacancies: Vec::with_capacity(30)
        }
    }

    /// Returns the number of elements in the `ContextManager`.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the `ContextManager` contains no entries.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn subject_ids(&self) -> Vec<SubjectId> {
        self.items.iter().filter_map(|i| {
            if let &Some(item) = i {
                Some(item.subject_id)
            }else{
                None
            }
        }).collect()
    }
    pub fn get_head (&self, subject_id: SubjectId) -> Option<&MemoRefHead> {
        if let Some(&Some(ref item)) = self.items.iter().find(|i| {
            if let &&Some(ref it) = i {
                it.subject_id == subject_id
            }else{
                false
            }
        }) {
            item.head.as_ref()
        }else{
            None
        }
    }

    /// Update the head for a given subject. The previous head is summarily overwritten.
    /// Any mrh.apply to the previous head must be done externally, if desired
    /// relation_links must similarly be pre-calculated
    pub fn set_subject_head(&mut self, subject_id: SubjectId, head: MemoRefHead, relation_links: Vec<RelationLink> ) {
        let item_id = { self.assert_item(subject_id) };
        if let Some(ref mut item) = self.items[item_id] {
            item.head = Some(head);
        }

        for link in relation_links {
            self.set_relation(item_id, link);
        }
    }

    /// Creates or returns a ContextManager item for a given subject_id
    fn assert_item( &mut self, subject_id: SubjectId ) -> ItemId {
        if let Some(item_id) = self.items.iter().position(|i| {
            if let &Some(ref it) = i {
                it.subject_id == subject_id
            }else{
                false
            }
        }){
            item_id
        }else{
            let item = Item::new(subject_id, None);
            let item_id = if let Some(item_id) = self.vacancies.pop(){
                item_id
            }else{
                self.items.len()
            };

            self.items[item_id] = Some(item);
            item_id

        }
    }

    fn set_relation (&mut self, item_id: ItemId, link: RelationLink ){

        //let item = &self.items[item_id];
        // retrieve existing relation by SlotId as the vec offset
        // Some(&Some()) due to empty vec slot vs None relation (logically equivalent)
        let item = {
            if let Some(ref item) = self.items[item_id] {
                item
            }else{
                panic!("sanity error. set relation on item that does not exist")
            }
        };

        if let Some(&Some(rel_item_id)) = item.relations.get(link.slot_id as usize){
            // relation exists

            let decrement;
            {
                if let &Some(ref rel_item) = &self.items[rel_item_id] {

                    // no change. bail out. do not increment or decrement
                    if Some(rel_item.subject_id) == link.subject_id {
                        return;
                    }

                    decrement = 0 - (1 + item.indirect_references);
                }else{
                    panic!("sanity error. relation item_id located, but not found in items")
                }
            }

            // ruh roh, we're different. Have to back out the old relation
            let mut removed = vec![false; self.items.len()];
            self.increment(rel_item_id, decrement, &mut removed);
            // item.relations[link.slot_id] MUST be set below
        };

        if let Some(subject_id) = link.subject_id {
            let new_rel_item_id = self.assert_item(subject_id);

            let increment;
            {
                if let &mut Some(item) = &mut self.items[item_id] {
                    item.relations[link.slot_id as usize] = Some(new_rel_item_id);
                    increment = 1 + item.indirect_references;
                }else{
                    panic!("sanity error. relation just set")
                }
            }

            let mut added = vec![false; self.items.len()];
            self.increment(new_rel_item_id, increment, &mut added );
        }else{
            // sometimes this will be unnecessary, but it's essential to overwrite a Some() if it's there
            if let &mut Some(item) = &mut self.items[item_id] {
                item.relations[link.slot_id as usize] = None;
            }else{
                panic!("sanity error. relation item not found in items")
            }
        }
    }
    fn increment(&mut self, item_id: ItemId, increment: isize, seen: &mut Vec<bool> ) {
        // Avoid traversing cycles
        if Some(&true) == seen.get(item_id){
            return; // dejavu! Bail out
        }
        seen[item_id] = true;

        let indirect_references;
        let relations : Vec<ItemId>;

        {
            if let &mut Some(item) = &mut self.items[item_id] {
                item.indirect_references += increment;
                assert!(item.indirect_references >= 0, "sanity error. indirect_references below zero");

                indirect_references = item.indirect_references;
                relations = item.relations.iter().filter_map(|r| *r).collect();
            }else{
                panic!("sanity error. increment for item_id");
            }
        };

        if indirect_references == 0 {
            self.items[item_id] = None; // important to preserve ordering. cheaper than doing a sort again I think
        }

        for rel_item_id in relations{
            self.increment( 123, increment, seen );
        }

    }
    pub fn subject_head_iter(&self) -> SubjectHeadIter {
        // TODO: make this respond to context changes while we're mid-iteration.
        // Approach A: switch Vec<Item> to Arc<Vec<Option<Item>>> and avoid slot reclamation until the iter is complete
        // Approach B: keep Vec<item> sorted (DESC) by indirect_references, and reset the increment whenever the sort changes

        // FOR now, taking the low road
        // Vec<(usize, MemoRefHead, Vec<SubjectId>)>
        let sorted = self.items.iter().filter_map(|i| {
            if let &Some(ref item) = i {
                if let Some(ref head) = item.head{

                    let relation_subject_ids : Vec<SubjectId> = item.relations.iter().filter_map(|maybe_item_id|{
                        if let &Some(item_id) = maybe_item_id {
                            if let Some(ref item) = self.items[item_id] {
                                Some(item.subject_id)
                            }else{
                                panic!("sanity error, subject_head_iter")
                            }
                        }else{
                            None
                        }
                    }).collect();
                    return Some((item.indirect_references as usize, head.clone(), relation_subject_ids));
                }
            }
            None
        }).collect();

        SubjectHeadIter{
            sorted: sorted
        }
    }
}

struct SubjectHeadIter {
    sorted: Vec<(usize, MemoRefHead, Vec<SubjectId>)>
}
impl Iterator for SubjectHeadIter {
    type Item = SubjectHead;

    fn next(&mut self) -> Option<SubjectHead> {
        //self.pop()
        unimplemented!()
    }
}


#[cfg(test)]
mod test {

    use super::ContextManager;

    #[test]
    fn iter() {
        let net = unbase::Network::create_new_system();
        let slab = unbase::Slab::new(&net);
        let mut manager = ContextManager::new();

        let head1 = slab.new_memo_basic_noparent( Some(1), MemoBody::FullyMaterialized {v: vals, r: RelationSlotSubjectHead::empty() } ).to_head();
        manager.add_subject_head(1, head1, head1.project_all_relation_links() );

        let head2 = slab.new_memo_basic_noparent( Some(2), MemoBody::FullyMaterialized {v: vals, r: RelationSlotSubjectHead::single(0, 1, head1 )} ).to_head();
        manager.add_subject_head(2, head2, head2.project_all_relation_links() );

        let head3 = slab.new_memo_basic_noparent( Some(3), MemoBody::FullyMaterialized {v: vals, r: RelationSlotSubjectHead::single(0, 2, head2 )} ).to_head();
        manager.add_subject_head(3, head3, head3.project_all_relation_links() );

        let head4 = slab.new_memo_basic_noparent( Some(4), MemoBody::FullyMaterialized {v: vals, r: RelationSlotSubjectHead::single(0, 3, head2 )} ).to_head();
        manager.add_subject_head(4, head4, head4.project_all_relation_links() );

        let iter = manager.iter();
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(Some(3), iter.next());
        assert_eq!(Some(4), iter.next());
        assert_eq!(None, iter.next());
    }
}
