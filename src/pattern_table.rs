// Copyright 2019, Sjors van Gelderen

use cgmath::Vector2;

use crate::media;
use crate::surface::Surface;
use crate::vertex::Vertex;

use std::{
    path::Path,
    sync::Arc,
};

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer
    },
    command_buffer::{
        AutoCommandBuffer,
        CommandBufferExecFuture,
    },
    device::{
        Device,
        Queue,
    },
    format::Format,
    image::{
        Dimensions,
        ImmutableImage,
    },
    sync::NowFuture,
};

pub struct PatternTable {
    pub bytes: [u8; 8192],
    pub pixels: [u8; 32768],
    pub surface: Surface,
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    pub vertex_shader: vs::Shader,
    pub fragment_shader: fs::Shader,
}

impl PatternTable {
    pub fn zero(device: Arc<Device>) -> Self {
        let bytes = [0; 8192];
        let pixels = [0; 32768];
        let surface = PatternTable::get_surface();
        let vertex_buffer = PatternTable::get_vertex_buffer(device.clone(), &surface);
        let index_buffer = PatternTable::get_index_buffer(device.clone(), &surface);
        let vertex_shader = vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Failed to create fragment shader");

        PatternTable {
            bytes,
            pixels,
            surface,
            vertex_buffer,
            index_buffer,
            vertex_shader,
            fragment_shader,
        }
    }

    pub fn load_from_file(self, path: &Path) -> Self {
        let (bytes, pixels) = match media::load_pattern_table_bytes_and_pixels(path) {
            Ok(result) => result,
            Err(_) => panic!("Failed to load bytes and pixels for pattern table!")
        };

        PatternTable {
            bytes,
            pixels,
            ..self
        }
    }

    pub fn get_texture_and_future(&self, queue: Arc<Queue>) -> (Arc<ImmutableImage<Format>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
        let mut image_data: [u8; 32768] = [0u8; 32768];
        
        for (i, x) in (*self).pixels.iter().enumerate() {
            let pixel: u8 = (*x as f32 * (255.0 / 4.0)) as u8;

            image_data[i] = pixel;
        }

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 256, height: 128 },
            Format::R8Unorm,
            queue.clone()
        ).unwrap()
    }

    fn get_surface() -> Surface {
        Surface::zero(Vector2::new(0.0, 0.0), Vector2::new(200.0, 100.0))
    }

    // TODO: Find alternative to CpuAccessibleBuffer as it will be deprecated soon
    fn get_vertex_buffer(device: Arc<Device>, surface: &Surface) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(), 
            BufferUsage::all(),
            surface.vertices.iter().cloned()
        ).unwrap()
    }

    fn get_index_buffer(device: Arc<Device>, surface: &Surface) -> Arc<CpuAccessibleBuffer<[u32]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            surface.indices.iter().cloned()
        ).unwrap()
    }
}

pub mod vs {
    vulkano_shaders::shader!{
    ty: "vertex",
    src:
"
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

// layout(set = 0, binding = 0) uniform UniformBufferObject
// {
//     mat4 mvp;
// } ubo;

layout(push_constant) uniform Matrices {
    mat4 mvp;
} matrices;

layout(location = 0) out vec2 uv_out;

void main() {
    gl_Position = matrices.mvp * vec4(position, 1);

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

layout(set = 0, binding = 0) uniform sampler2D tex; 

layout(location = 0) out vec4 color;

void main() {
    color = vec4(texture(tex, uv).xxx, 1.0);
}
"
    }
}