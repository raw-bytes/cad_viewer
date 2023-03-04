
//------------------------------------------
// INPUTS
//------------------------------------------

in vec3 varNormal;
in vec3 varPos;

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform vec3 diffuseColor;
uniform int normalsEnabled;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec4 outColor;

//------------------------------------------
// CONSTANTS
//------------------------------------------

vec3 flatNormal(vec3 pos) {
    vec3 fdx = dFdx(pos);
    vec3 fdy = dFdy(pos);
    return normalize(cross(fdx, fdy));
}

void main() {
    vec3 normal;

    if(normalsEnabled == 1) {
        normal = normalize(varNormal);
    } else {
        normal = flatNormal(varPos);
    }

    float f = abs(normal.z) * 0.75 + 0.25;

    outColor = vec4(f * diffuseColor, 1.0);
}