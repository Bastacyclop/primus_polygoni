#version 150 core

in vec2 v_TexCoord;

out vec4 Target0;

uniform sampler2D t_Color;

void main() {
    Target0 = texture(t_Color, v_TexCoord);
}