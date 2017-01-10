
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
//use slab::Slab;
use context::Context;

pub struct RecordHandle {
    pub id:      u64,
    context: Context
}

pub struct RecordSetHandle {

}

impl RecordHandle {
    pub fn new ( context: Context, vals: HashMap<String, String> ) -> Result<RecordHandle,String> {

        let id = context.get_slab().generate_record_id();

        let rec = RecordHandle {
            id:      id,
            context: context.clone()
        };

        context.subscribe_record( &rec );
        Memo::create( context.get_slab(), id, vec![], vals );

        Ok(rec)
    }
    pub fn new_kv ( context: Context, key: &str, value: &str) -> Result<RecordHandle,String> {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        Self::new( context, vals )
    }
    pub fn set_kv (&mut self, key: &str, value: &str) -> bool {
        let mut vals = HashMap::new();
        vals.insert(key.to_string(), value.to_string());

        self.context.create_record_memo(self,vals);
        //self.head = vec![ memo ];

        true
    }
    pub fn get_value ( &self, key: &str ) -> Option<&str> {
        //self.context.get_record_value(self.id, key)
        let mut value : String;

        //let mut memos = self.context.get_record_head(self.id);
        for memo in self.context.subject_memo_iter(self.id) {
            if let Some(v) = memo.inner.values.get(key) {
                return Some(v);
            }
        }
        None
    }

}

impl Drop for RecordHandle {
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

Record.prototype.getHeadMemoIDs = function(){
    return this.slab.getHeadMemoIDsForRecord( this.id );
}


Record.prototype.getMemoIDs = function(){
    return Object.keys(this.memos_by_id);
};
*/
