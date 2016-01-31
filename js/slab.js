/**
 * Slab - A storage substrate for memos/memos
 *
 * Purpose:
 *    Stores memos
 *    Manages memo peering
 *    Organizes memos into records
 *
 * The storage portion is very loosely based on Rasmus Andersson's js-lru
 * A doubly linked list-based Least Recently Used (LRU) cache. Will keep most
 * recently used memos while evicting least recently used memos when its limit
 * is reached.
 *
 */


var record_cls = require('./record');

var slab_increment = 0;

function Slab(args) {

    if(slab_increment > 1296)                 throw "cannot create more than 1296 slabs";
    if(typeof args != 'object')               throw "must provide args";

    //if(!args.node || !args.node.length == 8)  throw "must provide 8 digit node id";
    if(!args.id)                              throw "must provide slab id";
    if(!args.mesh)                            throw "must provide mesh object";

    // encode and zerofill the slab id
    //this.id = args.node + ( "00" + (++slab_increment).toString(36)).substr(-2,2);
    this.id = args.id;

    console.log('Initialized Slab', this.id);
    //if(this.id.length != 10) throw "sanity error " + this.id;

    // Temporarily assuming node ids are one character for simplicity
    //if(this.id.length != 3) throw "sanity error " + this.id;
    if(this.id.length != 1) throw "sanity error " + this.id;

    this.child_increment = 0;
    this._idmap = {};
    this._memos_by_record = {};
    this._records_by_id   = {};

    this.size = 0;
    this.quota = args.quota || 5;
    this.limit = args.limit || 10;

    this.mesh = args.mesh;
    this.mesh.registerSlab( this );

    this.local_peerings = {};
    this.ref_peerings   = {};

}


/*
 * convenience method for registering a peering between an memo, and a referenced memo.
 */

Slab.prototype.registerMemoPeering = function(memo, ref_memo_id, remote_slab_id, peer_state, silent ){

    silent = silent || false;
    console.log('slab[' + this.id + '].registerMemoPeering', "Memo:", memo.id, "Ref Memo:", ref_memo_id, "Remote Slab:", remote_slab_id, "Peering State:", peer_state, "Silent:", silent );

    var peerings = {};
    peerings[ref_memo_id] = {};
    peerings[ref_memo_id][remote_slab_id] = peer_state;

    this.updateMemoPeerings(memo, peerings, silent);

}

/*
 * updateMemoPeerings - method for updating peering data for a given memo
 *
 * memo     - the memo object
 * peerings - struct of peerings which we are updating { ref_memo_id : { remote_slab_id: peer_state }, .. }
 * silent   - boolean fflag to indicate that we should abstain from notifying the mesh
 *
 * Peering Definition:
 *     A "peering" is a loose collection of memos which are interested in each other's contact information.
 *     Participants in a peering may or may not even have a copy of the memo in question.
 *
 * Peering states:
 *     0 - Non participatory
 *     1 - Participatory, but does not have a copy of the referenced memo
 *     2 - Participatory, AND has a copy of the referenced memo
 *
 * By default, an instance of an memo peers with other instances of same, across slabs in which it is resident.
 * Upon (final) eviction of an memo from a slab, an update containing a negative peering state is pushed to other participants in the peering.
 *
 */
