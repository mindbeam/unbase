/**
 * Slab - A storage substrate for records
 * Very loosely based on Rasmus Andersson's js-lru
 * A doubly linked list-based Least Recently Used (LRU) cache. Will keep most
 * recently used items while discarding least recently used items when its limit
 * is reached.
 * 
 */

var record_cls = require('./record');

var slab_increment = 0;

function Slab(args) {

  if(slab_increment > 1296)                 throw "cannot create more than 1296 slabs";
  if(typeof args != 'object')               throw "must provide args";
  if(!args.node || !args.node.length == 8)  throw "must provide 8 digit node id";
  if(!args.mesh)                            throw "must provide mesh object";

  // encode and zerofill the slab id
  this.id = args.node + ( "00" + (++slab_increment).toString(36)).substr(-2,2);
  console.log('Initialized Slab', this.id);
  //if(this.id.length != 10) throw "sanity error " + this.id;
  
  // Temporarily assuming nods ids are one character for simplicity
  if(this.id.length != 3) throw "sanity error " + this.id;
  
  this.record_increment = 0;
  this._idmap = {};

  this.size = 0;
  this.quota = args.quota || 5;
  this.limit = args.limit || 10;
  
  this.mesh = args.mesh;
  this.mesh.registerSlab( this );
  
}

/* Store a record in this slab + manage LRU */
Slab.prototype.putRecord = function(record) {
    if( ! record instanceof record_cls ) throw "invalid record";
    if( this._idmap[record.id] ) throw "attempt to put record twice";
    
    this._idmap[record.id] = record;
    
    if (this.tail) {
        // link previous tail to the new tail record
        this.tail._newer = record;
        record._older = this.tail;
    } else {
        // we're first in -- yay
        this.head = record;
    }
    
    // add new entry to the end of the linked list -- it's now the freshest record.
    this.tail = record;
    if (this.size === this.limit) {
        this.evictRecords();
    } else {
        // increase the size counter
        this.size++;
    }
    
    console.log( 'Put record', record.id, 'to slab', this.id );
};


Slab.prototype.evictRecords = function() {
    var record = this.head,
        ct    = this.size - this.quota
    ;
    
    if(ct <= 0) return;
    
    /* Evict enough records to get back to the quota */
    while( record && ct-- ){
        this.evictRecord( record );
        record = record._newer;
    }
    
};

/* Time for this record to go. Lets make sure there are enough copies elsewhere first */
Slab.prototype.evictRecord = function(record){
    var me = this;
    if( !record instanceof record_cls ) record = this._idmap[ record ];
    if( !record ) throw 'attempted to evict invalid record';
    
    record.evicting(true);
    console.log( 'Evicting record', record.id, 'from slab', me.id );
    this.pushRecord( record, function( success ){
        if( success ){
            me.killRecord(record);
            console.log( 'Successfully evicted record', record.id );
        }else{
            console.log( 'Failed to evict record', record.id );
        }
    });
}


/* Remove record from slab without delay */
Slab.prototype.killRecord = function(record) {
    if( !record instanceof record_cls ) record = this._idmap[ record ];
    if( !record || !this._idmap[record.id] ){
        console.error('invalid record');
        return;
    }
    
    delete this._idmap[record.id]; // need to do delete unfortunately
    
    if (record._newer && record._older) {
        // relink the older entry with the newer entry
        record._older._newer = record._newer;
        record._newer._older = record._older;
      
    } else if (record._newer) {
        
        // remove the link to us
        record._newer._older = undefined;
        // link the newer entry to head
        this.head = record._newer;
      
    } else if (record._older) {
        
        // remove the link to us
        record._older._newer = undefined;
        // link the newer entry to head
        this.tail = record._older;
      
    } else { // if(record._older === undefined && record._newer === undefined) {
        this.head = this.tail = undefined;
      
    }
    
    this.size--;
    this.mesh.deregisterSlabRecord(this,record);
};

