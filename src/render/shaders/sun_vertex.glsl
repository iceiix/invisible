in vec3 aPosition;
in int id;

uniform mat4 perspectiveMatrix;
uniform mat4 cameraMatrix;
uniform mat4 modelMatrix[10];

void main() {
	vec3 pos = vec3(aPosition.x, -aPosition.y, aPosition.z);
	gl_Position = perspectiveMatrix * cameraMatrix * modelMatrix[id] * vec4(pos, 1.0);
}
