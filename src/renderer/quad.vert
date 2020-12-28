#version 450

layout(location=0) in vec2 s_position;
layout(location=1) in vec2 q_position;
layout(location=2) in vec2 q_size;
layout(location=3) in vec4 q_colour;

layout(location=0) out vec4 v_colour;

void main() {
    v_colour = q_colour;
    gl_Position = vec4(s_position * q_size + q_position, 0.0, 1.0);
}