precision mediump float;
uniform float time;
uniform mat4 modelViewMatrix;
uniform mat4 projectionMatrix;

varying vec4 color;
attribute vec3 position;
attribute vec4 vRgbaColor;
void main() {
    vec4 mvPosition = modelViewMatrix * vec4( position, 1.0 );

    float size = 20.0;
    gl_PointSize = size * ( 3.0 / -mvPosition.z );
    gl_Position = projectionMatrix * mvPosition;
    color = vRgbaColor;
}