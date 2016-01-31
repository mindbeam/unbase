
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

function Record(slab,id, memos) {
    this.id = id;
    this.slab = slab;
    this.memos_by_id = {};
    this.memos_by_parent = {};

    this.addMemos(memos);

}

module.exports.reconstitute = function(slab,id,memos){
    return new Record( slab, id, memos );
}

module.exports.create = function(slab,vals){

    var id = 'R.' + slab.genChildID();
    console.log('record.create', id);

    var firstmemo = memo_cls.create( slab, id, null, vals );
    var record = new Record( slab, id, [firstmemo] );

    slab.addRecord(record);

    return record;
}

Record.prototype.addMemos = function(memos){
    var me = this;

    memos.forEach(function(memo){
        memo.parents.forEach(function(parent){
            me.memos_by_parent[parent] = memo.id;
        });
        me.memos_by_id[memo.id] = memo;
    });
}

Record.prototype.set = function(vals){
    /*
     * Update values of this record. Presently schemaless. should have a schema in the future
    */

    var memo = new memo_cls.create( this.slab,this.id, this.getHeadMemoIDs(), vals );
    this.addMemos([memo]);

}

var memosort = function(a,b){
    // TODO - implement sorting by beacon-offset-millisecond LWW or node id as required to achieve desired determinism

    if ( a.id < b.id )
        return -1;
    if ( a.id > b.id )
        return 1;

    return 0;
}

// TODO - convert into a callback
Record.prototype.get = function(field){
    var me = this;
    var value = undefined;

    var done = false;
    var memos = this.getHeadMemos();

    while(!done){
        var nextmemos = [];
        memos.sort(memosort).forEach(function(memo){
            if (typeof memo.v[field] !== 'undefined'){
                value = memo.v[field];
                done = true;
            }else{
                memo.parents.forEach(function(pid){ nextmemos.push(me.memos_by_id[pid]) });
            }
        });
        console.log('nextmemos',nextmemos, done);
        if(!nextmemos.length) done = true;

        // TODO - Look up memos from slab
        if(!done) memos = nextmemos;
    }

    return value;
}

Record.prototype.getHeadMemos = function(){
    // The head of a record consists of all Memo IDs which are not parents of any other memos
    var me = this;

    var head = [];
    var parentmap = me.memos_by_parent;
    Object.getOwnPropertyNames(me.memos_by_id).forEach(function(id){
        var memo = me.memos_by_id[ id ];
        if(!parentmap[ memo.id ]) head.push( memo );
    });

    return head;
};
Record.prototype.getHeadMemoIDs = function(){
    return this.getHeadMemos().map(function(memo){ return memo.id });
}


Record.prototype.getMemoIDs = function(){
    return Object.keys(this.memos_by_id);
};


// The memo is getting removed from the Slab. Clean up our reference to it here
// somewhat less necessary now, as record objects are short-lived, but still not a bad idea in case there
// are a lot of memos attached to this record
Record.prototype.killMemo = function(memo){
    var me = this;
    if(!me.memos_by_parent[memo.id]) return; // refuse to kill head memos so the record is still viable

    // remove the reference from memos_by_id
    delete this.memos_by_id[ memo.id ];

    // remove the reference from memos_by_parent
    memo.parents.forEach(function(parentID){
        var ref = me.memos_by_parent[parentID];
        var index = ref.indexOf(memo);

        ref.splice(index, 1);
    })
}
