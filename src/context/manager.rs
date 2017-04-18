#![warn(bad_style, missing_docs,
        unused, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]

use super::*;
use memorefhead::{RelationSlotId,RelationLink};

use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

//TODO: farm the guts of this out to it's own topo-sort accumulator crate
//      using gratuitous Arc<Mutex<>> for now which will later be converted to unsafe Mutex<Rc<Item>>

pub struct SubjectHead {
    pub subject_id: SubjectId,
    pub head:       MemoRefHead,
    pub from_subject_ids: Vec<SubjectId>
}
struct Item {
    subject_id: SubjectId,
    indirect_references: isize,
    head: Option<MemoRefHead>,
    relations: HashMap<RelationSlotId,Arc<Mutex<Item>>>
}

/// Performs topological sorting.
pub struct ContextManager {
    items: HashMap<SubjectId, Arc<Mutex<Item>>>,
}

impl Item {
    fn new( subject_id: SubjectId, maybe_head: Option<MemoRefHead> ) -> Self {
        Item {
            subject_id: subject_id,
            head: maybe_head,
            indirect_references: 0,
            relations: HashMap::new(),
        }
    }
    fn set_relation (&mut self, link: RelationLink, manager: &mut ContextManager ){

        match self.relations.entry(link.slot_id) {
            Entry::Vacant(mut e) => {
                if let Some(subject_id) = link.subject_id {
                    let mut rel_item = manager.assert_item(subject_id);

                    let mut seen = HashSet::new();
                    rel_item.lock().unwrap().increment( &mut seen, 1 + self.indirect_references );
                    e.insert(rel_item);
                }else{
                    // Nothing do see here folks!
                    return;
                }
            }
            Entry::Occupied(mut e) =>{
                if let Some(subject_id) = link.subject_id {

                    let mut seen = HashSet::new();
                    e.get_mut().lock().unwrap().increment( &mut seen, 1 + self.indirect_references );
                }else{
                    // TODO: decrement and remove
                    unimplemented!()
                }
            }
        };
    }

    fn increment(&mut self, seen: &mut HashSet<SubjectId>, increment: isize ) {
        if seen.contains(&self.subject_id){
            return;
        }
        seen.insert(self.subject_id);
        self.indirect_references += increment;

        for (_,rel_item) in self.relations.iter(){
            rel_item.lock().unwrap().increment(seen, increment);
        }
    }
}

impl ContextManager {
    pub fn new() -> ContextManager {
        ContextManager { items: HashMap::new() }
    }

    /// Returns the number of elements in the `ContextManager`.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the `ContextManager` contains no entries.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn subject_ids(&self) -> Vec<SubjectId> {
        self.items.keys().map(|id| *id ).collect()
    }
    pub fn get_head (&self, subject_id: SubjectId) -> Option<MemoRefHead> {
        if let Some(item) = self.items.get( &subject_id ) {
            item.lock().unwrap().head.clone() // item.head is itself an Option
        }else{
            None
        }
    }

    /// Update the head for a given subject. The previous head is summarily overwritten.
    /// Any mrh.apply to the previous head must be done externally, if desired
    /// relation_links must similarly be pre-calculated
    pub fn set_subject_head(&mut self, subject_id: SubjectId, head: MemoRefHead, relation_links: Vec<RelationLink> ) {
        let mut item : Arc<Mutex<Item>> = match self.items.entry(subject_id) {
            Entry::Vacant(e) => {
                let item = Arc::new(Mutex::new(Item::new(subject_id, Some(head))));
                e.insert(item).clone()
            }
            Entry::Occupied(e) => {
                let mut item = e.get();
                item.lock().unwrap().head = Some(head);
                item.clone()
            }
        };

        for link in relation_links {
            item.lock().unwrap().set_relation(link, self);
        }
    }

    /// Creates or returns a ContextManager item for a given subject_id
    fn assert_item( &mut self, subject_id: SubjectId ) -> Arc<Mutex<Item>> {
        match self.items.entry(subject_id) {
            Entry::Vacant(e) => {
                let item = Arc::new(Mutex::new(Item::new(subject_id, None)));
                e.insert(item.clone());
                item
            }
            Entry::Occupied(e) => {
                e.get().clone()
            }
        }
    }

    /// Removes the item that is not depended on by any other items and returns it, or `None` if there is no such item.
    ///
    /// If `pop` returns `None` and `len` is not 0, there is cyclic dependencies.

    /*pub fn pop(&mut self) -> Option<T> {
        self.top
            .iter()
            .filter(|&(_, v)| v.num_prec == 0)
            .next()
            .map(|(k, _)| k.clone())
            .map(|key| {
                let _ = self.remove(&key);
                key
            })
    }*/
    pub fn subject_head_iter(&self) -> SubjectHeadIter {
        // QUESTION: how to make this respond to context changes while we're mid-iteration?
        //           is it even worth it?
        SubjectHeadIter{

        }
    }
}
impl Drop for ContextManager {
    fn drop (&mut self) {
        // Have to de-link all the items, as they may have circular references.
        for (subject_id,ref mut item) in self.items.iter_mut() {
            item.lock().unwrap().relations.clear();
        }
    }
}

struct SubjectHeadIter {
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
