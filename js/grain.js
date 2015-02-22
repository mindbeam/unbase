
function Grain(id,vals) {
  this.id = id;
  this.v  = vals;
  this.r  = [];
  
  /*
   * Questions:
   * Do we want each grain to have a reference to it's slab?
  */
  
}
Grain.prototype.__replica_ct = 1;

// class methods
Grain.prototype.desiredReplicas = function() {
   return this.__replica_ct - this.r.length;
};

Grain.prototype.registerReplica   = function( peer_id ) {
    this.r.push( peer_id );
}
Grain.prototype.deregisterReplica = function( peer_id ){
    var found = false;
    
    this.r = this.r.filter(function(id){
        if( id == peer_id ){
            found = true;
            return false;
        }
        return true;
    });
    
    return found;
}
Grain.prototype.getReplicas = function(){
    return this.r;
}

// export the class
module.exports = Grain;