Slab.prototype.updateMemoPeerings = function( memo, peerings, silent ){
    var me      = this,
        changes = {},
        peer_state
    ;

    console.log('slab[' + me.id + '].updateMemoPeerings', memo.id, peerings, silent);

    // Need to know what memos are referencing what, so we can de-peer from those memos if all references are removed
    // probably can't just ask the memo for its references, because changes would fail to trigger the necessary de-peering
    // need to detect when the reference changes, so we can de-peer the old reference, or reduce the reference count at least.
    var refs = me.local_peerings[memo.id] = me.local_peerings[memo.id] || [];

    Object.keys(peerings).forEach(function(ref_memo_id){

        if(refs.indexOf(ref_memo_id) == -1) refs.push(ref_memo_id);

        // lookup table so that remotes can be updated
        // includes explicit list of local memos so we can remove them, and determine when the
        // reference count hits zero, so we know when it's appropriate to de-peer
        var r   = me.ref_peerings[ref_memo_id] = me.ref_peerings[ref_memo_id] || { memos: [], remotes: {} };
        if( r.memos.indexOf(memo.id) == -1 ) r.memos.push(memo.id); // increase the count

        Object.keys(peerings[ref_memo_id]).forEach(function(remote_slab_id){
            peer_state = peerings[ref_memo_id][remote_slab_id];

            if( (remote_slab_id != me.id) && !r.remotes[remote_slab_id] ){
                r.remotes[remote_slab_id] = peer_state; // 2 means we actually have it, 1 means peering only

                var change = changes[remote_slab_id] = changes[remote_slab_id] || {};
                change[memo.id] = peer_state;
            }
        });
    });

    if(!silent ){
        if( Object.keys(changes).length ) me.mesh.sendPeeringChanges(me.id,changes);
    }
}

Slab.prototype.receivePeeringChange = function(sending_slab_id,change){
    var me         = this,
        peer_state
    ;

    console.log('slab[' + me.id + '].receivePeeringChange', sending_slab_id, change );

    Object.keys(change).forEach(function(memo_id){
        peer_state = change[memo_id];

        if( me.ref_peerings[memo_id] ){
            if( peer_state ){
                me.ref_peerings[memo_id].remotes[sending_slab_id] = peer_state;
            }else{
                delete me.ref_peerings[memo_id].remotes[sending_slab_id];
            }
        }
    });
}

Slab.prototype.deregisterPeeringForMemo = function(memo){
    var me      = this,
        changes = {};

    console.log('slab[' + me.id + '].deregisterPeeringForMemo', memo.id );
    var refs = me.local_peerings[memo.id] || [];

    refs.forEach(function(ref_memo_id){
        var r   = me.ref_peerings[ref_memo_id];
        r.memos = r.memos.filter(function(id){ return id != memo.id });
        // console.log('meow',ref_memo_id, r);
        // TODO this isn't right

        if(r.memos.length == 0){
            Object.keys(r.remotes).forEach(function(remote_slab_id){
                var change = changes[remote_slab_id] = changes[remote_slab_id] || {};
                change[memo.id] = 0; // we no have
            });
            delete me.ref_peerings[ref_memo_id];
        }
    });

    delete me.local_peerings[memo.id];
    me.mesh.sendPeeringChanges(me.id,changes);

}

Slab.prototype.getPeeringsForMemo = function(memo, include_self){
    var me      = this,
        peerings = {};

    var refs = this.local_peerings[memo.id] || [];

    refs.forEach(function(ref_memo_id){
        var remotes = me.ref_peerings[ref_memo_id].remotes;
        var peering = {};
        Object.keys(remotes).forEach(function(key) {
            peering[key] = remotes[key];
        });

        if(include_self) peering[ me.id ] = 2;

        peerings[ ref_memo_id ] = peering;
    });

    return peerings;
}

/*
 * Look up the current peering participants for a given memo id
 *
 */

Slab.prototype.getMemoPeers = function(memo_id,has_memo){
    var r   = this.ref_peerings[memo_id];
    if(!r) return null;

    var remotes = [];

    if(has_memo){
        Object.keys(r.remotes).forEach(function(remote_slab_id){
            if(r.remotes[remote_slab_id] == 2 ) remotes.push(remote_slab_id)
        });
    }else{
        Object.keys(r.remotes).forEach(function(remote_slab_id){
            if(r.remotes[remote_slab_id]) remotes.push(remote_slab_id)
        });
    }

    return remotes;
}

