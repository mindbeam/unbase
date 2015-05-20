

/* Memo
 * A memo is an immutable message.
*/
function Memo(slab,vals) {
    this.id = slab.genChildID();
    
    this.registerPeer('self',slab.id,true);
    
    this.v  = vals;
    //this.t = transaction;
    
    /*
     * Questions:
     * Do we want each memo to have a reference to it's slab?
    */
    
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
