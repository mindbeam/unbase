async function load() {
    start(await import('../pkg/tree-clock-sim-rs'));
}

load();

async function start(mymod: typeof import('../pkg/tree-clock-sim-rs')) {

    console.log("All modules loaded");
    await mymod.hello_worlx();

    init();
    animate();
}

import './style.css'
import * as DiscImage from './textures/sprites/disc.png';
import * as THREE from 'three'
import * as Stats from 'stats.js'
import * as dat from 'dat.gui'
import {Slab,SlabSet} from "./slab";
import DragControls from 'three-dragcontrols';
import * as TrackballControls from 'three-trackballcontrols';

var camera, scene, renderer, stats, material;
var mouseX = 0, mouseY = 0;
var windowHalfX = window.innerWidth / 2;
var windowHalfY = window.innerHeight / 2;


var slabset;
var frame = -1;
var time = -1;
var status = {
    Run: true,
    Speed: 1,
    Slabs: 200.0,
    Neighbors: 8,
    Chattyness: 0.02,
    "3D": false,
    Dropper: function(){},
    "RandNeighbor": function(){},
    "ResetColor": function(){},
};
var mouse;
var trackBallControls;
var dragControls;

function init() {
    camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 2, 4000 );
    camera.position.z = 1500;
    scene = new THREE.Scene();
    scene.fog = new THREE.FogExp2( 0x000000, 0.0004 );
    scene.fog.near = 1000;
    scene.fog.far = 4000;

    mouse = new THREE.Vector2();
    renderer = new THREE.WebGLRenderer();

    init_slabs();

    renderer.setPixelRatio( window.devicePixelRatio );
    renderer.setSize( window.innerWidth, window.innerHeight );
    document.body.appendChild( renderer.domElement );
    //
    stats = new Stats();
    document.body.appendChild( stats.dom );

    trackBallControls = new TrackballControls( camera, renderer.domElement );
    trackBallControls.rotateSpeed = 1.0;
    trackBallControls.zoomSpeed = 1.2;
    trackBallControls.panSpeed = 0.8;
    trackBallControls.noZoom = false;
    trackBallControls.noPan = false;
    trackBallControls.staticMoving = true;
    trackBallControls.dynamicDampingFactor = 0.3;


    //dragControls = new DragControls([], camera, renderer.domElement);

    //dragControls.addEventListener( 'dragstart', function ( event ) { trackBallControls.enabled = false; } );
    //dragControls.addEventListener( 'dragend', function ( event ) { trackBallControls.enabled = true; } );

    var gui = new dat.GUI();

    gui.add(status,'Run');
    gui.add(status,'Speed',0,5);
    gui.add(status, 'Slabs', 2, 5000).onChange(function(){
        console.log('CHANGE');
        init_slabs();
    });
    gui.add(status,'3D').onChange(function(){
        init_slabs();
    });

    gui.add(status,'Dropper').onChange(function(){
        var slab = slabset.select_random_slab();
        slab.color = new THREE.Color(0xff0000 );
        slabset.update_attributes();
    });
    gui.add(status,'Chattyness',0.0,0.5).onChange(function(){
        slabset.chattyness = status.Chattyness;
    });
    gui.add(status,'Neighbors',1,15).onChange(function(){
        slabset.randomize_all_neighbors(status.Neighbors);
    });
    gui.add(status,'RandNeighbor').onChange(function(){
        slabset.randomize_all_neighbors(status.Neighbors);
    });

    gui.add(status,'ResetColor').onChange(function(){
        slabset.reset_all_colors();
    });
    // gui.add( material, 'sizeAttenuation' ).onChange( function() {
    //     material.needsUpdate = true;
    // } );
    // gui.open();


    document.addEventListener( 'mousemove', onDocumentMouseMove, false );
    document.addEventListener( 'touchstart', onDocumentTouchStart, false );
    document.addEventListener( 'touchmove', onDocumentTouchMove, false );

    window.addEventListener( 'resize', onWindowResize, false );
}
function init_slabs (){
    scene.remove.apply(scene, scene.children);
    slabset = new SlabSet( scene, status.Slabs, status );
    slabset.chattyness = status.Chattyness;
    slabset.create_random_slabs( status.Slabs, status["3D"] );
    slabset.randomize_all_neighbors(status.Neighbors);
}
function onWindowResize() {
    windowHalfX = window.innerWidth / 2;
    windowHalfY = window.innerHeight / 2;
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize( window.innerWidth, window.innerHeight );
}

function onDocumentMouseMove( event ) {
    event.preventDefault();
    mouse.x = ( event.clientX / window.innerWidth ) * 2 - 1;
    mouse.y = - ( event.clientY / window.innerHeight ) * 2 + 1;
}

// function onDocumentMouseMove( event ) {
//     mouseX = event.clientX - windowHalfX;
//     mouseY = event.clientY - windowHalfY;
// }
function onDocumentTouchStart( event ) {
    if ( event.touches.length == 1 ) {
        event.preventDefault();
        mouseX = event.touches[ 0 ].pageX - windowHalfX;
        mouseY = event.touches[ 0 ].pageY - windowHalfY;
    }
}
function onDocumentTouchMove( event ) {
    if ( event.touches.length == 1 ) {
        event.preventDefault();
        mouseX = event.touches[ 0 ].pageX - windowHalfX;
        mouseY = event.touches[ 0 ].pageY - windowHalfY;
    }
}

function animate() {
    requestAnimationFrame( animate );
    render();
    stats.update();
}

function render() {

    frame++;
    camera.lookAt( scene.position );

    if (status.Run){
        time += status.Speed;
        slabset.update(time);
    }

    trackBallControls.update();
    renderer.render( scene, camera );

}
