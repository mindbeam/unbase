
function Mesh() {
    this._slabs = [];
}

// class methods
Mesh.prototype.register_slab = function( slab ) {
    this._slabs.push( slab );
    console.log('Registered Slab ', slab.id);
};

Mesh.prototype.get_accepting_peers = function( slab, number ) {
    number = (typeof number == 'number' && number > 0 ) ? number : 1;

    return this._slabs.filter(function(s){
	return (s.id != slab.id) && s.isAccepting() && number-- > 0;
    });
}

Mesh.prototype.push_grain = function( peer, grain ) {
    peer.acceptGrain( grain );
    grain.recordReplica( peer.id );
}


// export the class
module.exports = Mesh;
