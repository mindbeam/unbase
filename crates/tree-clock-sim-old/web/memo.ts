import {Slab} from './slab'
import {BufferGeometry, Float32BufferAttribute, Points, Scene, Vector3} from "three";
import * as THREE from "three";
import * as DiscImage from './textures/sprites/disc.png';
import * as shader from './shader';

// Reading = Array<Memo>
// Memo = SlabID+Increment, Array<Memo>



let speed = 5.0;

export class Memo {
    slab_id: number;
    increment: number;
    public parents: MemoHead;
    public color: THREE.Color;
    constructor(slab: Slab, color: THREE.Color){
        this.slab_id = slab.id;
        this.increment = slab.get_increment();
        this.parents = slab.clockstate;
        this.color = color;
    }
    descends(memo: Memo){
        if (memo == this){
            return true;
        }
        return this.parents.descends_or_contains(memo);
    }
}

export class MemoHead {
    memos: Array<Memo>;
    constructor(memos: Array<Memo>){
        this.memos = memos;
    }
    apply(new_memo: Memo):MemoHead{
        var new_head : Array<Memo> = [new_memo];

        for (let memo of this.memos) {
            if (memo === new_memo){
                //
            }else if (new_memo.descends(memo)) {
                //
            }else{
                new_head.push(memo);
            }
        }

        return new MemoHead(new_head);
    }
    descends_or_contains(memo){
        if (this.memos.length == 0) {
            return false; //  searching for positive descendency, not merely non-ascendency
        }
    }
}

export class MemoEmission {
    memo: Memo;
    from_slab: Slab;
    to_slab: Slab;
    emit_time: number;
    distance: number;
    duration: number;
    index: number;
    public delivered: boolean;
    constructor( from_slab: Slab, to_slab: Slab, emit_time: number, index: number, memo: Memo ){
        var q = from_slab;
        var p = to_slab;

        // how many frames should this memo be inflight?
        this.emit_time = emit_time;
        this.distance = Math.sqrt( ((q.x - p.x)**2) + ((q.y - p.y)**2) + ((q.z - p.z)**2) );
        this.duration = Math.floor(this.distance / speed );
        this.from_slab = from_slab;
        this.to_slab = to_slab;
        this.delivered = false;
        this.memo = memo;
        this.index = index;
    }
    deliver(){
        this.to_slab.deliver(this);
        this.delivered = true;
    }
}
class MemoEmissionUniforms{
    time: Object;
    color: Object;
    texture: Object;
    is_memo: true;
    constructor() {
        {
            var sprite = new THREE.TextureLoader().load( DiscImage );

            this.time = { value: 0 };
            this.color = { value: new THREE.Color( 0xcccccc ) };
            this.texture = { value: sprite };
        }
    }
}

export class MemoEmissionSet {
    emissions: Array<MemoEmission>;
    emission_free_slots: Array<number>;
    max_allocated_index: number;
    geometry: BufferGeometry;
    uniforms: any;
    points: Points;
    pool_size: number;
    status: Object;
    positionAttribute: THREE.BufferAttribute;
    customColorAttribute: THREE.Float32BufferAttribute;
    destinationAttribute: THREE.Float32BufferAttribute;
    emitTimeAttribute: THREE.Float32BufferAttribute;
    durationAttribute: THREE.Float32BufferAttribute;

