uniform float time;
//attribute float size;
attribute vec3 customColor;
//attribute vec3 source;
attribute vec3 destination;
attribute float emit_time;
attribute float duration;
varying vec4 vColor;

//${THREE.ShaderChunk[ "fog_pars_vertex" ]}

void main() {
    float elapsed = (time - emit_time);
    float progress = clamp(elapsed / duration, 0.0, 1.0);
    vColor = vec4(customColor, step(time, emit_time + duration)  );
    vec4 mvPosition = modelViewMatrix * vec4( mix(position, destination, progress), 1.0 );

    gl_PointSize = 25.0 * ( 300.0 / -mvPosition.z );

    gl_Position = projectionMatrix * mvPosition;

    //${THREE.ShaderChunk[ "fog_vertex" ]}
}