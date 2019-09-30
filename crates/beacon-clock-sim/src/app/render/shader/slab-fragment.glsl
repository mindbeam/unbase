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
//precision highp float;
//void main() {
//    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    //gl_FragColor = gl_FragColor * texture2D( disc_texture, gl_PointCoord );
//}

#ifdef GL_OES_standard_derivatives
#extension GL_OES_standard_derivatives : enable
#endif

precision mediump float;
varying  vec4 color;

void main() {

     float r = 0.0, delta = 0.0, alpha = 1.0;
     vec2 cxy = 2.0 * gl_PointCoord - 1.0;
     r = dot(cxy, cxy);

//    if (r > 1.1) {
//        discard;
//        return;
//    }

 #ifdef GL_OES_standard_derivatives
     delta = fwidth(r);
     alpha = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, r);
 #endif

 gl_FragColor = color * alpha;



}