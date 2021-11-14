#version 330 core

in vec3 color;
in vec2 uv;

out vec4 FragColor;

uniform sampler2D tex1;

void main()
{
    FragColor = texture(tex1, uv) * vec4(color.xyz, 1.0);
} 