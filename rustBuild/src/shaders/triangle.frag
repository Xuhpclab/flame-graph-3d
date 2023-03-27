#version 300 es
precision mediump float;
in vec3 position;
in vec4 v_color;
out vec4 color;
void main() {
    color = vec4( v_color);
}