/* Store a memo in this slab, manage LRU */
Slab.prototype.putMemo = function(memo,cb) {
    //if( ! memo instanceof memo_cls ) throw "invalid memo";
    if( this._idmap[memo.id] ) throw "attempt to put memo twice";

    console.log( 'slab[' + this.id + '].putMemo', memo.id, memo.rid );

    this._idmap[memo.id] = memo;
    var mbr = this._memos_by_record[memo.rid] = this._memos_by_record[memo.rid] || [];
    mbr.push(memo);

    var ex_record = this._records_by_id[ memo.rid ];
    console.log( 'slab[' + this.id + '].putMemo', memo.id, memo.rid, ex_record, Object.getOwnPropertyNames(this._records_by_id) );
    if(ex_record) ex_record.addMemos([memo]);

    if (this.tail) {
        // link previous tail to the new tail memo
        this.tail._newer = memo;
        memo._older = this.tail;
    } else {
        // we're first in -- yay
        this.head = memo;
    }

    // add new entry to the end of the linked list -- it's now the freshest memo.
    this.tail = memo;
    if (this.size === this.limit) {
        this.evictMemos();
    } else {
        // increase the size counter
        this.size++;
    }

    /*
     * TODO: handle initial object peering setup more intelligently than a self reference
     * For the time being, this is necessary, otherwise peering updates from other nodes will fall on deaf ears
    */
    this.registerMemoPeering( memo, memo.id, this.id, 2 );

    this.checkMemoReplicationFactor( memo, cb );

};


Slab.prototype.evictMemos = function() {
    var memo = this.head,
        ct    = this.size - this.quota
    ;

    if(ct <= 0) return;

    /* Evict enough memos to get back to the quota */
    while( memo && ct-- ){
        this.evictMemo( memo );
        memo = memo._newer;
    }

};

/* Time for this memo to go. Lets make sure there are enough copies elsewhere first */
Slab.prototype.evictMemo = function(memo){
    var me = this;
    if( typeof memo == 'string' ) memo = this._idmap[ memo ];
    if( !memo ) throw 'attempted to evict invalid memo';

    memo.evicting(true);
    console.log( 'Evicting memo', memo.id, 'from slab', me.id );

    this.checkMemoReplicationFactor( memo, function( success ){
        if( success ){
            me.killMemo(memo);
            console.log( 'Successfully evicted memo', memo.id );
        }else{
            console.log( 'Failed to evict memo', memo.id );
        }
    });
}


/* Remove memo from slab without delay */
Slab.prototype.killMemo = function(memo) {
    if( typeof memo == 'string' ) memo = this._idmap[ memo ];
    if( !memo || !this._idmap[memo.id] ){
        console.error('invalid memo');
        return;
    }

    delete this._idmap[memo.id]; // need to do delete unfortunately

    if (memo._newer && memo._older) {
        // relink the older entry with the newer entry
        memo._older._newer = memo._newer;
        memo._newer._older = memo._older;

    } else if (memo._newer) {

        // remove the link to us
        memo._newer._older = undefined;
        // link the newer entry to head
        this.head = memo._newer;

    } else if (memo._older) {

        // remove the link to us
        memo._older._newer = undefined;
        // link the newer entry to head
        this.tail = memo._older;

    } else { // if(memo._older === undefined && memo._newer === undefined) {
        this.head = this.tail = undefined;

    }

    // Tell the record to remove its reference to the memo
    // Should the slab manage the record/memo lookup?

    // TODO - find a way to tell if Records are still being used
    // TODO - don't purge head memos from the slab for records which are still being used

    var record = me._records_by_id[ memo.rid ];
    if(record) record.killMemo( memo );

    var mbr = this._memos_by_record[memo.rid];
    var index = mbr.indexOf(memo);
    mbr.splice(index, 1);

    this.size--;
    this.deregisterPeeringForMemo(memo);
};

