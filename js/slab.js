/**
 * Slab - A storage substrate for items/memos
 *
 * Purpose:
 *    Stores memos
 *    Manages memo peering
 *
 * The storage portion is very loosely based on Rasmus Andersson's js-lru
 * A doubly linked list-based Least Recently Used (LRU) cache. Will keep most
 * recently used items while evicting least recently used items when its limit
 * is reached.
 * 
 */

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
  
    this.size = 0;
    this.quota = args.quota || 5;
    this.limit = args.limit || 10;
    
    this.mesh = args.mesh;
    this.mesh.registerSlab( this );
    
    this.local_peerings = {};
    this.ref_peerings   = {};
    
}


/*
 * convenience method for registering a peering between an item, and a referenced item.
 */

Slab.prototype.registerItemPeering = function(item, ref_item_id, remote_slab_id, peer_state, silent ){
    
    silent = silent || false;
    console.log('slab[' + this.id + '].registerItemPeering', "Item:", item.id, "Ref Item:", ref_item_id, "Remote Slab:", remote_slab_id, "Peering State:", peer_state, "Silent:", silent );
    
    var peerings = {};
    peerings[ref_item_id] = {};
    peerings[ref_item_id][remote_slab_id] = peer_state;
    
    this.updateItemPeerings(item, peerings, silent);
    
}

/*
 * updateItemPeerings - method for updating peering data for a given item
 * 
 * item     - the item object
 * peerings - struct of peerings which we are updating { ref_item_id : { remote_slab_id: peer_state }, .. }
 * silent   - boolean fflag to indicate that we should abstain from notifying the mesh
 *
 * Peering Definition:
 *     A "peering" is a loose collection of items which are interested in each other's contact information.
 *     Participants in a peering may or may not even have a copy of the item in question.
 *
 * Peering states:
 *     0 - Non participatory
 *     1 - Participatory, but does not have a copy of the referenced item
 *     2 - Participatory, AND has a copy of the referenced item
 *
 * By default, an instance of an item peers with other instances of same, across slabs in which it is resident.
 * Upon (final) eviction of an item from a slab, an update containing a negative peering state is pushed to other participants in the peering. 
 * 
 */
Slab.prototype.updateItemPeerings = function( item, peerings, silent ){
    var me      = this,
        changes = {},
        peer_state
    ;
    
    console.log('slab[' + me.id + '].updateItemPeerings', item.id, peerings, silent);

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
            peer_state = peerings[ref_item_id][remote_slab_id];
            
            if( (remote_slab_id != me.id) && !r.remotes[remote_slab_id] ){
                r.remotes[remote_slab_id] = peer_state; // 2 means we actually have it, 1 means peering only
                
                var change = changes[remote_slab_id] = changes[remote_slab_id] || {};
                change[item.id] = peer_state;
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
    
    Object.keys(change).forEach(function(item_id){
        peer_state = change[item_id];
        
        if( me.ref_peerings[item_id] ){
            if( peer_state ){
                me.ref_peerings[item_id].remotes[sending_slab_id] = peer_state;
            }else{
                delete me.ref_peerings[item_id].remotes[sending_slab_id];
            }
        }
    });
}

Slab.prototype.deregisterPeeringForItem = function(item){
    var me      = this,
        changes = {};
    
    console.log('slab[' + me.id + '].deregisterPeeringForItem', item.id );
    var refs = me.local_peerings[item.id] || [];
    
    refs.forEach(function(ref_item_id){
        var r   = me.ref_peerings[ref_item_id];
        r.items = r.items.filter(function(id){ return id != item.id });
        // console.log('meow',ref_item_id, r);
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

/*
 * Look up the current peering participants for a given item id
 * 
 */

Slab.prototype.getItemPeers = function(item_id,has_item){
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

/* Store a item in this slab, manage LRU */
Slab.prototype.putItem = function(item,cb) {
    //if( ! item instanceof item_cls ) throw "invalid item";
    if( this._idmap[item.id] ) throw "attempt to put item twice";
    
    console.log( 'slab[' + this.id + '].putItem', item.id );
    
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
        this.evictItems();
    } else {
        // increase the size counter
        this.size++;
    }
    
    /*
     * TODO: handle initial object peering setup more intelligently than a self reference
     * For the time being, this is necessary, otherwise peering updates from other nodes will fall on deaf ears
    */
    this.registerItemPeering( item, item.id, this.id, 2 );
    
    this.checkItemReplicationFactor( item, cb );
    
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
    
    this.checkItemReplicationFactor( item, function( success ){
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
    this.deregisterPeeringForItem(item);
};

/*
 * checkItemReplicationFactor - Ensure that this item is sufficiently replicated
 * TODO: Ensure that we don't attempt to push to a replica that already has this item
 *       Verify that eviction from one slab ensures proper replica count on other slabs before item is killed
*/
Slab.prototype.checkItemReplicationFactor = function(item,cb){
    var me      = this,
        desired = item.desiredReplicas();
    
    console.log('slab[' + me.id + '].checkItemReplicationFactor',item.id);
    if (desired <= 0) return;
    
    var slabs     = this.mesh.getAcceptingSlabs( this.id, desired );

    /*
     * TODO:
     * Update to be async. Handle the possibility of push failure
     * 
    */
    var peers = this.getItemPeers(item.id,true);
    //console.log(this.id,'peers',peers);
    
    var pending = 0;
    
    slabs.forEach(function(slab){
        if( peers.indexOf(slab) == -1 ){
            
            pending++;
            me.mesh.pushItemToSlab( me, slab, item, function( success ){
                pending--;
                
                if(success){
                    desired--;
                }
                
                /*
                 * TODO:
                 * Stop assuming pushItemToSlab has a setTimeout or some delay.
                 * Will short circuit prematurely if this callback gets called synchronously
                 */
                
                if( pending <= 0 ){
                    cb( desired <= 0 );
                    
                    if( desired > 0 ) console.error( "unable to achieve required replica count" );
                    
                    /*
                     * TODO:
                     * Add insufficiently replicated items to a remediation list, and set a timer to retry them
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
    return this.id + (this.child_increment++).toString(36);
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
