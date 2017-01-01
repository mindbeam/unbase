use slab::Slab;
use memo::Memo;
use record::Record;
use std::sync::{Mutex,Arc};
use std::result;

pub struct ContextShared {
    memos: Vec<Memo>
}

pub struct ContextInner {
    slab: Slab,
    shared: Mutex<ContextShared>
}
#[derive(Clone)]
pub struct Context {
    inner: Arc<ContextInner>
}

impl Context{
    pub fn new ( slab: &Slab ) -> Context {
        Context {
            inner: Arc::new(ContextInner {
                slab: slab.clone(),
                shared: Mutex::new(ContextShared {
                    memos: vec![]
                })
            })
        }
    }
    pub fn get_slab (&self) -> &Slab {
        &self.inner.slab
    }
    pub fn subscribe_record (&self, record: &Record) {

    }
    pub fn unsubscribe_record (&self, record: &Record ){
        
    }
    pub fn get_record (&self, record_id: u64) -> Result<Record, &str> {
        Err("failed to retrieve record")
    }
    pub fn put_memos (&self, memos: Vec<&Memo>){

    }
}

/*

Context.prototype.addMemos = function (memos) {
    var index;

    // TODO: account for possible consolidation among out of order memos being added
    memos.forEach( (memo) => {
        // remove any memo precursors from our present context
        memo.getPrecursors().forEach((id) => {
            index = this._context.indexOf(id);
            if(index != -1) this._context.splice(index, 1);
        });

        //console.log('Context[slab' + this.slab.id + '].addMemo', memo.id);
        if(this._context.indexOf(memo.id) == -1) this._context.push(memo.id);
    });

};
Context.prototype.getPresentContext = function () {
    //console.log('Context[slab' + this.slab.id + '].getPresentContext', this._context);
    return [].concat(this._context); // have to clone this, as it's a moving target
};
Context.prototype.addRecord = function(record){
    this._records_by_id[record.id] = record;
}
Context.prototype.getRecord = function(rid){
    var me = this;

    return new Promise((resolve, reject) => {
        if (!this.slab.hasMemosForRecord(rid)){
            resolve(null);
            return;
        }
        // TODO - perform an index lookup

        var record = record_cls.reconstitute( this, rid );
        // TODO: wait for updates which would be causally sufficient, or reject
        // var t = setTimeout(() => reject(), 2000);

        resolve( record );
        return;
    });
}


use std::sync::mpsc::{Sender,Receiver,channel};
use std::mem;
use std::thread;
use std::result;
use std::thread::JoinHandle;


struct SlabInner{
    rx_thread: Option<JoinHandle<()>>,
}


let ( tx, rx  ) = channel();
internals.tx_map.insert(slab.id,tx);
rx

let me_clone  = me.clone();
inner.rx_thread = Some(thread::spawn(move || {
    for memo in rx.iter() {
        //println!("Got memo from net: {:?}", memo);
    }
}));



pub fn join (self) -> thread::Result<()> {
    let mut inner = self.inner.lock().unwrap();

    match mem::replace(&mut inner.rx_thread, None) {
        Some(t)   => t.join(),
        None      => result::Result::Ok(()) as thread::Result<()>
    }
    //result::Result::Ok(()) as thread::Result<()>
}
*/
