in vec3 aPosition;

uniform mat4 perspectiveMatrix;
uniform mat4 cameraMatrix;
uniform mat4 modelMatrix;

void main() {
	vec3 pos = vec3(aPosition.x, -aPosition.y, aPosition.z);
	gl_Position = perspectiveMatrix * cameraMatrix * modelMatrix * vec4(pos, 1.0);
}
