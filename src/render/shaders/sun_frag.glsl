uniform sampler2DArray textures;
uniform vec4 colorMul[10];

in vec4 vTextureInfo;
in vec2 vTextureOffset;
in float vAtlas;
in float vID;

out vec4 fragColor;

void main() {
	fragColor=vec4(1.0, 1.0, 1.0, 1.0);
}
