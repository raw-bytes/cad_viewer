
//------------------------------------------
// INPUTS
//------------------------------------------

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform mat4 combinedMat;
uniform mat4 modelMat;
uniform mat3 normalMat;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec3 varNormal;
out vec3 varPos;

//------------------------------------------
// CONSTANTS
//------------------------------------------

void main() {
    // transform normal
    vec3 normal = inNormal;
    varNormal = normalMat * normal;

    // apply model view matrix
    varPos = vec3(modelMat * vec4(inPosition, 1.0));

    // project 
    vec4 ppos = combinedMat * vec4(inPosition, 1.0);

    // transform vertex position
    gl_Position = ppos;
}
