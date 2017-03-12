
use super::*;

struct BidirectionalRef {
    parents: Vec<(SubjectId, u8)>,
    children: Vec<(u8, SubjectId)>
}

pub struct TopoSubjectHeadIter {
    context: Context,
    slab: Slab,
    head_links: HeadLinks
}

// An iterator specifically for the subject heads in our context
// This is not for all subjects which are loaded necessarily
// This means that the subject heads may be disjointed, but a
// topological search is essential to the compaction behaving correctly
impl TopoSubjectHeadIter {
    pub fn new (context: &Context) -> TopoSubjectHeadIter {

        let slab = context.get_slab().clone();
        let shared = context.inner.shared.lock().unwrap();

        TopoSubjectHeadIter {
            head_links: shared.resident_head_links.clone(),
            context: context.clone(),
            slab: context.get_slab().clone()
        }
        
    }
}

impl Iterator for TopoSubjectHeadIter {
    type Item = (SubjectId, MemoRefHead);
    fn next (&mut self) -> Option<(MemoRefHead,Vec<SubjectId>,Vec<SubjectId>)> {

        if let Some(subject_id) = self.subject_ids.pop() {
            if let Some(head) = self.context.inner.shared.lock().unwrap().subject_heads.get(&subject_id) {
                Some((subject_id,head.clone()))
            }else{
                None
            }
        }else{
            None
        }
    }
}
impl DoubleEndedIterator for TopoSubjectHeadIter {
    fn next_back (&mut self) -> Option<(SubjectId, MemoRefHead)> {
        // temporary
        Some((1,MemoRefHead::new()))
    }
}
