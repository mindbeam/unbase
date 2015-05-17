
/*
 * Mesh object handles all inter-slab communication
 * Currently set up only for single node testing
 * Will eventually handle network transport where slabs are on different nodes
 * 
*/

var record_cls = require('./record');

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

Mesh.prototype.pushRecordToPeer = function( slab, peer, record ) {
    
    console.log('Pushing record', record.id, 'from slab', slab.id, 'to peer', peer.id);
    
    /* JSON clone to ensure wire safety */
    // console.log( record.packetize );
    var serialized = JSON.stringify( record.packetize() );
    
    var cloned_packet = JSON.parse( serialized );
    
    if( typeof cloned_packet == 'object' ){
        /* Shouldn't need to filter replicas, as the putRecord will fail if we're trying to perform a duplicate put
         * This probably isn't very robust, but is useful for proof-of-concept stuffs
         * packet.replicas = packet.replicas.filter(function(id){ return id != peer.id });
        */
        
        cloned_packet.replicas.push(slab.id); // origin 
        var cloned_record = new record_cls( cloned_packet.id, cloned_packet.vals, cloned_packet.replicas );

        peer.putRecord( cloned_record );
        record.registerReplica( peer.id );
      
        //console.log(slab.id, '(origin) record  ', record.packetize());
        //console.log(peer.id, '(dest)   record  ',   cloned_record.packetize());
        
        /* 
         console.log('pushRecord completed for record id', cloned_record.id, 'replicas are:', cloned_packet.replicas );
         console.log('original record replicas are', record.r );
        */
    }
}

/* not super in love with the name of this */
Mesh.prototype.deregisterSlabRecord = function( slab, record ) {
    var me = this;
    
    record.getReplicas().forEach(function(id){
        var peer = me._slabs[id];
        if( peer ){
            var rv = peer.deregisterRecordPeer( record.id, slab.id );
            console.log('deregisterRecordPeer from', slab.id, record.id, 'to', peer.id, rv ? 'Succeeded' : 'Failed' );
        }
    });
    
}

Mesh.prototype.replicateRecordEdit = function(record,diff){
    var me   = this,
        reps = record.getReplicas();
    
    reps.forEach(function(id){
        var peer = me._slabs[id];
        if( peer ) peer.receiveRecordReplication( record.id, diff);
    });
}

// export the class
module.exports = Mesh;
