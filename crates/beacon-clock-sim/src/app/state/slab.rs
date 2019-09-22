use crate::app::{Color, Position};

struct Slab {
    pub id: u32,
    pub position: Position,
    pub color: Color,
    pub neighbors: Array<[f32,Slab]>;
    pub beacon_increment: number;
    pub clockstate: MemoHead;
}

impl Slab {
    pub fn new(slabset: &SlabSet, id: number, threedim: boolean, seed: MemoHead) {
        Slab {
            id,
            position: Position::random_2d(0f32),
            color: Color::from(0xffffff);
            beacon_increment: 0,
            clockstate: seed
        }
    }

    pub fn init_new_system(){

        this.clockstate = new MemoHead([]);
        var color = Color::from(0xffffff);
        var memo = new Memo(this, color);
        this.clockstate = new MemoHead([memo]);
    }
    get_increment() {
    return this.beacon_increment + +;
    }
    select_peers(count){
    count = Math.min(count, this.neighbors.length);

    return this.neighbors.slice(0, count).map( n => n[1] );
    }
    deliver(memoemission: MemoEmission) {
    this.apply_memo( memoemission.memo );

    let from_slab_id = memoemission.from_slab.id;
    let neighbor_index = this.neighbors.findIndex( n => n[1].id == from_slab_id );
    if (neighbor_index > - 1){
    let ex = this.neighbors.splice(neighbor_index, 1)[0];
    ex[0] + +;
    this.neighbors.unshift(ex);
    }else{
    this.neighbors.unshift([1, memoemission.from_slab]);
    }

    this.neighbors = this.neighbors.sort((a, b) => b[0] - a[0]);
    if (this.neighbors.length > 5){
    this.neighbors.pop();
    }
    }
    apply_memo (new_memo: Memo){

    this.clockstate = this.clockstate.apply(new_memo);

    this.color.multiply( new_memo.color )
    var customColor = < Float32BufferAttribute > this.slabset.geometry.getAttribute('customColor');
    customColor.setXYZ(this.id, this.color.r, this.color.g, this.color.b);
    customColor.needsUpdate = true;
    }
    choose_random_neighbors(count: number){
    this.neighbors = [];
    if ( this.slabset.slabs.length > 0 ){
    for ( var i = 0; i < count; i + + ) {
    var neighbor = this.slabset.slabs[Math.floor(Math.random() * this.slabset.slabs.length )];
    if (neighbor == this){
    if (this.slabset.slabs.length > 1){
    i - -;
    }
    continue;
    }
    if (neighbor & & this.neighbors.findIndex( n => n[1].id == neighbor.id) == - 1) {
    this.neighbors.push([0, neighbor]);
    }
    }
    }
    }
}