//uniform float time;
uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;
//uniform vec3 cameraPosition;

attribute vec3 position;

precision mediump float;


//attribute float last_memo_time;
//attribute vec3 customColor;
//varying vec3 vColor;
//${THREE.ShaderChunk[ "fog_pars_vertex" ]}


float cubicPulse( float c, float w, float x ){
    x = abs(x - c);
    if( x>w ) return 0.0;
    x /= w;
    return 1.0 - x*x*(3.0-2.0*x);
}
void main() {
    //vColor = customColor;
    vec4 mvPosition = modelViewMatrix * vec4( position, 1.0 );
    //float size = 100.0 + (100.0 * cubicPulse(0.5,0.5,smoothstep(-50.0,50.0, time - last_memo_time)));
    float size = 100.0;
    gl_PointSize = size * ( 300.0 / -mvPosition.z );
    gl_Position = projectionMatrix * mvPosition;

    // ${THREE.ShaderChunk[ "fog_vertex" ]}

}