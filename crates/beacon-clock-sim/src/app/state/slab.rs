use crate::util::{Color, Position};
use uuid::Uuid;
use log::info;

#[derive(Clone)]
pub struct MemoRefHead;

impl MemoRefHead {
    pub fn blank() -> Self {
        MemoRefHead
    }
}

pub struct SlabSystem{
    pub id: Vec<Uuid>,
    pub position: Vec<f32>,
    pub color: Vec<f32>,
    pub clockstate: Vec<MemoRefHead>
}

impl SlabSystem {
    pub fn new() -> Self {
        SlabSystem {
            id: Vec::new(),
            position: Vec::new(),
            color: Vec::new(),
            clockstate: Vec::new(),
        }
    }
    pub fn len (&self) -> usize {
        self.id.len()
    }
    pub fn new_slab(&mut self, position: Position, seed: MemoRefHead) -> usize {
        let offset = self.id.len();

        let color = Color::from(0xffffffff);

        self.id.push(Uuid::new_v4());
        self.position.push(position.x as f32 / 100.0);
        self.position.push(position.y as f32 / 100.0);
        self.position.push(position.z as f32 / 100.0);
        self.color.push(color.r as f32 / 255.0);
        self.color.push(color.g as f32 / 255.0);
        self.color.push(color.b as f32 / 255.0);
        self.color.push(color.a as f32 / 255.0);
        self.clockstate.push(seed);

        offset
    }
    pub fn truncate (&mut self) {
        self.id.truncate(0);
        self.position.truncate(0);
        self.color.truncate(0);
        self.clockstate.truncate(0);
    }
    pub fn create_random_slabs(&mut self, count: u32, threedim: bool) {

        info!("create_random_slabs {}, {}",count,threedim);
        // TODO call init_new_system here
        let seed = MemoRefHead::blank();

        if threedim {
            for i in 0..count {
                let position = Position::random_3d();
                self.new_slab(position, seed.clone());
            }
        } else {
            for i in 0..count {
                let position = Position::random_2d(0f32);
                self.new_slab(position, seed.clone());
            }
        }
    }
}

pub struct Slab {
    pub offset: usize
}

impl Slab {

//    pub fn init_new_system(){
//        this.clockstate = new MemoHead([]);
//        var color = Color::from(0xffffff);
//        var memo = new Memo(this, color);
//        this.clockstate = new MemoHead([memo]);
//    }
//    fn get_increment() -> {
//        return this.beacon_increment + +;
//    }
//    fn select_peers(count){
//        count = Math.min(count, this.neighbors.length);
//        return this.neighbors.slice(0, count).map( n => n[1] );
//    }
//    deliver(memoemission: MemoEmission) {
//    this.apply_memo( memoemission.memo );
//
//    let from_slab_id = memoemission.from_slab.id;
//    let neighbor_index = this.neighbors.findIndex( n => n[1].id == from_slab_id );
//    if (neighbor_index > - 1){
//    let ex = this.neighbors.splice(neighbor_index, 1)[0];
//    ex[0] + +;
//    this.neighbors.unshift(ex);
//    }else{
//    this.neighbors.unshift([1, memoemission.from_slab]);
//    }
//
//    this.neighbors = this.neighbors.sort((a, b) => b[0] - a[0]);
//    if (this.neighbors.length > 5){
//    this.neighbors.pop();
//    }
//    }
//    apply_memo (new_memo: Memo){
//
//    this.clockstate = this.clockstate.apply(new_memo);
//
//    this.color.multiply( new_memo.color )
//    var customColor = < Float32BufferAttribute > this.slabset.geometry.getAttribute('customColor');
//    customColor.setXYZ(this.id, this.color.r, this.color.g, this.color.b);
//    customColor.needsUpdate = true;
//    }
//    choose_random_neighbors(count: number){
//    this.neighbors = [];
//    if ( this.slabset.slabs.length > 0 ){
//    for ( var i = 0; i < count; i + + ) {
//    var neighbor = this.slabset.slabs[Math.floor(Math.random() * this.slabset.slabs.length )];
//    if (neighbor == this){
//    if (this.slabset.slabs.length > 1){
//    i - -;
//    }
//    continue;
//    }
//    if (neighbor & & this.neighbors.findIndex( n => n[1].id == neighbor.id) == - 1) {
//    this.neighbors.push([0, neighbor]);
//    }
//    }
//    }
//    }
}