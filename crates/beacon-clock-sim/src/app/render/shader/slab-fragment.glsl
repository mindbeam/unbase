//precision mediump float;
//uniform vec3 color;
//uniform sampler2D disc_texture;
//varying vec3 vColor;
//${THREE.ShaderChunk[ "common" ]}
//${THREE.ShaderChunk[ "fog_pars_fragment" ]}
//void main() {
//    gl_FragColor = vec4(1,0,0,1);
    //gl_FragColor = vec4( color, 1.0 );
    //gl_FragColor = gl_FragColor * texture2D( disc_texture, gl_PointCoord );
    //if ( gl_FragColor.a < ALPHATEST ) discard;
    // ${THREE.ShaderChunk[ "fog_fragment" ]}
//}

void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}