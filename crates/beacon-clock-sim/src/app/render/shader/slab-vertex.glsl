//uniform float time;

//attribute vec3 position;

//precision mediump float;


//attribute float last_memo_time;
//attribute vec3 customColor;
//varying vec3 vColor;
//${THREE.ShaderChunk[ "fog_pars_vertex" ]}


//float cubicPulse( float c, float w, float x ){
//    x = abs(x - c);
//    if( x>w ) return 0.0;
//    x /= w;
//    return 1.0 - x*x*(3.0-2.0*x);
//}
//void main() {
    //vColor = customColor;
    //float size = 100.0 + (100.0 * cubicPulse(0.5,0.5,smoothstep(-50.0,50.0, time - last_memo_time)));

    // ${THREE.ShaderChunk[ "fog_vertex" ]}

//}

//uniform mat4 modelViewMatrix;
//uniform mat4 projectionMatrix;
//uniform vec3 cameraPosition;

varying  vec4 color;
attribute vec3 position;
attribute vec4 vRgbaColor;
void main() {
//    vec4 worldPosition = modelViewMatrix * vec4(position.x, 0.0, position.y, 1.0);
//    vec4 clipSpace = projectionMatrix * modelViewMatrix *  worldPosition;
//    gl_Position = clipSpace;

    //vec4 mvPosition = modelViewMatrix * vec4( position, 1.0 );
//    float size = 100.0;
//    gl_PointSize = size * ( 300.0 / -mvPosition.z );
//    gl_Position = projectionMatrix * mvPosition;

    gl_PointSize = 100.0;
    gl_Position = vec4( position, 1.0 );
    color = vRgbaColor;
}