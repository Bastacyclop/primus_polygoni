#version 150 core

in vec4 a_Pos;
in vec3 a_Color;

out vec3 v_Color;

uniform Locals {
    mat4 u_Transform;
};

void main() {
    gl_Position = u_Transform * a_Pos;
    v_Color = a_Color;
    // gl_ClipDistance[0] = 1.0;
}