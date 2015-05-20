
//var memo_cls = require('./record');

/* Record
 * A record is a bundle of values representing a discrete thing.
 * Its present state is determined by the totality of its memos
*/

function Record(id, peering, slab, vals) {
    this.id = id;
    this.slab = slab;
    
    //this.initPeering();
    //this.setPeering( peering );
    
    // Temporary hack - doing the value init here out of convenience
    // because edit propagation doesn't work yet. relying in the initial pushItemToSlab for preliminary testing
    vals = vals || {};
    var val;
    Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Record ){
                vals[key] = val.id;
                slab.registerItemPeering( this, val.id, slab.id );
            }else{
                throw "need a slab id AND a record id";
            }
            // else, should already be a valid record id
            // TBD: how to convey locations of said record id
            
        }
        
    });
    
    
    
    slab.putItem(this);
 
    //this.slab = slab;   // A record object only exists within the context of a slab
    //this.memos = memos; // the present state of a record is determined by the sum of it's (relevant) memos
    // do records even have replicas?? or just memos
  
}

Record.prototype.set = function(args){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */
    
    var id = this.id + '-' + (this.memo_increment++).toString(36),
        m  = new memo_cls(id,args)
}

Record.prototype.serialize = function(){
    var vals = this.v,
        val
    ;
    
  /*  Object.keys(vals).forEach(function(key){
        if( key.charAt(0) == '$' ){
            val = vals[key];
            if( val instanceof Record ) vals[key] = val.id;
            // else, should already be a valid record id
            // TBD: how to convey locations of said record id
        }
    });
    
    var rep =
    */

    return JSON.stringify({
        id: this.id,
        //p:  this.getPeering(),
    });
}

Record.prototype._evicting    = 0;
Record.prototype.__replica_ct = 1;

// should we un-set this if an eviction fails?
Record.prototype.evicting = function(v) {
    this._evicting = v ? 1 : 0;
};

Record.prototype.desiredReplicas = function() {
   return Math.max(0,(this.__replica_ct - this.slab.getPeers(this.id,true).length) + this._evicting);
};
    
// export the class
module.exports.createRecord = function(slab,vals){
    var id = slab.genChildID();
    
    var record = new Record(id,null,slab,vals);
    
    return record;
    //vals = vals || {};
    //var set_memo = new memo_cls(slab,vals);
}

module.exports.deserialize = function(slab, serialized){
    var packet = JSON.parse( serialized );
    if(typeof packet != 'object') return null;
        
    var record = new Record(packet.id,packet.p,slab);
  
    return record;
}
