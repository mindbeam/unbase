
/*
 * Mesh object handles all inter-slab communication
 * Currently set up only for single node testing
 * Will eventually handle network transport where slabs are on different nodes
 *
*/

var memo_cls = require('./memo');

function Mesh(params) {
    params = params || {};
    /* Only local slabs are supported as peers at this time */
    this._slabs = [];
    this.network_latency_ms = params.network_latency_ms || 100;
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

Mesh.prototype.getAcceptingSlabs = function( exclude_slab_id, number ) {
    number = (typeof number == 'number' && number > 0 ) ? number : 1;
    var slabs = this._slabs,
        slab,
        out   = []
    ;



    /* TODOs:
     *
     *    Perform selection on the basis of the diasporosity score of the memos you wish to replicate
     *    Shuffle the list to encourage diasporosity, and ensure that retries do not hit the same nodes
     */

    Object.keys(slabs).forEach(function(id){

        slab = slabs[id];
        if( exclude_slab_id == slab.id ) return;
        if( (slab.quotaRemaining() > 0) && number-- > 0 ) out.push(slab);

    });

    return out;

}


Mesh.prototype.pushMemoToSlab = function( from_slab, to_slab, memo, cb ) {

    console.log('mesh.pushMemoToSlab', memo.id, 'from slab', from_slab.id, 'to slab', to_slab.id);

    // Simulate network latency
    setTimeout(function(){

        /* Shouldn't need to filter replicas, as the putMemo will fail if we're trying to perform a duplicate put
         * This probably isn't very robust, but is useful for proof-of-concept stuffs
         * packet.replicas = packet.replicas.filter(function(id){ return id != to_slab.id });
        */

        // peering handoff is being handled via serialize/deserialize for the time being.
        // This seems weird, on account of the possibility for duplicates across multiple memos being pushed
        // think about:
        // including only ref info in the serialized object
        // with subsequent peering hints, which would then be applied to the registered refs

        if(to_slab.limitRemaining() <= 0){
            cb( false );
            return;
        }

        var serialized = JSON.stringify( memo.packetize() );
        console.log(serialized);
        var packet = JSON.parse( serialized );

        var cloned_memo = new memo_cls.depacketize( to_slab, packet );

        cb( true );

    }, this.network_latency_ms );

    return;

    //console.log(from_slab.id, '(origin) memo  ', memo.id);
    //console.log(to_slab.id, '(dest)   memo  ' );

    /*
     console.log('pushMemo completed for memo id', cloned_memo.id, 'replicas are:', packet.p );
     console.log('original memo replicas are', memo.p );
    */

}

Mesh.prototype.sendPeeringChanges = function( sending_slab_id, peeringchanges ) {
    var me = this;

    console.log('mesh.sendPeeringChanges from slab[' + sending_slab_id + ']', peeringchanges );

    Object.keys(peeringchanges).forEach(function(receiving_slab_id){
        var slab = me._slabs[receiving_slab_id];
        if( slab ){
            var rv = slab.receivePeeringChange( sending_slab_id, peeringchanges[ receiving_slab_id ] );
        }
    });

}

// export the class
module.exports = Mesh;
