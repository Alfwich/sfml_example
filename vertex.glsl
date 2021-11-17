#version 330 core
layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec2 in_uv;

out vec2 uv;
out float out_border;

uniform mat4 mvp;
uniform float border;

void main()
{
   gl_Position = mvp * vec4(in_pos.xyz, 1.0);
   uv = in_uv;
   out_border = border;
}