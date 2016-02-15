

// should we track causal references only for foreign key reads/writes? or all reads/writes?

describe('intermediate causal consistency', function() {
    it('should initialize three slabs');
    it('should create recordA on slab A');
    it('should create recordB on slab B');
    it('should flush all messages');
    it('should originate an edit to recordB on slab B');
    it('should propagate recordB messages to only slab C');
    it('should open a transaction on slab C');
    it('should read latest recordA on slab C');
    it('should read latest recordB on slab C');
    it('should originate an edit on recordA');
    it('should ensure that recordA contains causal reference to RecordA,RecordB memos');
    it('should commit the transaction on slab C');
    it('should propagate the edit on recordA to slab A');
    it('should attempt to read the value of record B on slab A, blocking');
    it('should propagate the edits on RecordB from slab C to slab A');
    it('should now unblock the read of recordB')

});
