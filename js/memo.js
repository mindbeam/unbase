
/* Memo
 * A memo is an immutable message.
*/
function Memo(id,vals,replicas) {
  this.id = id;
  this.v  = vals;
  this.r  = replicas || [];
  
  this.o = originator;
  this.e = endorser;
  this.t = transaction;
  
  /*
   * Questions:
   * Do we want each memo to have a reference to it's slab?
  */
  
}

Memo.prototype._evicting    = 0;
Memo.prototype.__replica_ct = 1;

/* should we un-set this if an eviction fails? */
Memo.prototype.evicting = function(v) {
    this._evicting = v ? 1 : 0;
};

Memo.prototype.desiredReplicas = function() {
   return Math.max(0,(this.__replica_ct - this.r.length) + this._evicting);
};

Memo.prototype.registerReplica   = function( peer_id ) {
    this.r.push( peer_id );
}
Memo.prototype.deregisterReplica = function( peer_id ){
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
Memo.prototype.getReplicas = function(){
    return this.r;
}

Memo.prototype.packetize = function(){
    var vals = this.v,
        val
    ;
    
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Memo ) vals[key] = val.id;
            // else, should already be a valid memo id
            // TBD: how to convey locations of said memo id
        }
    });
    return { id: this.id, vals: this.v, replicas: this.r };
}

// export the class
module.exports = Memo;
