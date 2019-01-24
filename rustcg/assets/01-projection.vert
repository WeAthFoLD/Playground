#version 330 core

uniform Transform {
    mat4 uMat;
};

layout (location = 0) in vec3 aPos;

out vec4 vColor;

void main() {
    gl_Position = uMat * vec4(aPos, 1);
    vColor = vec4(aPos.xy + vec2(0.5, 0.5), 0.8, 1);
}
