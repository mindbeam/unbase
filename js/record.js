
var memo_cls = require('./record');
var peerable_cls = require('./mixin/peerable');

/* Record
 * A record is a bundle of values representing a discrete thing.
 * Its present state is determined by the totality of its memos
*/

function Record(slab,vals) {
    this.id = slab.genChildID();
    
    vals = vals || {};
    var set_memo = new memo_cls(slab,vals);
    
    this.registerPeer('self',slab.id,true);
    
    slab.putRecord(g);
    slab.pushRecord( g, cb );
 
    //this.slab = slab;   // A record object only exists within the context of a slab
    //this.memos = memos; // the present state of a record is determined by the sum of it's (relevant) memos
    // do records even have replicas?? or just memos
  
}
peerable_cls.mixin(Record);

Record.prototype.set = function(args){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */
    
    var id = this.id + '-' + (this.memo_increment++).toString(36),
        m  = new memo_cls(id,args)
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
