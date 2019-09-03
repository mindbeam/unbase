import {Memo, MemoHead, MemoEmission, MemoEmissionSet} from './memo'
import * as THREE from "three";
import {BufferGeometry, Scene, Camera, Color, Points, Float32BufferAttribute, Raycaster} from "three";
import * as DiscImage from './textures/sprites/disc.png';
import * as shader from './shader';


// TODO: calculate the mean clock reading comparison time for each slab in the system
// How?
// * Could do it through sampling and actually doing the comparison.
//   (real, or virtualized?  - benefit of real is that it can approximate load,
//    but downside is that it would create even load, which might not be desirable)
// performance will definitely change with which comparisons are requested, but we
// might want to measure hypothetical performance without affecting it, so a hybrid approach seems reasonable...
// That means: do the comparison in "real" mode for places where work is occurring. This yields a clock convergence metric AND effects system dynamics
//             AND also do the comparison (sampled?) in virtual mode in places where "work is not occurring" which will measure hypothetical performance
//             if work were to begin there, without actually changing the performance

// TODO2: Create a plot of convergence time for uniform workload
// TODO3: create a plot of convergence time for newly started workloads (recency to last convergence vs convergence time for median comparator? )

// QUESTION: in all of the above cases, how do we select the comparator? Seems quite unfair to actually select one at random.
// (though "warp" comparator selection is something that might be interesting plot as a point of comparison)
// INITIAL ANSWER: best to have a radius selector, where the comparator point is selected from within that radius
// still may need "warp" option to select comparator immediately, or delay comparator availability for light travel time

// PLOT idea: comparator selection radius versus convergence time versus convergence recency ( 3d plot )



// * Could do a time accelerated simulation (not accurate due to changes of hosting status while requests inflight)_

export class Slab {
    id: number;
    public x: number;
    public y: number;
    public z: number;
    public color: Color;
    neighbors: Array<[number,Slab]>;
    beacon_increment: number;
    clockstate: MemoHead;
    slabset: SlabSet;
    constructor(slabset: SlabSet, id: number, threedim: boolean, seed: MemoHead){
        this.id = id;

        this.clockstate = seed;
        this.beacon_increment = 0;
        this.x = 2000 * Math.random() - 1000;
        this.y = 2000 * Math.random() - 1000;
        if (threedim) {
            this.z = 2000 * Math.random() - 1000;
        }else{
            this.z = 0;
        }
        this.color = new THREE.Color( 0xffffff );

        this.slabset = slabset;
    }
    init_new_system(){
        this.clockstate = new MemoHead([]);
        var color = new THREE.Color(0xffffff);
        var memo = new Memo(this, color);
        this.clockstate = new MemoHead([memo]);
    }
    get_increment() {
        return this.beacon_increment++;
    }
    select_peers(count){
        count = Math.min(count, this.neighbors.length);

        return this.neighbors.slice(0, count).map( n => n[1] );
    }
    deliver(memoemission: MemoEmission) {
        this.apply_memo( memoemission.memo );

        let from_slab_id = memoemission.from_slab.id;
        let neighbor_index = this.neighbors.findIndex( n => n[1].id == from_slab_id );
        if (neighbor_index > -1){
            let ex = this.neighbors.splice(neighbor_index, 1)[0];
            ex[0]++;
            this.neighbors.unshift(ex);
        }else{
            this.neighbors.unshift([1,memoemission.from_slab]);
        }

        this.neighbors = this.neighbors.sort((a, b) => b[0] - a[0]);
        if (this.neighbors.length > 5){
            this.neighbors.pop();
        }
    }
    apply_memo (new_memo: Memo){

        this.clockstate = this.clockstate.apply(new_memo);

        this.color.multiply( new_memo.color )
        var customColor = <Float32BufferAttribute> this.slabset.geometry.getAttribute('customColor');
        customColor.setXYZ(this.id, this.color.r, this.color.g, this.color.b);
        customColor.needsUpdate = true;
    }
    choose_random_neighbors(count: number){
        this.neighbors = [];
        if ( this.slabset.slabs.length > 0 ){
            for ( var i = 0; i < count; i ++ ) {
                var neighbor = this.slabset.slabs[Math.floor(Math.random() * this.slabset.slabs.length )];
                if (neighbor == this){
                    if(this.slabset.slabs.length > 1){
                        i--;
                    }
                    continue;
                }
                if (neighbor && this.neighbors.findIndex( n => n[1].id == neighbor.id) == -1) {
                    this.neighbors.push([0,neighbor]);
                }
            }
        }
    }
}
class SlabUniforms{
    time: Object;
    color: Object;
    texture: Object;
    is_slab: true;
    constructor() {
        {
            var sprite = new THREE.TextureLoader().load( DiscImage );

            this.time = { value: 0 };
            this.color = { value: new THREE.Color( 0xffffff ) };
            this.texture = { value: sprite };
        }
    }
}

export class SlabSet {
    slabs: Array<Slab>;
    geometry: BufferGeometry;
    uniforms: any;
    points: Points;
    memoemissionset: MemoEmissionSet;
    raycaster: Raycaster;
    chattyness: number;
    status: Object;
    constructor(scene: Scene, slab_count: number, status: Object){
        this.geometry = new THREE.BufferGeometry();
        this.geometry.addAttribute( 'position',    new THREE.Float32BufferAttribute( new Float32Array(slab_count * 3), 3 ) );
        this.geometry.addAttribute( 'customColor', new THREE.Float32BufferAttribute( new Float32Array(slab_count * 3), 3 ) );
        this.geometry.addAttribute( 'last_memo_time', new THREE.Float32BufferAttribute( new Float32Array( slab_count * 1), 1 ) );

        this.status = status;

        //var sprite = new THREE.TextureLoader().load( DiscImage );
        // this.uniforms = THREE.UniformsUtils.merge([
        //     THREE.UniformsLib[ "fog" ],
        //     //new SlabUniforms(),
        //     {
        //
        //         time: { value: 0 },
        //         color: { value: new THREE.Color( 0x00ffff ) },
        //         texture: { value: sprite }
        //     }
        // ]);

        var sprite = new THREE.TextureLoader().load( DiscImage );
        //
        this.uniforms = {

            time: { value: 0 },
            color: { value: new THREE.Color( 0xffffff ) },
            texture: { value: sprite },

            fogColor: { value: new THREE.Color( 0x000000 ) },
            fogDensity: { value: 0.00025 },
            fogFar: {value: 2000 },
            fogNear: { value: 1 },
        };

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