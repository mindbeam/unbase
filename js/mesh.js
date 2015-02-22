
/*
 * Mesh object handles all inter-slab communication
 * Currently set up only for single node testing
 * Will eventually handle network transport where slabs are on different nodes
 * 
*/

function Mesh() {
    /* Only local slabs are supported as peers at this time */
    this._slabs = [];
}

// class methods
Mesh.prototype.registerSlab = function( slab ) {
    this._slabs[slab.id] = slab;
    console.log('Registered Slab ', slab.id);
};


/* TODO:
 * Update this to consider RTT, Health, Geo-redundancy and better handle storage quota.
 * Optimize to avoid looping over all known peers every time.
*/

Mesh.prototype.getAcceptingPeers = function( exclude_slab_id, number ) {
    number = (typeof number == 'number' && number > 0 ) ? number : 1;
    var slabs = this._slabs,
        slab,
        out   = []
    ;
    
    Object.keys(slabs).forEach(function(id){
        slab = slabs[id];
        if( exclude_slab_id == slab.id ) return;
        if( (slab.quotaRemaining() > 0) && number-- > 0 ) out.push(slab);
    });
    
    return out;

}


/*
 * Working terminology:
 *   slab = my slab
 *   peer = your slab
*/

Mesh.prototype.pushGrain = function( peer, grain ) {
    peer.putGrain( grain );
    grain.registerReplica( peer.id );
}

/* not super in love with the name of this */
Mesh.prototype.deregisterSlabGrain = function( slab, grain ) {
    
    grain.getReplicas().forEach(function(id){
        var peer = this._slabs[id];
        if( peer ){
            var rv = peer.deregisterGrainPeer( grain.id, slab.id );
            console.log('deregisterGrainPeer', grain.id, slab.id, rv ? 'Succeeded' : 'Failed' );
        }
    });
    
}

// export the class
module.exports = Mesh;
