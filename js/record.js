
/* Record
*/

function Record(id,memos,slab,replicas) {
  this.id = id;
  this.v  = vals;
 
  this.slab = slab;   // A record object only exists within the context of a slab
  this.memos = memos; // the present state of a record is determined by the sum of it's (relevant) memos
  // do records even have replicas?? or just memos
  
}

Record.prototype._evicting    = 0;
Record.prototype.__replica_ct = 1;

/* should we un-set this if an eviction fails? */
Record.prototype.evicting = function(v) {
    this._evicting = v ? 1 : 0;
};

Record.prototype.desiredReplicas = function() {
   return Math.max(0,(this.__replica_ct - this.r.length) + this._evicting);
};

Record.prototype.registerReplica   = function( peer_id ) {
    this.r.push( peer_id );
}
Record.prototype.deregisterReplica = function( peer_id ){
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
Record.prototype.getReplicas = function(){
    return this.r;
}

Record.prototype.set = function(args){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */
    
    var id = this.id + '-' + (this.memo_increment++).toString(36),
        m  = new memo_cls(id,arguments)
    
}

Record.prototype.packetize = function(){
    var vals = this.v,
        val
    ;
    
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Record ) vals[key] = val.id;
            // else, should already be a valid record id
            // TBD: how to convey locations of said record id
        }
    });
    return { id: this.id, vals: this.v, replicas: this.r };
}

// export the class
module.exports = Record;
