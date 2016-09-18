
/* Record
 *
 * A record is a container for a series of memos on a discrete topic/subject
 * Its present state is determined by the totality of its memos,
 * using keyframe memos as an optimization, such that record state
 * may be projected without traversing the full memo history
 *
 * Records do not have any concept of peering. Only memos have peering
*/

var memo_cls = require('./memo');

function Record(context,id) {
    this.id = id;
    this.slab = context.slab;
    this.context = context;
    this.memos_by_id = {};
    this.memos_by_parent = {};

    this.slab.subscribeRecord(this);

}

module.exports.reconstitute = function(context,id,memos){
    return new Record( context, id );
}

module.exports.create = function(context,vals){

    var id = 'R.' + context.slab.genChildID();

    var record = new Record( context, id );
    var firstmemo = memo_cls.create( context.slab, id, null, context.getPresentContext(), vals );

    return record;

}
// addMemos gets called after relevant are added to the slab
Record.prototype.addedMemos = function(memos){
    //console.log('Record[' + this.id + ' slab' + this.slab.id + '].addedMemos',this.slab.id, this.id, memos.map((memo) => memo.id));

    // TODO: notice when memos show up and trigger behaviors
}

// FYI, The memo is getting removed from the Slab.
Record.prototype.killMemos = function(memos){
    var me = this;
    // nothing for now
}

Record.prototype.set = function(vals){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */

    var memo = new memo_cls.create( this.slab,this.id, this.getHeadMemoIDs(), this.context.getPresentContext(), vals );
    this.context.addMemos([memo]);
    // TODO - return promise which is fulfilled on transaction commit
}

var memosort = function(a,b){
    // TODO - implement sorting by beacon-offset-millisecond LWW or node id as required to achieve desired determinism

    if ( a.id < b.id )
        return -1;
    if ( a.id > b.id )
        return 1;

    return 0;
}


Record.prototype.get = function(field){
    // TODO: implement promises for get
    return this.getFreshOrNull(field);
}

Record.prototype.getFreshOrNull = function(field){
    var me = this;
    var value = undefined;

    var done = false;
    var memos = this.slab.getHeadMemosForRecord(this.id);

    this.context.addMemos(memos);

    while(!done){
        var nextmemos = [];
        memos.sort(memosort).forEach(function(memo){
            if(!memo.v) console.log(memo);
            if (typeof memo.v[field] !== 'undefined'){
                value = memo.v[field];
                done = true;
            }else{
                memo.parents.forEach(function(pid){ nextmemos.push(me.memos_by_id[pid]) });
            }
        });
        // console.log('nextmemos',nextmemos, done);
        if(!nextmemos.length) done = true;

        // TODO - Look up memos from slab
        if(!done) memos = nextmemos;
    }

    return value;
}

Record.prototype.getHeadMemoIDs = function(){
    return this.slab.getHeadMemoIDsForRecord( this.id );
}


Record.prototype.getMemoIDs = function(){
    return Object.keys(this.memos_by_id);
};
