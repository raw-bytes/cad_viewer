
//------------------------------------------
// INPUTS
//------------------------------------------

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform mat4 combinedMat;
uniform mat3 normalMat;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec3 varNormal;

//------------------------------------------
// CONSTANTS
//------------------------------------------

void main() {
    // transform normal
    vec3 normal = inNormal;
    varNormal = normalMat * normal;

    // project 
    vec4 ppos = combinedMat * vec4(inPosition, 1.0);

    // transform vertex position
    gl_Position = ppos;
}
