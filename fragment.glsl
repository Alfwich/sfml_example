#version 330 core

in vec4 gl_FragCoord;
out vec4 FragColor;

void main()
{
    FragColor = vec4(cos(gl_FragCoord.x), sin(gl_FragCoord.x), cos(gl_FragCoord.x) + sin(gl_FragCoord.y), 1.0f);
} 