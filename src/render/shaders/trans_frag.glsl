uniform sampler2DMS tcolor;

out vec4 fragColor;

void main() {
    fragColor = texelFetch(tcolor, ivec2(gl_FragCoord.xy), 0);
}
