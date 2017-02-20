#version 150 core

in vec4 a_Pos;

uniform Locals {
    mat4 u_Transform;
};

void main() {
    gl_Position = u_Transform * a_Pos;
    // gl_ClipDistance[0] = 1.0;
}