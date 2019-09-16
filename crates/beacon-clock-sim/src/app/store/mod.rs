use std::ops::Deref;

mod mouse;
use self::mouse::*;

mod camera;
use self::camera::*;

mod behavior;
use self::behavior::*;

pub struct Store {
    pub state: StateWrapper,
}

impl Store {
    pub fn new() -> Store {
        Store {
            state: StateWrapper(State::new()),
        }
    }

    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            _ => self.state.msg(msg),
        }
    }
}

pub struct State {
    clock: f32,
    camera: Camera,
    mouse: Mouse,
    slabs: Vec<Slab>,
    memoemissions: Vec<MemoEmission>,
}

impl State {
    fn new() -> State {
        State {
            /// Time elapsed since the application started, in milliseconds
            clock: 0.,
            camera: Camera::new(),
            mouse: Mouse::default(),
            behavior: Behavior::new(),
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn behavior(&self) -> &Behavior {
        &self.behavior
    }

    /// The current time in milliseconds
    pub fn clock(&self) -> f32 {
        self.clock
    }

    pub fn show_scenery(&self) -> bool {
        self.show_scenery
    }

    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            Msg::AdvanceClock(dt) => {
                self.clock += dt;
            }
            Msg::MouseDown(x, y) => {
                self.mouse.set_pressed(true);
                self.mouse.set_pos(*x, *y);
            }
            Msg::MouseUp => {
                self.mouse.set_pressed(false);
            }
            Msg::MouseMove(x, y) => {
                if !self.mouse.get_pressed() {
                    return;
                }

                let (old_x, old_y) = self.mouse.get_pos();

                let x_delta = old_x as i32 - x;
                let y_delta = y - old_y as i32;

                self.camera.orbit_left_right(x_delta as f32 / 50.0);
                self.camera.orbit_up_down(y_delta as f32 / 50.0);

                self.mouse.set_pos(*x, *y);
            }
            Msg::Zoom(zoom) => {
                self.camera.zoom(*zoom);
            }
            Msg::Behavior(change) => {
                self.behavior.applychange(change);
            },
            Msg::Reset => {
                self.slabs.truncate(0);
                self.memoemissions.truncate(0);
            }
        }
    }
}

pub struct StateWrapper(State);

impl Deref for StateWrapper {
    type Target = State;

    fn deref(&self) -> &State {
        &self.0
    }
}

impl StateWrapper {
    pub fn msg(&mut self, msg: &Msg) {
        &self.0.msg(msg);
    }
}

pub enum Msg {
    AdvanceClock(f32),
    MouseDown(i32, i32),
    MouseUp,
    MouseMove(i32, i32),
    Zoom(f32),
    BehaviorChange(BehaviorChange),
    Reset()
}



//debugger;
//this.uniforms = new SlabUniforms();

this.slabs = [];
this.chattyness = 0.01;

var material = new THREE.ShaderMaterial( {
uniforms: this.uniforms,
vertexShader: shader.slab_vertex,
fragmentShader: shader.slab_fragment,
alphaTest: 0.5,
fog: true,
} );

this.raycaster = new THREE.Raycaster();
this.points = new THREE.Points( this.geometry, material );
scene.add( this.points );

this.memoemissionset = new MemoEmissionSet(scene, status);
}

create_random_slabs( count: number, threedim: boolean ) {
var seed_slab = new Slab(this,0, threedim, new MemoHead([]));
seed_slab.init_new_system();
this.slabs.push(seed_slab);

for (var i = 1; i < count; i++) {
var slab = new Slab(this, i, threedim, seed_slab.clockstate);
this.slabs.push(slab);
}
this.update_attributes();
}
select_random_slab() : Slab {
return this.slabs[Math.floor(Math.random() * this.slabs.length )];
}
update_attributes(){
var attributes = this.points.geometry;

var position = <Float32BufferAttribute> this.geometry.getAttribute('position');
var customColor = <Float32BufferAttribute> this.geometry.getAttribute('customColor');

for ( var i = 0, l = this.slabs.length; i < l; i ++ ) {
var slab = this.slabs[i];
position.setXYZ(i, slab.x, slab.y, slab.z);
customColor.setXYZ(i, slab.color.r, slab.color.g, slab.color.b);
// console.log('setXYZ');
// size.setX(i,200); //Math.max( PARTICLE_SIZE, attributes.size.array[i] * .99 );
}

console.log(position.array.length);
position.needsUpdate = true;
customColor.needsUpdate = true;

}
update (time: number){

// var positions = new Float32Array( vertices.length * 3 );
// var colors = new Float32Array( vertices.length * 3 );
// var sizes = new Float32Array( vertices.length );
// for (let slab of this.slabs){
//     vertices.push( slab.x, slab.y, slab.z );
//     sizes.push( PARTICLE_SIZE * 0.5 );
//
//     // var color = new THREE.Color();
//     // //color.setHSL( 0.01 + 0.2 * ( i / l ), 1.0, 0.5 );
//     // color.setHSL( 0.01 + 0.2 * ( slab.id / PARTICLE_COUNT ), 1.0, 0.5 );
//     // colors.push( color );
//     // color.toArray( colors, slab.id * 3 );
// }

//var uniforms : any = this.uniforms;
this.uniforms.time.value = time;

//if (time % 10 == 0) {
this.send_memos(time);
//}

this.memoemissionset.update(time);
}
randomize_all_neighbors(count: number){
for (let slab of this.slabs) {
slab.choose_random_neighbors(count);
}
}
reset_all_colors(){
this.memoemissionset.reset_all_colors();

for (let slab of this.slabs) {
slab.color = new THREE.Color(0xffffff);
}
this.update_attributes();

}
send_memos(time){

var last_memo_time = <Float32BufferAttribute> this.geometry.getAttribute('last_memo_time');
var other_slab;
var status : any = this.status;
for (let slab of this.slabs){
if (!status.Run) return; // necessary in case we exceed our max inflight memoemissions and need to pause

let number = Math.random();//this.slabs.length);
if ( number < this.chattyness ) {
var memo = new Memo( slab, slab.color.clone() );

for (let other_slab of slab.select_peers(5) ){
//last_memo_time.setX(slab.id, time);
this.memoemissionset.send_memo(slab, other_slab,time, memo);
}
}
}
//last_memo_time.needsUpdate = true;
}
//
// var memo_geometry = new THREE.BufferGeometry();
// memo_geometry.addAttribute( 'position',    new THREE.Float32BufferAttribute( starts,       3 ) );
// memo_geometry.addAttribute( 'destination', new THREE.Float32BufferAttribute( destinations, 3 ) );
// memo_geometry.addAttribute( 'steps',       new THREE.Float32BufferAttribute( steps_list, 1 ) );
// memo_geometry.addAttribute( 'start ',      new THREE.Float32BufferAttribute( starts_list, 1 ) );
//
//



}