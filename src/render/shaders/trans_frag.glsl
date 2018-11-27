uniform sampler2D taccum;
uniform sampler2DMS tcolor;

out vec4 fragColor;

void main() {
    ivec2 C = ivec2(gl_FragCoord.xy);
    vec4 accum = texelFetch(taccum, C, 0);
    vec4 col = texelFetch(tcolor, C, 0);

    float r = accum.a;
    accum.a = 1.0;
    if (r >= 1.0) {
        fragColor = vec4(col.rgb, 0.0);
    } else {
        fragColor = vec4(col.rgb, 0.0);
    }
}
