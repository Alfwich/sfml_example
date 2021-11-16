#version 330 core

in vec2 uv;

out vec4 FragColor;

uniform sampler2D tex1;

void main()
{
    FragColor = texture(tex1, uv).xxxx;
} 