    constructor(scene: Scene, status: Object) {
        this.pool_size = 100000;
        this.status = status;
        this.max_allocated_index = -1;

        this.emission_free_slots = new Array(this.pool_size);
        for (let i=0;i<this.pool_size;i++){
            this.emission_free_slots[i] = (this.pool_size - i) - 1;
        }

        this.geometry = new THREE.BufferGeometry();

        this.positionAttribute = new THREE.Float32BufferAttribute(new Float32Array(this.pool_size * 3), 3);
        this.positionAttribute.setDynamic(true);
        this.geometry.addAttribute('position',  this.positionAttribute);

        this.customColorAttribute = new THREE.Float32BufferAttribute(new Float32Array(this.pool_size * 3), 3);
        this.customColorAttribute.setDynamic(true);
        this.geometry.addAttribute('customColor', this.customColorAttribute );

        this.destinationAttribute = new THREE.Float32BufferAttribute(new Float32Array(this.pool_size * 3), 3);
        this.destinationAttribute.setDynamic(true);
        this.geometry.addAttribute('destination', this.destinationAttribute);

        this.emitTimeAttribute = new THREE.Float32BufferAttribute(new Float32Array(this.pool_size * 1), 1);
        this.emitTimeAttribute.setDynamic(true);
        this.geometry.addAttribute('emit_time', this.emitTimeAttribute );

        this.durationAttribute = new THREE.Float32BufferAttribute(new Float32Array(this.pool_size * 1), 1);
        this.durationAttribute.setDynamic(true);
        this.geometry.addAttribute('duration', this.durationAttribute );

        var sprite = new THREE.TextureLoader().load( DiscImage );
        this.uniforms = {

            time: { value: 0 },
            color: { value: new THREE.Color( 0xffffff ) },
            texture: { value: sprite },

            fogColor: { value: new THREE.Color( 0x000000 ) },
            fogDensity: { value: 0.00025 },
            fogFar: {value: 2000 },
            fogNear: { value: 1 },
        };;
        this.emissions = [];

        var material = new THREE.ShaderMaterial({
            uniforms: this.uniforms,
            vertexShader: shader.memo_vertex,
            fragmentShader: shader.memo_fragment,
            alphaTest: 0.5,
            fog: true,
        });

        this.points = new THREE.Points(this.geometry, material);
        this.points.frustumCulled = false;
        scene.add(this.points);
    }
    update_attributes(index){

        var emission = this.emissions[index];
        var memo = emission.memo;
        var from_slab = emission.from_slab;
        var to_slab   = emission.to_slab;

        this.positionAttribute.setXYZ(index, from_slab.x, from_slab.y, from_slab.z);
        this.destinationAttribute.setXYZ(index, to_slab.x, to_slab.y, to_slab.z);
        this.emitTimeAttribute.setX(index, emission.emit_time );
        this.durationAttribute.setX(index, emission.duration );
        this.customColorAttribute.setXYZ(index, memo.color.r, memo.color.g, memo.color.b);

        this.updateRange(this.positionAttribute.updateRange, index, 3);
        this.updateRange(this.destinationAttribute.updateRange, index, 3);
        this.updateRange(this.emitTimeAttribute.updateRange, index, 1);
        this.updateRange(this.durationAttribute.updateRange, index, 1);
        this.updateRange(this.customColorAttribute.updateRange, index, 3);

        //
        this.positionAttribute.needsUpdate = true;
        this.destinationAttribute.needsUpdate = true;
        this.emitTimeAttribute.needsUpdate = true;
        this.durationAttribute.needsUpdate = true;
        this.customColorAttribute.needsUpdate = true;
        //
        //this.positionAttribute.updateRange  = {offset:0, count: this.max_allocated_index*3};
        // this.destinationAttribute.updateRange = {offset:0, count: this.max_allocated_index*3};
        // this.emitTimeAttribute.updateRange = {offset:0, count: this.max_allocated_index};
        // this.durationAttribute.updateRange = {offset:0, count: this.max_allocated_index};
        // this.customColorAttribute.updateRange = {offset:0, count: this.max_allocated_index*3};

    }
    updateRange( ur, index, itemsize ) {
        // TODO: this is pretty ugly. Clean it up
        let offset = index * itemsize;

        if (ur.count == -1){
            //console.log('UR INIT', index, 1);
            ur.offset = offset;
            ur.largest_offset = offset;
            ur.count = itemsize;
        } else if (index * itemsize < ur.offset){
            ur.offset = offset;
            ur.count = (ur.largest_offset - ur.offset) + itemsize;
        }else if ( offset > (ur.offset + ur.count) ){
            ur.largest_offset = offset;
            ur.count = (ur.largest_offset - ur.offset) + itemsize;
        }
    }
    update (time: number) {
        for (let i=0; i < this.emissions.length; i++){
            var memo = this.emissions[i];
            if (memo && (time > (memo.emit_time + memo.duration))){ // will need to extend this if there's any delivery flourish
                //console.log('deliver', i, time, memo.emit_time, memo.duration, memo.emit_time + memo.duration);
                memo.deliver();
                this.deallocate(memo.index);
            }
        }
        // var uniforms: any = this.uniforms;
        this.uniforms.time.value = time;
    }
    reset_all_colors(){
        var emission,memo;

        var newcolor = new THREE.Color(0xffffff);
        for ( var i = 0, l = this.emissions.length; i < l; i ++ ) {
            emission = this.emissions[i];
            if (emission){
                memo     = emission.memo;
                memo.color = newcolor.clone();
                this.customColorAttribute.setXYZ(i, memo.color.r, memo.color.g, memo.color.b);
            }
        }
        // Using the big hammer here, because this is the only place we should ever update the whole lot of 'em
        this.customColorAttribute.needsUpdate = true;
        this.customColorAttribute.updateRange = { offset: 0, count: -1 };
    }

    send_memo(from_slab: Slab, to_slab: Slab, emit_time: number, memo: Memo){ // color is a cheap analog for tree clock fragment
        var index = this.allocate();

        var emission = new MemoEmission(from_slab, to_slab, emit_time, index, memo);

        if (typeof index == 'undefined'){
            var status : any = this.status;
            status.Run = false;
            alert("Exceeded maximum memo inflight buffer size");
            return;
        }

        this.emissions[index] = emission;
        this.update_attributes(index);

    }
    allocate() {
        var index = this.emission_free_slots.pop();
        if (index > this.max_allocated_index) {
            this.max_allocated_index = index;
            //console.log('set draw range', this.max_allocated_index);
            this.geometry.setDrawRange( 0, this.max_allocated_index );
        }
        return index;
    }
    deallocate(index: number) {
        this.emissions[index] = undefined;
        this.emission_free_slots.push(index);


        // TODO: more efficiently manage this so we don't have to iterate the active slots to update max_allocated_index downward
        //
        this.geometry.setDrawRange( 0, this.max_allocated_index + 1 );

    }
}
