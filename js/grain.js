
/* Grain
 *   A logical node which originates from a single slab (and is numbered by said slab)
 *   but is replicated across several other peer slabs for persistence, redundancy, and scalability
 *
 *  TODO: work out the most memory efficient way to structure the grain object
 *  while still being colllision resistant
*/
function Grain(id,vals,replicas) {
  this.id = id;
  this.v  = vals;
  this.r  = replicas || [];
  
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

Grain.prototype.set = function(args){
    /*
     * Update values of this grain. Presently schemaless. should have a schema in the future
     * 
     *    Simple values as foo: "string value or simple struct"
     *    References   as $bar: "string grain id or direct reference"
     *
     *    TBD: move this to the slab object, or include a slab reference in every grain ( yuck )
     *    Note that the grain id prefix DOES NOT identify the slab in which it's currently resident
    */
}

Grain.prototype.packetize = function(){
    var vals = this.v,
        val
    ;
    
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Grain ) vals[key] = val.id;
            // else, should already be a valid grain id
            // TBD: how to convey locations of said grain id
        }
    });
    return { id: this.id, vals: this.v, replicas: this.r };
}

// export the class
module.exports = Grain;
