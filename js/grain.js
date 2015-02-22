
function Grain(id,vals) {
  this.id = id;
  this.v  = vals;
  this.r  = [];
}
Grain.prototype.__want_replicas = 1;


// class methods
Grain.prototype.desiredReplicas = function() {
   return this.__want_replicas - this.r.length;
};

Grain.prototype.recordReplica   = function( peer_id ) {
    this.r.push( peer_id );
}

// export the class
module.exports = Grain;
