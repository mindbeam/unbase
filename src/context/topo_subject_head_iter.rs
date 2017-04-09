use std::collections::VecDeque;
use super::*;

pub struct TopoSubjectHeadIter {
    context: Context,
    slab: Slab,
    vertices: VecDeque<SubjectVertex>
}


// An iterator specifically for the subject heads in our context
// This is not for all subjects which are loaded necessarily
// This means that the subject heads may be disjointed, but a
// topological search is essential to the compaction behaving correctly
impl TopoSubjectHeadIter {
    pub fn new (context: &Context) -> TopoSubjectHeadIter {

        let slab = context.slab.clone();
        let shared = context.inner.shared.lock().unwrap();

        let mut topo_sorted_vertices : VecDeque<SubjectVertex> = VecDeque::new();
        let mut subject_graph = shared.subject_graph.clone();

        let mut indegree_sorted_vertices = subject_graph.indegree_sorted_vertices();

        while indegree_sorted_vertices.len() > 0 {

            let vertex : &SubjectVertex =
                    // Take the lowest indegree vertex
                match indegree_sorted_vertices.pop_front() {
                    Some(v) => { v },
                    None    => { break; }
                };

            for child_id in vertex.children.iter() {

            }
                } else {
                    match hopper.pop_front() {
                        Some(subject_id) => {
                            // ok this is a little weird
                            match indegree_sorted_vertices.iter().find(|v| v.subject_id == subject_id ) {
                                Some(v) => {
                                    v
                                }
                                None => {
                                    panic!("Sanity error");
                                }
                            }

                        },
                        None    => { continue; }
                    }
                };

            topo_sorted_vertices.push_back(*vertex);
            for child_id in (){
            hopper.push_back();

        }
        TopoSubjectHeadIter {
            subject_ids: subject_ids,
            context: context.clone(),
            slab: context.slab.clone()
        }

    }
}

impl Iterator for TopoSubjectHeadIter {
    type Item = (SubjectId,MemoRefHead,Vec<SubjectId>,Vec<SubjectId>);
    fn next (&mut self) -> Option<(SubjectId,MemoRefHead,Vec<SubjectId>,Vec<SubjectId>)> {

        if let Some(vertex) = self.vertices.pop_front() {
            if let Some(head) = self.context.inner.shared.lock().unwrap().subject_heads.get(&subject_id) {
                Some((vertex.subject_id,head.clone(), vertex.inbound, vertex.outbound ))
            }else{
                None
            }
        }else{
            None
        }
    }
}
impl DoubleEndedIterator for TopoSubjectHeadIter {
    fn next_back (&mut self) -> Option<(SubjectId,MemoRefHead,Vec<SubjectId>,Vec<SubjectId>)> {
        if let Some(vertex) = self.vertices.pop_back() {
            if let Some(head) = self.context.inner.shared.lock().unwrap().subject_heads.get(&subject_id) {
                Some((vertex.subject_id,head.clone(), vertex.inbound, vertex.outbound ))
            }else{
                None
            }
        }else{
            None
        }
    }
}
