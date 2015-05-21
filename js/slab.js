/**
 * Slab - A storage substrate for items/memos
 * Very loosely based on Rasmus Andersson's js-lru
 * A doubly linked list-based Least Recently Used (LRU) cache. Will keep most
 * recently used items while discarding least recently used items when its limit
 * is reached.
 * 
 */


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
  
    this.local_peerings = {};
    this.ref_peerings   = {};
    
}

Slab.prototype.registerItemPeering = function(item, ref_item_id, remote_slab_id, peer_flag, skip ){
    console.log('slab.registerItemPeering', item.id, ref_item_id, remote_slab_id, peer_flag, skip );
    var peerings = {};
    peerings[ref_item_id] = {};
    peerings[ref_item_id][remote_slab_id] = peer_flag;
    this.registerItemPeerings(item, peerings, skip);
    
}
Slab.prototype.registerItemPeerings = function(item, peerings, skip ){
    var me      = this,
        changes = {};
        console.log('slab[' + me.id + '].registerItemPeerings', item.id, peerings, skip);

    // Need to know what items are referencing what, so we can de-peer from those items if all references are removed
    // probably can't just ask the item for its references, because changes would fail to trigger the necessary de-peering
    // need to detect when the reference changes, so we can de-peer the old reference, or reduce the reference count at least.
    var refs = me.local_peerings[item.id] = me.local_peerings[item.id] || [];
    Object.keys(peerings).forEach(function(ref_item_id){
        
        if(refs.indexOf(ref_item_id) == -1) refs.push(ref_item_id);
        
        // lookup table so that remotes can be updated
        // includes explicit list of local items so we can remove them, and determine when the
        // reference count hits zero, so we know when it's appropriate to de-peer
        var r   = me.ref_peerings[ref_item_id] = me.ref_peerings[ref_item_id] || { items: [], remotes: {} };
        if( r.items.indexOf(item.id) == -1 ) r.items.push(item.id); // increase the count
        
        Object.keys(peerings[ref_item_id]).forEach(function(remote_slab_id){
            peer_flag = peerings[ref_item_id][remote_slab_id];
            
            if( (remote_slab_id != me.id) && !r.remotes[remote_slab_id] ){
                r.remotes[remote_slab_id] = peer_flag; // 2 means we actually have it, 1 means peering only
                
                var change = changes[remote_slab_id] = changes[remote_slab_id] || {};
                change[item.id] = peer_flag;
            }
        });
    });
    
    if(!skip ){
        if( Object.keys(changes).length ) me.mesh.sendPeeringChanges(me.id,changes);
    }
}

Slab.prototype.receivePeeringChange = function(sending_slab_id,change){
    var me = this;
    console.log('slab[' + me.id + '].receivePeeringChange', sending_slab_id, change );
    Object.keys(change).forEach(function(item_id){
        var flag = change[item_id];
        
        if( me.ref_peerings[item_id] ){
            if( flag ){
                me.ref_peerings[item_id].remotes[sending_slab_id] = flag;
            }else{
                delete me.ref_peerings[item_id].remotes[sending_slab_id];
            }
        }
    });
}

Slab.prototype.deregisterItemPeering = function(item){
    var me      = this,
        changes = {};
    
    console.log('slab[' + me.id + '].deregisterItemPeering', item.id );
    var refs = me.local_peerings[item.id] || [];
    
    refs.forEach(function(ref_item_id){
        var r   = me.ref_peerings[ref_item_id];
        r.items = r.items.filter(function(id){ return id != item.id });
        console.log('meow',ref_item_id, r);
        // TODO this isn't right
        
        if(r.items.length == 0){
            Object.keys(r.remotes).forEach(function(remote_slab_id){
                var change = changes[remote_slab_id] = changes[remote_slab_id] || {};
                change[item.id] = 0; // we no have
            });
            delete me.ref_peerings[ref_item_id];
        }
    });
    
    delete me.local_peerings[item.id];
    me.mesh.sendPeeringChanges(me.id,changes);
}

Slab.prototype.getPeeringsForItem = function(item, include_self){
    var me      = this,
        peerings = {};
    
    var refs = this.local_peerings[item.id] || [];
    
    refs.forEach(function(ref_item_id){
        var remotes = me.ref_peerings[ref_item_id].remotes;
        var peering = {};
        Object.keys(remotes).forEach(function(key) {
            peering[key] = remotes[key];
        });
        if(include_self) peering[ me.id ] = 2;
        
        peerings[ ref_item_id ] = peering;
    });
    
    return peerings;
}

Slab.prototype.getPeers = function(item_id,has_item){
    var r   = this.ref_peerings[item_id];
    if(!r) return null;
    
    var remotes = [];
    
    if(has_item){
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

/* Store a item in this slab + manage LRU */
Slab.prototype.putItem = function(item,cb) {
    //if( ! item instanceof item_cls ) throw "invalid item";
    if( this._idmap[item.id] ) throw "attempt to put item twice";
    
    console.log( 'slab[' + this.id + '].putItem', item.id, 'to slab', this.id );
    
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
    
    // TODO: handle initial object peering setup more intelligently than a self reference
    // For the time being, this is necessary, otherwise peering updates from other nodes will fall on deaf ears
    this.registerItemPeering( item, item.id, this.id, 2 );
    
    this.pushItem( item, cb );
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
    if( typeof item == 'string' ) item = this._idmap[ item ];
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
    if( typeof item == 'string' ) item = this._idmap[ item ];
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
    this.deregisterItemPeering(item);
};

/*
 * pushItem - Ensure that this item is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this item
 *       Verify that eviction from one slab ensures proper replica count on other slabs before item is killed
*/
Slab.prototype.pushItem = function(item,cb){
    var me      = this,
        desired = item.desiredReplicas();
    
    console.log('slab[' + me.id + '].pushItem',item.id);
    if (desired <= 0) return;
    var ap     = this.mesh.getAcceptingSlabs( this.id, desired );

    /* TODO:
     * Update to be async
     * Handle the possibility of push failure
    */
    var peers = this.getPeers(item.id,true);
    //console.log(this.id,'peers',peers);
    
    ap.forEach(function(slab){
        if( peers.indexOf(slab) == -1 ){
            me.mesh.pushItemToSlab( me, slab, item );
            desired--;
        }
    });

    if( desired > 0 ) console.error( "unable to achieve required replica count" );
    if(cb) cb( desired <= 0 );
};

Slab.prototype.quotaRemaining = function() {
   return this.quota - this.size;
}


/*
 * Create a totally new item and attempt to replicate it to other slabs 
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

/*
 * Edits to be handled directly via the record object
 * 
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
*/

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
