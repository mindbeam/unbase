describe('disappearing messages', function() {
    it('should originate some edits');
    it('should lose a portion of the messages in queue');
    it('should attempt, after a reasonable timeout to retransmit');
    it('should update records to reflect the revised peering status');
    it('should automatically reattempt to re-peer the memos in question');
    it('should hold the transaction commit until all memos are believed to be sufficiently distributed')
});
