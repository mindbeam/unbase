

struct Envelope {
    dest: SlabAddress,
    packets: Vec<Packet>
}

impl Envelope{
    yield_memos
}

struct Packet {
    envelope_id: u64,
    buf: [u8; 10000]
}
