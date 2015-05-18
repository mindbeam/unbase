/**
 * Slab - A storage substrate for items/memos
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
  
  this.child_increment = 0;
  this._idmap = {};

  this.size = 0;
  this.quota = args.quota || 5;
  this.limit = args.limit || 10;
  
  this.mesh = args.mesh;
  this.mesh.registerSlab( this );
  
}

/* Store a item in this slab + manage LRU */
Slab.prototype.putItem = function(item) {
    if( ! item instanceof item_cls ) throw "invalid item";
    if( this._idmap[item.id] ) throw "attempt to put item twice";
    
    this._idmap[item.id] = item;
    
    if (this.tail) {
        // link previous tail to the new tail item
        this.tail._newer = item;
        item._older = this.tail;
    } else {
        // we're first in -- yay
        this.head = item;
    }
    
    // add new entry to the end of the linked list -- it's now the freshest item.
    this.tail = item;
    if (this.size === this.limit) {
        this.evictRecords();
    } else {
        // increase the size counter
        this.size++;
    }
    
    console.log( 'Put item', item.id, 'to slab', this.id );
};


Slab.prototype.evictItems = function() {
    var item = this.head,
        ct    = this.size - this.quota
    ;
    
    if(ct <= 0) return;
    
    /* Evict enough items to get back to the quota */
    while( item && ct-- ){
        this.evictItem( item );
        item = item._newer;
    }
    
};

/* Time for this item to go. Lets make sure there are enough copies elsewhere first */
Slab.prototype.evictItem = function(item){
    var me = this;
    if( !item instanceof item_cls ) item = this._idmap[ item ];
    if( !item ) throw 'attempted to evict invalid item';
    
    item.evicting(true);
    console.log( 'Evicting item', item.id, 'from slab', me.id );
    this.pushItem( item, function( success ){
        if( success ){
            me.killItem(item);
            console.log( 'Successfully evicted item', item.id );
        }else{
            console.log( 'Failed to evict item', item.id );
        }
    });
}


/* Remove item from slab without delay */
Slab.prototype.killItem = function(item) {
    if( !item instanceof item_cls ) item = this._idmap[ item ];
    if( !item || !this._idmap[item.id] ){
        console.error('invalid item');
        return;
    }
    
    delete this._idmap[item.id]; // need to do delete unfortunately
    
    if (item._newer && item._older) {
        // relink the older entry with the newer entry
        item._older._newer = item._newer;
        item._newer._older = item._older;
      
    } else if (item._newer) {
        
        // remove the link to us
        item._newer._older = undefined;
        // link the newer entry to head
        this.head = item._newer;
      
    } else if (item._older) {
        
        // remove the link to us
        item._older._newer = undefined;
        // link the newer entry to head
        this.tail = item._older;
      
    } else { // if(item._older === undefined && item._newer === undefined) {
        this.head = this.tail = undefined;
      
    }
    
    this.size--;
    this.mesh.deregisterSlabItem(this,item);
};

Slab.prototype.deregisterItemPeer = function( item_id, peer_id ) {
    var item = this._idmap[item_id];
    
    if( item ){
        return item.deregisterReplica( peer_id );
    }else{
        return false;
    }
}

/*
 * pushItem - Ensure that this item is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this item
 *       Verify that eviction from one slab ensures proper replica count on other peers before item is killed
*/
Slab.prototype.pushItem = function(g,cb){
    var me      = this,
        desired = g.desiredReplicas();
        
    if (desired <= 0) return;
    var ap     = this.mesh.getAcceptingPeers( this.id, desired );

    /* TODO:
     * Update to be async
     * Handle the possibility of push failure
    */
    
    ap.forEach(function(peer){
        me.mesh.pushItemToPeer( me, peer, g );
        desired--;
    });

    if( desired > 0 ) console.error( "unable to achieve required replica count" );
    if(cb) cb( desired <= 0 );
};

Slab.prototype.quotaRemaining = function() {
   return this.quota - this.size;
}


/*
 * Create a totally new item and attempt to replicate it to other peers 
 * Item IDs must be globally unique, and consist of:
 *    eight digit base36 node id ( enumerated by central authority )
 *    two digit base36 slab id   ( enumerated by node )
 *    N digit base36 item id    ( enumerated by slab )
 *
 * Discuss: How to prevent a malicious node/slab from originating a non-authorized item id?
 * Is there any difference between that vs propagating an authorized edit to a pre-existing item id?
 * 
*/

Slab.prototype.genChildID = function(vals,cb) {
    return this.id + '-' + (this.child_increment++).toString(36);
}

/* getItem - Retrieve item from this slab by id
 * Update LRU cache accordingly
*/

Slab.prototype.getItem = function(id){
    
    // First, find our cache entry
    var item = this._idmap[id];
    if (item === undefined) return; // Not cached. Sorry.
    
    // As <key> was found in the cache, register it as being requested recently
    if (item === this.tail) {
        // Already the most recenlty used entry, so no need to update the list
        return item;
    }
    
    // HEAD--------------TAIL
    //   <.older   .newer>
    //  <--- add direction --
    //   A  B  C  <D>  E
    if (item._newer) {
        if ( item === this.head ) this.head = item._newer;
        item._newer._older = item._older; // C <-- E.
    }
    
    if (item._older) item._older._newer = item._newer; // C. --> E
    item._newer = undefined; // D --x
    item._older = this.tail; // D. --> E
    
    if (this.tail) this.tail._newer = item; // E. <-- D
    this.tail = item;
    
    return item;

};

Slab.prototype.editItem = function(item,vals){
    if(!item instanceof item_cls) throw "invalid item";
    var diff = {}, val;
    
    Object.keys(vals).forEach(function(key){
        val = vals[key];
        if( key.charAt(0) == '$' ){
            if( val instanceof Item ){
                val = val.id;
            }else{
                // TODO validate item id
            }
        }
        
        if(item.v[key] != val){
            diff[key] = val;
            item.v[key] = val; // apply to local item
        }
    });
    
    
    /* TODO IMPLEMENT MVCC - NBD */
    
    this.mesh.replicateItemEdit(item,diff);
}

Slab.prototype.receiveItemReplication = function( item_id, diff ){
    var item = this._idmap[item_id];
    if( item ){
        Object.keys(diff).forEach(function(key){
            item.v[key] = diff[key];
        });
        console.log('Slab', this.id, 'receiveItemReplication', item.id, diff );
    }
}

Slab.prototype.dumpItemIds = function(){
    var ids = [];
    var item = this.tail;
    while(item){
        ids.push(item.id);
        item = item._older;
    }
    return ids;
}

module.exports = Slab;
