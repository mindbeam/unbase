/* Memo
 * A memo is an immutable message.
*/

use slab::Slab;
use std::{fmt};



/*
use std::hash::{Hash, Hasher};

pub struct MemoId{
    originSlab: u32,
    id: u32,
}
impl Hash for MemoId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.originSlab.hash(state);
        self.id.hash(state);
    }
}
*/


pub struct Memo {
    pub id: u64,
//    type: TypeName enum {
//        Beacon
//    }
}
impl Clone for Memo {
    fn clone(&self) -> Memo {
        Memo {
            id: self.id,
        }
    }
}

impl fmt::Debug for Memo{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Memo")
           .field("id", &self.id)
           .finish()
    }
}

impl Memo {
    pub fn new (slab : &Slab) { // -> Memo{ // , topic: Topic){
        let me = Memo {
            id:    slab.gen_memo_id()
            //topic: topic,
//            type:  Beacon
        };

        println!("New Memo: {:?}", me.id );
        slab.put_memo(me);
        //me
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