Slab.prototype.deregisterRecordPeer = function( record_id, peer_id ) {
    var record = this._idmap[record_id];
    
    if( record ){
        return record.deregisterReplica( peer_id );
    }else{
        return false;
    }
}

/*
 * pushRecord - Ensure that this record is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this record
 *       Verify that eviction from one slab ensures proper replica count on other peers before record is killed
*/
Slab.prototype.pushRecord = function(g,cb){
    var me      = this,
        desired = g.desiredReplicas();
        
    if (desired <= 0) return;
    var ap     = this.mesh.getAcceptingPeers( this.id, desired );

    /* TODO:
     * Update to be async
     * Handle the possibility of push failure
    */
    
    ap.forEach(function(peer){
        me.mesh.pushRecordToPeer( me, peer, g );
        desired--;
    });

    if( desired > 0 ) console.error( "unable to achieve required replica count" );
    if(cb) cb( desired <= 0 );
};

Slab.prototype.quotaRemaining = function() {
   return this.quota - this.size;
}

/*
 * Create a totally new record and attempt to replicate it to other peers 
 * Record IDs must be globally unique, and consist of:
 *    eight digit base36 node id ( enumerated by central authority )
 *    two digit base36 slab id   ( enumerated by node )
 *    N digit base36 record id    ( enumerated by slab )
 *
 * Discuss: How to prevent a malicious node/slab from originating a non-authorized record id?
 * Is there any difference between that vs propagating an authorized edit to a pre-existing record id?
 * 
*/

Slab.prototype.newRecord = function(vals,cb) {
    var me = this,
        id = this.id + '-' + (this.record_increment++).toString(36),
        g = new record_cls(id,vals)
    ;

    this.putRecord(g);
    this.pushRecord( g, cb );

    return g;
}

/* getRecord - Retrieve record from this slab by id
 * Update LRU cache accordingly
*/

Slab.prototype.getRecord = function(id){
    
    // First, find our cache entry
    var record = this._idmap[id];
    if (record === undefined) return; // Not cached. Sorry.
    
    // As <key> was found in the cache, register it as being requested recently
    if (record === this.tail) {
        // Already the most recenlty used entry, so no need to update the list
        return record;
    }
    
    // HEAD--------------TAIL
    //   <.older   .newer>
    //  <--- add direction --
    //   A  B  C  <D>  E
    if (record._newer) {
        if ( record === this.head ) this.head = record._newer;
        record._newer._older = record._older; // C <-- E.
    }
    
    if (record._older) record._older._newer = record._newer; // C. --> E
    record._newer = undefined; // D --x
    record._older = this.tail; // D. --> E
    
    if (this.tail) this.tail._newer = record; // E. <-- D
    this.tail = record;
    
    return record;

};

Slab.prototype.editRecord = function(record,vals){
    if(!record instanceof record_cls) throw "invalid record";
    var diff = {}, val;
    
    Object.keys(vals).forEach(function(key){
        val = vals[key];
        if( key.charAt(0) == '$' ){
            if( val instanceof Record ){
                val = val.id;
            }else{
                // TODO validate record id
            }
        }
        
        if(record.v[key] != val){
            diff[key] = val;
            record.v[key] = val; // apply to local record
        }
    });
    
    
    /* TODO IMPLEMENT MVCC - NBD */
    
    this.mesh.replicateRecordEdit(record,diff);
}

Slab.prototype.receiveRecordReplication = function( record_id, diff ){
    var record = this._idmap[record_id];
    if( record ){
        Object.keys(diff).forEach(function(key){
            record.v[key] = diff[key];
        });
        console.log('Slab', this.id, 'receiveRecordReplication', record.id, diff );
    }
}

Slab.prototype.dumpRecordIds = function(){
    var ids = [];
    var record = this.tail;
    while(record){
        ids.push(record.id);
        record = record._older;
    }
    return ids;
}

module.exports = Slab;
