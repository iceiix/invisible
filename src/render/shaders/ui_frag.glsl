uniform sampler2DArray textures;

in vec4 vColor;
in vec4 vTextureInfo;
in vec2 vTextureOffset;
in float vAtlas;

out vec4 fragColor;

void main() {
	fragColor = vec4(0.2, 0.2, 0.2, 1.0);
}
