
use super::*;

pub struct TopoSubjectHeadIter {
    context: Context,
    slab: Slab,
    subject_ids: Vec<SubjectId>
}

impl TopoSubjectHeadIter {
    pub fn new (context: &Context) -> TopoSubjectHeadIter {
        
        // TODO: Do this in a less ridiculous way,
        //       and move it into ContextSubjectHeadIter::new
        let subject_ids : Vec<SubjectId>;
        {
            let shared = context.inner.shared.lock().unwrap();
            subject_ids = shared.subject_heads.keys().map(|k| k.to_owned()).collect();
        }
        TopoSubjectHeadIter {
            subject_ids: subject_ids,
            context: context.clone(),
            slab: context.get_slab().clone()
        }
    }
}

impl Iterator for TopoSubjectHeadIter {
    type Item = (SubjectId, MemoRefHead);
    fn next (&mut self) -> Option<(SubjectId, MemoRefHead)> {

        //NOTE: Some pretttyy shenanegous stuff here, but taking the
        //      low road for now in the interest of time. Playing
        //      stupid games to try to avoid a deadlock with the slab
        //      inserting new memos mid-iteration via update_subject_head
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
