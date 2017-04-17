
use petgraph::{Graph,Direction};
use petgraph::graph::{NodeIndex};
use super::*;
use memorefhead::RelationSlotId;
use std::collections::HashMap;
use std::collections::hash_map::Entry;


/// Not super in love with the name here.
/// The point of SubjectGraph is threefold:
/// 1. Contain the per-subject MemorefHeads corresponding to our present query context sufficient to enforce consistency model invariants.
/// 2. Maintain a projection of relations between these heads, sufficient to perform a topological iterataion over the subject heads
/// 3. Facilitate the compression of these heads on the basis of the above

pub struct SubjectGraph {
    graph: Graph<SubjectId,RelationSlotId>,
    subject_map: HashMap<SubjectId,(Option<MemoRefHead>, NodeIndex)>
}

impl SubjectGraph {
    pub fn new () -> Self {

        Self {
            graph:       Graph::new(),
            subject_map: HashMap::new()
        }
    }
    pub fn apply_memoref (&mut self, subject_id: SubjectId, memoref: &MemoRef, slab: &Slab) {
        // TODO optimize this
        let relation_links;
        let from_node;
        {
            match self.subject_map.entry(subject_id) {
                Entry::Occupied(mut e) => {
                    let mut tuple = e.get_mut();
                    match tuple{
                        &mut (Some(ref mut mrh), ref node) => {
                            mrh.apply_memoref(memoref, slab);
                            from_node = *node;
                            relation_links = mrh.project_all_relation_links( slab );
                        }
                        &mut (None, ref node) => {
                            let new_mrh = memoref.to_head();
                            from_node = *node;
                            relation_links = new_mrh.project_all_relation_links( slab );
                            tuple.0 = Some(new_mrh);
                        }
                    }
                }

                Entry::Vacant(e) => {
                    let node = self.graph.add_node(subject_id);
                    let new_mrh = memoref.to_head();

                    from_node = node;
                    relation_links = new_mrh.project_all_relation_links( slab );

                    e.insert( (Some(new_mrh), node) );
                }
            };
        }

        // This code sucks. TODO: Optimize it. probably get rid of petgraph in favor of something much simpler / tailored to this use case
        // Just brute forcing this for now. Should also be doing this on an incremental basis

        // Add Nodes and edges for all present relations
        // remove edges and nodes for all old relations
        let mut removed_slots = Vec::new();
        for (slot_id, to_subject_id) in relation_links.iter().enumerate() {
            //let to_node = self.assert_node(*to_subject_id);
            // HACK - for now we can assume that every relationship slot will be covered, at least with a zero
            //        Will need to update this later so that omitted relationships are removed from the graph
            if *to_subject_id > 0 {
                let to_node = match self.subject_map.entry(*to_subject_id) {
                    Entry::Occupied(e) => e.get().1,
                    Entry::Vacant(e)   => e.insert((None, self.graph.add_node(*to_subject_id))).1
                };
                self.graph.update_edge(from_node, to_node, slot_id as RelationSlotId);
            }else{
                removed_slots.push(slot_id as RelationSlotId);
                //self.graph.remove_edge(from_node, to_node);
            }
        }

        let mut remove_edges = Vec::with_capacity(removed_slots.len());
        use petgraph::visit::EdgeRef;
        for edge in self.graph.edges(from_node) {
            let slot_id = edge.weight();
            if removed_slots.contains(slot_id) {
                remove_edges.push(edge.id());
            }
        }

        for edge_idx in remove_edges {
             self.graph.remove_edge(edge_idx);
        }
    }
    pub fn apply_head (&mut self, subject_id: SubjectId, apply_mrh: &MemoRefHead, slab: &Slab ) {
        //TODO: optimize this
        for memoref in apply_mrh.iter() {
            self.apply_memoref(subject_id,memoref,slab);
        }
    }
    pub fn get_head(&self, subject_id: SubjectId ) -> Option<&MemoRefHead> {
        if let Some(tuple) = self.subject_map.get(&subject_id) {
            if let Some(ref mrh) = tuple.0 {
                return Some(mrh);
            }
        };

        None
    }
    pub fn remove(&mut self, subject_id: SubjectId){
        //unimplemented!()
    }
    pub fn subject_ids(&self) -> Vec<SubjectId> {
        let mut subject_ids : Vec<SubjectId> = Vec::new();
        for (subject_id, &(ref maybe_mrh, _ ) ) in self.subject_map.iter() {
            if let &Some(_) = maybe_mrh {
                subject_ids.push((*subject_id).clone());
            }
        }

        subject_ids
    }
    pub fn head_iter(&self) -> SubjectHeadIter {
        SubjectHeadIter{}
    }
    /*
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
    */
}

struct SubjectHeadIter {

}
impl Iterator for SubjectHeadIter {
    type Item = (SubjectId,MemoRefHead);
    fn next (&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
