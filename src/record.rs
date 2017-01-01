
/* Record
 *
 * A record is a container for a series of memos on a discrete topic/subject
 * Its present state is determined by the totality of its memos,
 * using keyframe memos as an optimization, such that record state
 * may be projected without traversing the full memo history
 *
 * Records do not have any concept of peering. Only memos have peering
*/

use std::collections::HashMap;
use memo::Memo;
use std::mem;
//use slab::Slab;
use context::Context;

pub struct Record {
    pub id:      u64,
    context: Context,
    head:    Vec<Memo>
}

impl Record {
    pub fn new ( context: Context, vals: HashMap<String, String> ) -> Result<Record,String> {

        let id = context.get_slab().generate_record_id();
        let firstmemo = Memo::new( context.get_slab(), id, vec![], vals );

        let rec = Record {
            id:      id,
            context: context.clone(),
            head:    vec![firstmemo],
        };

        context.subscribe_record( &rec );
        Ok(rec)
    }
    pub fn new_kv ( context: Context, key: &str, value: &str) -> Result<Record,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals )
    }
    pub fn get_value ( &self, _key: &str ) -> Result<&str, &str> {
        Ok("woof")

        // TODO: start from self.head and iterate through parents to collapse the value
    }
    pub fn set_kv (&mut self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        let memo = Memo::new( self.context.get_slab(), self.id, self.head.clone(), vals );
        //self.head = vec![ memo ];

        true
    }
}

impl Drop for Record {
    fn drop (&mut self) {
        self.context.unsubscribe_record(self)
    }
}

/*
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
*/
