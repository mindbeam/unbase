pub struct Behavior {
    speed: u32,
    slabs: u32,
    neighbors: u32,
    chattyness: f32,
}

impl Behavior {
    pub fn new() -> Behavior {
        Behavior {
            speed: 1,
            slabs: 200,
            neighbors: 8,
            chattyness: 0.02,
        }
    }

    pub fn applychange( &mut self, change: BehaviorChange ){
        match change {
            Speed(v)      => self.speed = v,
            Slabs(v)      => self.slabs = v,
            Neighbors(v)  => self.neighbors = v,
            Chattyness(v) => self.chattyness = v,
        }
    }
}
