/**
 * Slab - A storage substrate for grains
 * Very loosely based on Rasmus Andersson's js-lru
 * A doubly linked list-based Least Recently Used (LRU) cache. Will keep most
 * recently used items while discarding least recently used items when its limit
 * is reached.
 * 
 */

var grain_cls = require('./grain');

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
  
  this.grain_increment = 0;
  this._idmap = {};

  this.size = 0;
  this.quota = args.quota || 5;
  this.limit = args.limit || 10;
  
  this.mesh = args.mesh;
  this.mesh.registerSlab( this );
  
}

/* Store a grain in this slab + manage LRU */
Slab.prototype.putGrain = function(grain) {
    if( ! grain instanceof grain_cls ) throw "invalid grain";
    if( this._idmap[grain.id] ) throw "attempt to put grain twice";
    
    this._idmap[grain.id] = grain;
    
    if (this.tail) {
        // link previous tail to the new tail grain
        this.tail._newer = grain;
        grain._older = this.tail;
    } else {
        // we're first in -- yay
        this.head = grain;
    }
    
    // add new entry to the end of the linked list -- it's now the freshest grain.
    this.tail = grain;
    if (this.size === this.limit) {
        this.evictGrains();
    } else {
        // increase the size counter
        this.size++;
    }
    
    console.log( 'Put grain', grain.id, 'to slab', this.id );
};


Slab.prototype.evictGrains = function() {
    var grain = this.head,
        ct    = this.size - this.quota
    ;
    
    if(ct <= 0) return;
    
    /* Evict enough grains to get back to the quota */
    while( grain && ct-- ){
        this.evictGrain( grain );
        grain = grain._newer;
    }
    
};

/* Time for this grain to go. Lets make sure there are enough copies elsewhere first */
Slab.prototype.evictGrain = function(grain){
    var me = this;
    if( !grain instanceof grain_cls ) grain = this._idmap[ grain ];
    if( !grain ) throw 'attempted to evict invalid grain';
    
    grain.evicting(true);
    console.log( 'Evicting grain', grain.id, 'from slab', me.id );
    this.pushGrain( grain, function( success ){
        if( success ){
            me.killGrain(grain);
            console.log( 'Successfully evicted grain', grain.id );
        }else{
            console.log( 'Failed to evict grain', grain.id );
        }
    });
}


/* Remove grain from slab without delay */
Slab.prototype.killGrain = function(grain) {
    if( !grain instanceof grain_cls ) grain = this._idmap[ grain ];
    if( !grain || !this._idmap[grain.id] ){
        console.error('invalid grain');
        return;
    }
    
    delete this._idmap[grain.id]; // need to do delete unfortunately
    
    if (grain._newer && grain._older) {
        // relink the older entry with the newer entry
        grain._older._newer = grain._newer;
        grain._newer._older = grain._older;
      
    } else if (grain._newer) {
        
        // remove the link to us
        grain._newer._older = undefined;
        // link the newer entry to head
        this.head = grain._newer;
      
    } else if (grain._older) {
        
        // remove the link to us
        grain._older._newer = undefined;
        // link the newer entry to head
        this.tail = grain._older;
      
    } else { // if(grain._older === undefined && grain._newer === undefined) {
        this.head = this.tail = undefined;
      
    }
    
    this.size--;
    this.mesh.deregisterSlabGrain(this,grain);
};

Slab.prototype.deregisterGrainPeer = function( grain_id, peer_id ) {
    var grain = this._idmap[grain_id];
    
    if( grain ){
        return grain.deregisterReplica( peer_id );
    }else{
        return false;
    }
}

/*
 * pushGrain - Ensure that this grain is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this grain
 *       Verify that eviction from one slab ensures proper replica count on other peers before grain is killed
*/
Slab.prototype.pushGrain = function(g,cb){
    var me      = this,
        desired = g.desiredReplicas();
        
    if (desired <= 0) return;
    var ap     = this.mesh.getAcceptingPeers( this.id, desired );

    /* TODO:
     * Update to be async
     * Handle the possibility of push failure
    */
    
    ap.forEach(function(peer){
        me.mesh.pushGrainToPeer( me, peer, g );
        desired--;
    });

    if( desired > 0 ) console.error( "unable to achieve required replica count" );
    if(cb) cb( desired <= 0 );
};

Slab.prototype.quotaRemaining = function() {
   return this.quota - this.size;
}

/*
 * Create a totally new grain and attempt to replicate it to other peers 
 * Grain IDs must be globally unique, and consist of:
 *    eight digit base36 node id ( enumerated by central authority )
 *    two digit base36 slab id   ( enumerated by node )
 *    N digit base36 grain id    ( enumerated by slab )
 *
 * Discuss: How to prevent a malicious node/slab from originating a non-authorized grain id?
 * Is there any difference between that vs propagating an authorized edit to a pre-existing grain id?
 * 
*/

Slab.prototype.newGrain = function(vals,cb) {
    var me = this,
        id = this.id + '-' + (this.grain_increment++).toString(36),
        g = new grain_cls(id,vals)
    ;

    this.putGrain(g);
    this.pushGrain( g, cb );

    return g;
}

/* getGrain - Retrieve grain from this slab by id
 * Update LRU cache accordingly
*/

Slab.prototype.getGrain = function(id){
    
    // First, find our cache entry
    var grain = this._idmap[id];
    if (grain === undefined) return; // Not cached. Sorry.
    
    // As <key> was found in the cache, register it as being requested recently
    if (grain === this.tail) {
        // Already the most recenlty used entry, so no need to update the list
        return grain;
    }
    
    // HEAD--------------TAIL
    //   <.older   .newer>
    //  <--- add direction --
    //   A  B  C  <D>  E
    if (grain._newer) {
        if ( grain === this.head ) this.head = grain._newer;
        grain._newer._older = grain._older; // C <-- E.
    }
    
    if (grain._older) grain._older._newer = grain._newer; // C. --> E
    grain._newer = undefined; // D --x
    grain._older = this.tail; // D. --> E
    
    if (this.tail) this.tail._newer = grain; // E. <-- D
    this.tail = grain;
    
    return grain;

};

Slab.prototype.dumpGrainIds = function(){
    var ids = [];
    var grain = this.tail;
    while(grain){
        ids.push(grain.id);
        grain = grain._older;
    }
    return ids;
}

module.exports = Slab;
