

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
 * There are two types of peers: replica, and observer
 * 
 * A replica is an actual copy of the object in question.
 * An observer contains no object data, but knows where the replicas are.
 * Both types participate in the peering, such that all peers are kept up to date with registrations and deregistrations.
 * 
 */
function Peerable() {}

Peerable.prototype.registerPeer = function(ref, peer, isReplica) {    
    var list;
    
    if(isReplica){
        list = this.r[ref] = this.r[ref] || [];
    }else{
        list = this.l[ref] = this.l[ref] || [];
    }
    
    list.push(peer);
    
};

Peerable.prototype.deregisterPeer = function(ref, peer) {
    var list,
        found = false;

    this.r[ref] = (this.r[ref] || []).filter(function(id){
        if( id == peer ){
            found = true;
            return false;
        }
        return true;
    });
    this.l[ref] = (this.l[ref] || []).filter(function(id){
        if( id == peer ){
            found = true;
            return false;
        }
        return true;
    });
    
    return found;
}

Record.prototype.getPeers = function(ref,isReplica){
    
    if (typeof isReplica == 'undefined'){
        return [].concat( this.r[ref] || [], this.l[ref] || [] );
    }else if( isReplica ){
        return this.r[ref];
    }else{
        return this.l[ref];
    }
    
}

/*
    Record.prototype._evicting    = 0;
    Record.prototype.__replica_ct = 1;
    
    // should we un-set this if an eviction fails?
    Record.prototype.evicting = function(v) {
        this._evicting = v ? 1 : 0;
    };
    
    Record.prototype.desiredReplicas = function() {
       return Math.max(0,(this.__replica_ct - this.r.length) + this._evicting);
    };
*/

Peerable.prototype.isPeerable = true;

// mixin - augment the target object with the Peerable functions
Peerable.mixin = function(destObject){
    ['registerPeer','deregisterPeer','getPeers','isPeerable'].forEach(function(property) {
        destObject.prototype[property] = Peerable.prototype[property];
    });
};


module.exports = Peerable;