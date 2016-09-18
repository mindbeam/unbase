
var record_cls = require('./record');

function Context(slab,memo_id_list) {
    //params = params || {};
    this.slab = slab;
    this._context = [].concat(memo_id_list);
}

module.exports.create = function(slab,memo_id_list){
    return new Context(slab,memo_id_list);
};
Context.prototype.addRawContext = function(ids){
    ids.forEach( (id) => this._context.push(id) );
};
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
