#version 300 es
in vec3 in_position;
in vec4 in_color;

out vec3 position;
out vec4 v_color;

uniform mat4 transform;

void main() {
    position = in_position;
    v_color = in_color;
    //gl_Position = vec4(in_position, 1.0);
    gl_Position = transform * vec4(in_position, 1.0);
}