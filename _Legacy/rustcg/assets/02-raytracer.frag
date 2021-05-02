#version 330 core

uniform sampler2D tex;

in vec2 vUV;

out vec4 Target0;

void main() {
    Target0 = texture(tex, vUV);
}