/*
 * checkMemoReplicationFactor - Ensure that this memo is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this memo
 *       Verify that eviction from one slab ensures proper replica count on other slabs before memo is killed
*/
Slab.prototype.checkMemoReplicationFactor = function(memo,cb){
    var me      = this,
        desired = memo.desiredReplicas();

    console.log('slab[' + me.id + '].checkMemoReplicationFactor',memo.id);
    if (desired <= 0) return;

    var slabs     = this.mesh.getAcceptingSlabs( this.id, desired );

    /*
     * TODO:
     * Update to be async. Handle the possibility of push failure
     *
    */
    var peers = this.getMemoPeers(memo.id,true);
    //console.log(this.id,'peers',peers);

    var pending = 0;

    slabs.forEach(function(slab){
        if( peers.indexOf(slab) == -1 ){

            pending++;
            me.mesh.pushMemoToSlab( me, slab, memo, function( success ){
                pending--;

                if(success){
                    desired--;
                }

                /*
                 * TODO:
                 * Stop assuming pushMemoToSlab has a setTimeout or some delay.
                 * Will short circuit prematurely if this callback gets called synchronously
                 */

                if( pending <= 0 ){
                    cb( desired <= 0 );

                    if( desired > 0 ) console.error( "unable to achieve required replica count" );

                    /*
                     * TODO:
                     * Add insufficiently replicated memos to a remediation list, and set a timer to retry them
                    */
                }

            });
        }
    });


};

Slab.prototype.quotaRemaining = function() {
   return this.quota - this.size;
}

Slab.prototype.limitRemaining = function() {
   return this.limit - this.size;
}


/*
 * Create a totally new memo and attempt to replicate it to other slabs
 * Memo IDs must be globally unique, and consist of:
 *    eight digit base36 node id ( enumerated by central authority )
 *    two digit base36 slab id   ( enumerated by node )
 *    N digit base36 memo id    ( enumerated by slab )
 *
 * Discuss: How to prevent a malicious node/slab from originating a non-authorized memo id?
 * Is there any difference between that vs propagating an authorized edit to a pre-existing memo id?
 *
*/

Slab.prototype.genChildID = function(vals,cb) {
    return this.id + (this.child_increment++).toString(36);
}

/* getMemo - Retrieve memo from this slab by id
 * Update LRU cache accordingly
*/

Slab.prototype.getMemo = function(id){

    // First, find our cache entry
    var memo = this._idmap[id];
    if (memo === undefined) return; // Not cached. Sorry.

    // As <key> was found in the cache, register it as being requested recently
    if (memo === this.tail) {
        // Already the most recenlty used entry, so no need to update the list
        return memo;
    }

    // HEAD--------------TAIL
    //   <.older   .newer>
    //  <--- add direction --
    //   A  B  C  <D>  E
    if (memo._newer) {
        if ( memo === this.head ) this.head = memo._newer;
        memo._newer._older = memo._older; // C <-- E.
    }

    if (memo._older) memo._older._newer = memo._newer; // C. --> E
    memo._newer = undefined; // D --x
    memo._older = this.tail; // D. --> E

    if (this.tail) this.tail._newer = memo; // E. <-- D
    this.tail = memo;

    return memo;

};

Slab.prototype.getRecord = function(rid, cb){
    var me = this;
    var memos = me._memos_by_record[ rid ];

    if (typeof memos === 'undefined' || !memos.length){
        cb(null);
        return;
    }
    // TODO - perform an index lookup

    var record = record_cls.reconstitute( me, rid, memos );
    me._records_by_id[ rid ] = record;
    cb( record );
    return;
}

Slab.prototype.addRecord = function(record){
    this._records_by_id[record.id] = record;
}

Slab.prototype.dumpMemoIds = function(){
    var ids = [];
    var memo = this.tail;
    while(memo){
        ids.push(memo.id);
        memo = memo._older;
    }
    return ids;
}

module.exports = Slab;
