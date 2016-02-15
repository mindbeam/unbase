
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
    this.disconnected       = params.disconnected       || false;
    this._messages = [];
}

// class methods
Mesh.prototype.registerSlab = function( slab ) {
    this._slabs[slab.id] = slab;
    // console.log('Registered Slab ', slab.id);
};

Mesh.prototype.knownSlabCount = function(){
    return Object.keys(this._slabs).length;
}

/* TODO:
 * Update this to consider RTT, Health, Geo-redundancy and better handle storage quota.
 * Optimize to avoid looping over all known peers every time.
*/

Mesh.prototype.getAcceptingSlabIDs = function( exclude_slab_id, number ) {
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
        if( (slab.quotaRemaining() > 0) && number-- > 0 ) out.push(slab.id);

    });

    return out;

}


Mesh.prototype.pushMemoToSlab = function( from_slab_id, to_slab_id, memo ) {

    /* Shouldn't need to filter replicas, as the putMemo will fail if we're trying to perform a duplicate put
     * This probably isn't very robust, but is useful for proof-of-concept stuffs
     * packet.replicas = packet.replicas.filter(function(id){ return id != to_slab.id });
    */

    // peering handoff is being handled via serialize/deserialize for the time being.
    // This seems weird, on account of the possibility for duplicates across multiple memos being pushed
    // think about:
    // including only ref info in the serialized object
    // with subsequent peering hints, which would then be applied to the registered refs

    console.log('mesh.pushMemoToSlab', memo.id, 'from slab', from_slab_id, 'to slab', to_slab_id);
    this.queueMessage( 'memo', from_slab_id, to_slab_id, memo.packetize() );
};
Mesh.prototype.sendPeeringChanges = function( sending_slab_id, peeringchanges ) {
    var me = this;
    // console.log('mesh.sendPeeringChanges from slab[' + sending_slab_id + ']', peeringchanges );

    Object.keys(peeringchanges).forEach((receiving_slab_id) => {
        this.queueMessage( 'peer', sending_slab_id, receiving_slab_id, peeringchanges[ receiving_slab_id ] );
    });

}

Mesh.prototype.queueMessage = function(type, from_slab_id, to_slab_id, data){
    message = [type,from_slab_id,to_slab_id,JSON.stringify(data)];

    if(this.disconnected){
        this._messages.push(message);
    }else{
        setTimeout(() => {
            this.receiveMessage(message);
        }, this.network_latency_ms);
    }
}
Mesh.prototype.deliverAllQueuedMessages = function(){
    var messages = [].concat(this._messages);
    this._messages.length = 0; // clear the send queue first

    messages.forEach((message) => this.receiveMessage(message));
}

Mesh.prototype.receiveMessage = function(message){

    var type         = message[0],
        from_slab_id = message[1],
        to_slab_id   = message[2],
        data         = JSON.parse(message[3])
    ;

    // console.log('receiveMessage', type, from_slab_id, to_slab_id,data);
    var to_slab = this._slabs[to_slab_id];
    if(!to_slab) return; // drop it to the floor

    // TODO - get a signal back to the originating slab if we're expressly rejecting
    // should it be explicit? or simply reiterate the recipient slab's peering status and limits?
    if( type === 'memo' ){
        if(to_slab.limitRemaining() <= 0) return;
        var cloned_memo = new memo_cls.depacketize( to_slab, data );
    }else if (type === 'peer') {
        var rv = to_slab.receivePeeringChange( from_slab_id, data );
    }

}

// export the class
module.exports = Mesh;
