#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buffer;

void main() {
    uint index = gl_GlobalInvocationID.x;
    buffer.data[index] *= 12;
}