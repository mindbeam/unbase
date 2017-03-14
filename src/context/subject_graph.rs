use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use super::*;
use memorefhead::*;

pub struct SubjectChildEdge{
    child_id:    SubjectId,
    parent_slot: RelationSlotId,
}
pub struct SubjectParentEdge{
    parent_id:   SubjectId,
    parent_slot: RelationSlotId,
}

pub struct SubjectGraph {
    parent_cache: HashMap<SubjectId, [SubjectId; SUBJECT_MAX_RELATIONS]>,
    vertices: HashMap<SubjectId, SubjectVertex>
}

pub struct SubjectVertex {
    pub subject_id: SubjectId,
    pub parents:  Vec<SubjectParentEdge>,
    pub in_degree: usize,
    pub children: Vec<SubjectChildEdge>,
}

impl SubjectGraph {
    pub fn new () -> Self {
        Self {
            parent_cache: HashMap::new(),
            vertices:     HashMap::new()
        }
    }
    //                                         Parent P \/       points to \/
    pub fn update (&mut self, slab: &Slab, parent_id: SubjectId, links: &[SubjectId] ){
        // TODO: Optimize this. Should probably be offset based, and incremental.
        //       Consider using SubjectHead Arc address instead of subject id.
        //       (will be useful for faster subjectHead retrieval too)
        //
        // Rather than extracting every relation every time, we should only do it
        // for the relation(s) that were just updated
        // links = 256 relation slots * 8 bytes each = minimum 2kb per call (will get much worse when we switch to uuid).
        // That's rather expensive given how often we need to do this

        let mut parent_cache = self.parent_cache.entry(parent_id).or_insert( [0; SUBJECT_MAX_RELATIONS] );
        let mut parent_vertex = self.vertices.entry(parent_id).or_insert(SubjectVertex{
            subject_id: parent_id,
            parents: Vec::new(),
            in_degree: 0,
            children: Vec::new()
        });

        // quick and very dirty
        parent_vertex.children = Vec::new();

        for (slot, new_child_id) in links.into_iter().enumerate() {

            let old_child_id = parent_cache[slot];
            // B               A
            if *new_child_id != old_child_id {
                parent_cache[slot] = *new_child_id;
                // Remove backlink from A -> P[0]
                self.remove_backlink(&old_child_id, &(slot as RelationSlotId), &parent_id);

                // Add backlink from B -> P[0]
                self.set_backlink(new_child_id, &(slot as RelationSlotId), &parent_id);
            }
            if *new_child_id > 0 {
                parent_vertex.children.push(SubjectChildEdge{
                    parent_slot: slot as RelationSlotId,
                    child_id: *new_child_id
                })
            }
        }

    }
    fn set_backlink (&mut self, child_id: &SubjectId, slot: &RelationSlotId, parent_id: &SubjectId ) {
        if *child_id == 0 {
            return;
        }

        let mut child_vertex = self.vertices.entry(*child_id).or_insert(SubjectVertex{
            subject_id: *child_id,
            parents: Vec::new(),
            in_degree: 0,
            children: Vec::new()
        });

        child_vertex.parents.push(SubjectParentEdge{
            parent_slot:   *slot,
            parent_id:     *parent_id
        });
        child_vertex.in_degree = child_vertex.parents.len();

    }
    fn remove_backlink (&mut self, child_id: &SubjectId, slot: &RelationSlotId, parent_id: &SubjectId ) {
        if *child_id == 0 {
            return;
        }

        match self.vertices.entry(*child_id){
            Entry::Occupied(mut e) => {

                let mut do_remove : bool = false;
                {
                    let child_vertex = e.get_mut();
                    child_vertex.parents.retain(|l| { l.parent_id != *parent_id && l.parent_slot != *slot });

                    if child_vertex.parents.len() == 0 {
                        do_remove = true;
                    }else{
                        child_vertex.in_degree = child_vertex.parents.len();
                    }
                }
                if do_remove {
                    e.remove();
                }
            }
            _ => {}
        }
    }
    // vertices sorted by in degree
    pub fn least_indegree_vertex (&self) -> Option<&SubjectVertex> {
        unimplemented!();
        //let mut vertices : Vec<&SubjectVertex> = self.vertices.values().collect();
        //vertices.sort_by(|a, b| a.in_degree.cmp(&b.in_degree) );
        //vertices.get(0)
    }
}
