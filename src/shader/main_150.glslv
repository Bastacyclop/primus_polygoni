#version 150 core

in vec4 a_Pos;
in vec2 a_TexCoord;
in vec4 a_T1;
in vec4 a_T2;
in vec4 a_T3;
in vec4 a_T4;

out vec2 v_TexCoord;

uniform Locals {
    mat4 u_Transform;
};

void main() {
    gl_Position = u_Transform * mat4(a_T1, a_T2, a_T3, a_T4) * a_Pos;
    v_TexCoord = a_TexCoord;
    // gl_ClipDistance[0] = 1.0;
}