import * as THREE from "three";

export const slab_vertex = `
    uniform float time;
    attribute float last_memo_time;
    attribute vec3 customColor;
    varying vec3 vColor;
    ${THREE.ShaderChunk[ "fog_pars_vertex" ]}

    
    float cubicPulse( float c, float w, float x ){
        x = abs(x - c);
        if( x>w ) return 0.0;
        x /= w;
        return 1.0 - x*x*(3.0-2.0*x);
    }
    void main() {
        vColor = customColor;
        vec4 mvPosition = modelViewMatrix * vec4( position, 1.0 );
        //float size = 100.0 + (100.0 * cubicPulse(0.5,0.5,smoothstep(-50.0,50.0, time - last_memo_time)));
        float size = 100.0;
        gl_PointSize = size * ( 300.0 / -mvPosition.z );
        gl_Position = projectionMatrix * mvPosition;
        
        ${THREE.ShaderChunk[ "fog_vertex" ]}

    }
`;

export const memo_vertex = `
    uniform float time;
    //attribute float size;
    attribute vec3 customColor;
    //attribute vec3 source;
    attribute vec3 destination;
    attribute float emit_time;
    attribute float duration;
    varying vec4 vColor;
    
    ${THREE.ShaderChunk[ "fog_pars_vertex" ]}
    
    void main() {
    
    
        float elapsed = (time - emit_time);
        float progress = clamp(elapsed / duration, 0.0, 1.0);
        vColor = vec4(customColor, step(time, emit_time + duration)  );
        vec4 mvPosition = modelViewMatrix * vec4( mix(position, destination, progress), 1.0 );
    
        gl_PointSize = 25.0 * ( 300.0 / -mvPosition.z );
    
        gl_Position = projectionMatrix * mvPosition;
        
        ${THREE.ShaderChunk[ "fog_vertex" ]}
    }
`;

export const slab_fragment = `
    uniform vec3 color;
    uniform sampler2D texture;
    varying vec3 vColor;
    
    ${THREE.ShaderChunk[ "common" ]}
    ${THREE.ShaderChunk[ "fog_pars_fragment" ]}
    
    void main() {
        gl_FragColor = vec4( color * vColor, 1.0 );
        gl_FragColor = gl_FragColor * texture2D( texture, gl_PointCoord );
        if ( gl_FragColor.a < ALPHATEST ) discard;
        ${THREE.ShaderChunk[ "fog_fragment" ]}
    }
`;

export const memo_fragment = `
    uniform vec3 color;
    uniform sampler2D texture;
    varying vec4 vColor;
    ${THREE.ShaderChunk[ "common" ]}
    ${THREE.ShaderChunk[ "fog_pars_fragment" ]}
    
    void main() {
        gl_FragColor = vec4(color,1.0) * vColor;
        gl_FragColor = gl_FragColor * texture2D( texture, gl_PointCoord );
        if ( gl_FragColor.a < ALPHATEST ) discard;
        ${THREE.ShaderChunk[ "fog_fragment" ]}
    
    }
`;