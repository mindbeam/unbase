/* Memo
 * A memo is an immutable message.
*/

use std::collections::HashMap;
use std::{fmt};
use std::sync::Arc;
use subject::{SubjectId};
use memoref::*;
use memorefhead::*;
use network::SlabRef;
use slab::Slab;

//pub type MemoId = [u8; 32];
pub type MemoId = u64;


#[derive(Debug,Clone,PartialEq)]
pub enum PeeringStatus{
    Resident,
    Participating,
    NonParticipating
}

#[derive(Debug)]
pub enum MemoBody{
    Relation(HashMap<u8,(SubjectId,MemoRefHead)>),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    PartiallyMaterialized { v: HashMap<String, String>, r: HashMap<u8,(SubjectId,MemoRefHead)> },
    Peering(MemoId,SlabRef,PeeringStatus),
    MemoRequest(Vec<MemoId>,SlabRef)
}

// All portions of this struct should be immutable

#[derive(Clone)]
pub struct Memo {
    pub id: u64,
    pub subject_id: u64,
    pub inner: Arc<MemoInner>
}
pub struct MemoInner {
    pub id: u64,
    pub subject_id: u64,
    parents: MemoRefHead,
    pub body: MemoBody
}


/*
use std::hash::{Hash, Hasher};

impl Hash for MemoId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.originSlab.hash(state);
        self.id.hash(state);
    }
}
*/

impl fmt::Debug for Memo{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = &self.inner;
        fmt.debug_struct("Memo")
           .field("id", &inner.id)
           .field("subject_id", &inner.subject_id)
           .field("parents", &inner.parents)
           .field("body", &inner.body)
           .finish()
    }
}

impl Memo {
    pub fn new (id: MemoId, subject_id: SubjectId, parents: MemoRefHead, body: MemoBody) -> Memo {

        println!("# Memo.new(id: {},subject_id: {}, parents: {:?}, body: {:?})", id, subject_id, parents.memo_ids(), body );

        let me = Memo {
            id:    id,
            subject_id: subject_id,
            inner: Arc::new(MemoInner {
                id:    id,
                subject_id: subject_id,
                parents: parents,
                body: body
            })
        };

        //println!("# New Memo: {:?}", me.inner.id );
        me
    }
    pub fn new_basic (id: MemoId, subject_id: SubjectId, parents: MemoRefHead, body: MemoBody) -> Self {
        Self::new(id, subject_id, parents, body)
    }
    pub fn new_basic_noparent (id: MemoId, subject_id: SubjectId, body: MemoBody) -> Self {
        Self::new(id, subject_id, MemoRefHead::new(), body)
    }
    pub fn get_parent_head (&self) -> MemoRefHead {
        self.inner.parents.clone()
    }
    pub fn get_values (&self) -> Option<(HashMap<String, String>,bool)> {

        match self.inner.body {
            MemoBody::Edit(ref v)
                => Some((v.clone(),false)),
            MemoBody::FullyMaterialized { ref v, r: _ }
                => Some((v.clone(),true)),
            _   => None
        }
    }
    pub fn get_relations (&self) -> Option<(HashMap<u8, (SubjectId, MemoRefHead)>,bool)> {

        match self.inner.body {
            MemoBody::Relation(ref r)
                => Some((r.clone(),false)),
            MemoBody::FullyMaterialized { v: _, ref r }
                => Some((r.clone(),true)),
            _   => None
        }
    }
    pub fn does_peering (&self) -> bool {
        match self.inner.body {
            MemoBody::MemoRequest(_,_) => {
                false
            }
            MemoBody::Peering(_,_,_) => {
                false
            }
            _ => {
                true
            }
        }
    }
    pub fn descends (&self, memoref: &MemoRef, slab: &Slab) -> bool {
        //TODO: parallelize this
        //TODO: Use sparse-vector/beacon to avoid having to trace out the whole lineage
        //      Should be able to stop traversal once happens-before=true. Cannot descend a thing that happens after


        // breadth-first
        for parent in self.inner.parents.iter() {
            if parent == memoref {
                return true
            };
        }

        // Ok now depth
        for parent in self.inner.parents.iter() {
            if parent.descends(&memoref,slab) {
                return true
            }
        }
        return false;
    }
}

/*
function Memo(slab,memo_id,record_id,peerings,parents,precursors,vals) {
    var me = this;
    me.id  = memo_id;
    me.rid = record_id;
    me.v   = vals;
    me.parents = parents || [];
    me.precursors = precursors || [];

    me.slab = slab;
    peerings = peerings ? JSON.parse(JSON.stringify(peerings)) : {};

    // Temporary hack - doing the value init here out of convenience
    // because edit propagation doesn't work yet. relying in the initial pushMemoToSlab for preliminary testing
    vals = vals || {};
    var val;
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Record ){
                vals[key] = val.id;
                peerings[val.id] = {};
                peerings[val.id][slab.id] = 2; // cheating with just assuming the peer_type here
            }else{
                throw "need a slab id AND a record id";
            }
            // else, should already be a valid record id
            // TBD: how to convey locations of said record id

        }

    });

    if( Object.keys(peerings).length  ){
        slab.updateMemoPeerings(this,peerings);
    }

    slab.putMemo(this);

}

// export the class
module.exports.create = function(slab,record_id,parents,precursors,vals){

    var memo_id ='M.' + slab.genChildID();
    return new Memo(slab,memo_id,record_id,null,parents,precursors,vals);

};

Memo.prototype._evicting    = 0;
Memo.prototype.__replica_ct = 2;

// should we un-set this if an eviction fails?
Memo.prototype.evicting = function(v) {
    this._evicting = v ? 1 : 0;
};

Memo.prototype.desiredReplicas = function() {
   return Math.max(0,(this.__replica_ct - this.slab.getMemoPeers(this.id,true).length) + this._evicting);
};

Memo.prototype.getPrecursors = function(){
    return this.precursors;
};

Memo.prototype.packetize = function(){
    /*
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Memo ) vals[key] = val.id;
            // else, should already be a valid memo id
            // TBD: how to convey locations of said memo id
        }
    });
    */

    return {
        id:  this.id,
        rid: this.rid,
        v:   this.v,
        p:   this.slab.getPeeringsForMemo(this,true),
        r:   this.parents,
        o:   this.precursors
    }
}

module.exports.depacketize = function(slab, packet){
    if(typeof packet != 'object') return null;

    // console.log('memo.depacketize', packet.id, 'into slab', slab.id );
    //console.log(packet);

    var memo_id   = packet.id;
    var record_id = packet.rid;
    var vals      = packet.v;
    var peerings  = packet.p;
    var parents   = packet.r;
    var precursors = packet.o;

    var record = new Memo( slab,memo_id,record_id,peerings,parents,precursors,vals );

    // this is weird. I think this should be based on the payload of the memo, rather than the peering hints
    //slab.setMemoPeering(record, packet.p);
    return record;
}

*/
