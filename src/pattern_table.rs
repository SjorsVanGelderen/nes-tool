use cgmath::Vector2;

use crate::surface::Surface;

pub struct PatternTable {
    pub bytes: [u8; 8192],
    pub pixels: [u8; 32768],
}

impl PatternTable {
    pub fn zero() -> PatternTable {
        PatternTable {
            bytes: [0; 8192],
            pixels: [0; 32768],
        }
    }
}

pub fn surface_zero() -> Surface {
    Surface::zero(Vector2::new(0.0, 0.0), Vector2::new(200.0, 100.0))
}

pub mod vs {
    vulkano_shaders::shader!{
    ty: "vertex",
    src:
"
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

layout(set = 0, binding = 0) uniform UniformBufferObject
{
    mat4 mvp;
} ubo;

layout(location = 0) out vec2 uv_out;

void main() {
    gl_Position = ubo.mvp * vec4(position, 1);

    uv_out = uv;
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src:
"
#version 450

layout(location = 0) in vec2 uv;

layout(set = 0, binding = 1) uniform sampler2D tex; 

layout(location = 0) out vec4 color;

void main() {
    color = vec4(texture(tex, uv).xxx, 1.0);
}
"
    }
}