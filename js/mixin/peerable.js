

/*
 * Peerable is a mixin which adds peering behavior to a module.
 * Peering is necessary to locate and account for copies of objects in the network.
 * 
 * A Peerable object manages a series of "reference" lists, each of which corresponds to a specific identifier.
 * Each of these lists seeks to participate in a "peering", which is to say that the list contains a list of remote peers, who in turn reference the local copy.
 *
 * The reference "self" is reserved for other copies of the local Peerable object itself.
 * Other references, such as "relationship1", "relationship2", are used to track other related objects.
 *
 * a "peering" always pertains to an object ID
 * Members of a peering are not limited to copies of the referenced object itself
 *
 * 
 * There are two types of peers: replica, and observer
 * 
 * A replica is an actual copy of the object in question.
 * An observer contains no object data, but knows where the replicas are.
 * Both types participate in the peering, such that all peers are kept up to date with registrations and deregistrations.
 *
 * Slab A:
 *   Item 1 [self]           - Peers: B
 *    \___  [foo ]->Item 2   - Peers: B, C
 *    
 * Slab B:
 *   Item 1 [self]          - Peers: A
 *    \___  [foo ]->Item 2  - Peers: A, C
 *   Item 2 [self]          - Peers: A, C
 *
 * Slab C:
 *   Item 2 [self]          - Peers: A,B
 *   
 */
function Peerable() {}


Peerable.prototype.registerPeer = function(id, slab_id) {
    if(id == this.id) id = 'self';
    
    var list = this.p[id] = this.p[id] || [];
    if(list.indexOf(slab_id) == -1) list.push(slab_id);
};

Peerable.prototype.deregisterPeer = function(ref, peer) {
    var list,
        found = false;

    this.p[ref] = (this.p[ref] || []).filter(function(id){
        if( id == peer ){
            found = true;
            return false;
        }
        return true;
    });
    
    return found;
};

Peerable.prototype.getPeers = function(name){
    return [];
   // return (this.p[name] || []);
};

Peerable.prototype.getPeering = function(){
    var me      = this,
        p       = me.p
        peering = {};
    
    // return the peers we know about, plus self
    Object.getOwnPropertyNames(p).forEach(function (name) {
        peering[name] = [me.slab.id].concat(p[name]);
    });
    
    return peering;
};

// not sure if initPeering is needed
Peerable.prototype.initPeering = function(){
    this.p = { self: [] };
}

Peerable.prototype.setPeering = function(peering){
    peering = peering || {};

    var me = this,
        p  = this.p = { self: [] }
    ;
    
    Object.getOwnPropertyNames(peering).forEach(function (name) {
        p[name] = peering[name].filter(function(slab_id){
            return slab_id != me.slab.id;
        });
    });
};


// TODO:
Peerable.prototype.destroyPeering = function(){
    var c;
    
    var notofy_slabs = {};
    Object.getOwnPropertyNames(this.p).forEach(function (name) {
        this.p[name].forEach(function(slab_id){
            c = slabs[ slab_id ] = slabs[ slab_id ] || {};
            c[name] = 0;
        });
    });
    
    var peeringchange = { id: this.id, slab: this.slab.id, notify_slabs: notify_slabs };
    this.slab.mesh.sendPeeringChange( peeringchange );
}

Peerable.prototype.isPeerable = true;

// mixin - augment the target object with the Peerable functions
Peerable.mixin = function(destObject){
    ['registerPeer','deregisterPeer','getPeers','initPeering','getPeering','setPeering','isPeerable'].forEach(function(property) {
        destObject.prototype[property] = Peerable.prototype[property];
    });
};


module.exports = Peerable;