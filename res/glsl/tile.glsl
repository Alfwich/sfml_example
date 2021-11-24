#version 330 core

in vec2 uv;
in float out_border;

out vec4 FragColor;

uniform sampler2D tex1;

void main()
{
    if (uv.x < out_border || uv.x > 1 - out_border || uv.y < out_border || uv.y > 1 - out_border) {
        FragColor = vec4(1.0);
    } else {
        FragColor = texture(tex1, uv);
    }
} 