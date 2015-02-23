
/*
 * Mesh object handles all inter-slab communication
 * Currently set up only for single node testing
 * Will eventually handle network transport where slabs are on different nodes
 * 
*/

var grain_cls = require('./grain');

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

Mesh.prototype.pushGrainToPeer = function( slab, peer, grain ) {
    
    console.log('Pushing grain', grain.id, 'from slab', slab.id, 'to peer', peer.id);
    
    /* JSON clone to ensure wire safety */
    // console.log( grain.packetize );
    var serialized = JSON.stringify( grain.packetize() );
    
    var cloned_packet = JSON.parse( serialized );
    
    if( typeof cloned_packet == 'object' ){
        /* Shouldn't need to filter replicas, as the putGrain will fail if we're trying to perform a duplicate put
         * This probably isn't very robust, but is useful for proof-of-concept stuffs
         * packet.replicas = packet.replicas.filter(function(id){ return id != peer.id });
        */
        
        cloned_packet.replicas.push(slab.id); // origin 
        var cloned_grain = new grain_cls( cloned_packet.id, cloned_packet.vals, cloned_packet.replicas );
        
        peer.putGrain( cloned_grain );
        grain.registerReplica( peer.id );
        
        /* 
         console.log('pushGrain completed for grain id', cloned_grain.id, 'replicas are:', cloned_packet.replicas );
         console.log('original grain replicas are', grain.r );
        */
    }
}

/* not super in love with the name of this */
Mesh.prototype.deregisterSlabGrain = function( slab, grain ) {
    var me = this;
    
    grain.getReplicas().forEach(function(id){
        var peer = me._slabs[id];
        if( peer ){
            var rv = peer.deregisterGrainPeer( grain.id, slab.id );
            console.log('deregisterGrainPeer', grain.id, slab.id, rv ? 'Succeeded' : 'Failed' );
        }
    });
    
}

// export the class
module.exports = Mesh;
