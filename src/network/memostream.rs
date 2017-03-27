use std::collections::BTreeMap;
const MAX_FRAGMENT : usize = 10000;

struct MemoStreamSender {
    dest: TransportAddress,
    seq: u32,
    packets: VecDeque<Packet>
}

impl MemoStreamSender {
    pub fn new (dest: TransportAddress) -> Self {
        MemoStreamSender {
            dest: dest,
            seq: 0,
            packets: VecDeque::with_capacity(10);
        }
    }
    pub fn send_memo( &mut self, memo: &Memo) {
        let b = bincode::serialize(&memo, SizeLimit::Infinite).unwrap();

        let len = b.len();
        let mut offset = 0;

        while taken < len {
            // should be
            let take = MAX_FRAGMENT.min( len - offset );
            let slice = &b[offset..offset+take];
            offset += take;

            let type =
                if ( len == offset ){
                    // Now we're complete
                    if (take == taken){
                        // are we comlee
                        MemoFragmentType::Complete
                    }else{
                        MemoFragmentType::CompleteContinuation(self.seq-1)
                } else {
                    MemoFragmentType::PartialContinuation(self.seq-1)
                }
            ;

            let fragment = MemoFragment{ seq: self.seq, type: type, buf: slice.copy() };

            self.seq += 1;
        }
    }
}
    pub fn yield_memos {

    }
}

type FragmentSequence = u32;
enum MemoFragmentType {
    Complete,
    PartialContinuation(FragmentSequence),
    CompleteContinuation(FragmentSequence),
}
struct MemoFragment {
    seq: FragmentSequence,
    type: MemoFragmentType,
    buf: [u8]
}
// Packets:
struct Packet {
    envelope_id: u64,
    fragments: Vec<MemoFragment>
}
