uniform vec3 color;
uniform sampler2D texture;
varying vec4 vColor;

//${THREE.ShaderChunk[ "common" ]}
//${THREE.ShaderChunk[ "fog_pars_fragment" ]}

void main() {
    gl_FragColor = vec4(color,1.0) * vColor;
    gl_FragColor = gl_FragColor * texture2D( texture, gl_PointCoord );
    if ( gl_FragColor.a < ALPHATEST ) discard;
    // ${THREE.ShaderChunk[ "fog_fragment" ]}
}
