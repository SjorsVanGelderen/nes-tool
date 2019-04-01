// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
};

use crate::media;
use crate::palette;
use crate::surface::Surface;
use crate::vertex::Vertex;

use std::{
    boxed::Box,
    marker::{
        Send,
        Sync,
    },
    path::Path,
    sync::Arc,
};

use vulkano::{
    buffer::{
        BufferUsage,
        cpu_access::CpuAccessibleBuffer,
    },
    command_buffer::{
        AutoCommandBuffer,
        CommandBufferExecFuture,
    },
    descriptor::{
        descriptor_set::{
            DescriptorSet,
            PersistentDescriptorSet,
            // PersistentDescriptorSetImg,
            // PersistentDescriptorSetSampler,
        },
        PipelineLayoutAbstract,
    },
    device::{
        Device,
        Queue,
    },
    format::Format,
    framebuffer::{
        RenderPassAbstract,
        Subpass,
    },
    image::{
        Dimensions,
        ImmutableImage,
    },
    pipeline::{
        GraphicsPipeline,
        vertex::{
            SingleBufferDefinition
        },
    },
    sampler::Sampler,
    sync::NowFuture,
};

type PatternTableGraphicsPipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<(dyn PipelineLayoutAbstract + Send + Sync + 'static)>,
        Arc<(dyn RenderPassAbstract + Send + Sync + 'static)>
    >
>;

pub struct PatternTable {
    pub bytes: [u8; 8192],
    pub pixels: [u8; 32768],
    pub surface: Surface,
    pub vertex_shader: vs::Shader,
    pub fragment_shader: fs::Shader,
    pub pipeline: PatternTableGraphicsPipeline,
    pub texture: Arc<ImmutableImage<Format>>,
    pub tex_future: CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
    pub descriptor_set: Arc<(dyn DescriptorSet + Send + Sync + 'static)>,
}

impl PatternTable {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        sampler: Arc<Sampler>
    ) -> Self {
        let bytes = [0; 8192];
        let pixels = [0; 32768];
        let surface = Self::get_surface(device.clone());
        let vertex_shader = vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Failed to create fragment shader");
        let pipeline = Self::get_pipeline(device.clone(), &vertex_shader, &fragment_shader, render_pass.clone());

        // Arguably redundant
        let (texture, tex_future) = Self::get_texture_and_future(queue.clone(), &pixels);
        let descriptor_set = Self::get_descriptor_set(pipeline.clone(), texture.clone(), sampler.clone());

        Self {
            bytes,
            pixels,
            surface,
            vertex_shader,
            fragment_shader,
            pipeline,
            texture,
            tex_future,
            descriptor_set,
        }
    }

    pub fn load_from_file(
        self, device: Arc<Device>, path: &Path, queue: Arc<Queue>, sampler: Arc<Sampler>
    ) -> Self {
        let (bytes, pixels) = match media::load_pattern_table_bytes_and_pixels(path) {
            Ok(result) => result,
            Err(_) => panic!("Failed to load bytes and pixels for pattern table!")
        };

        let (texture, tex_future) = Self::get_texture_and_future(queue.clone(), &pixels);
        let descriptor_set = Self::get_descriptor_set(self.pipeline.clone(), texture.clone(), sampler.clone());

        Self {
            bytes,
            pixels,
            texture,
            tex_future,
            descriptor_set,
            ..self
        }
    }

    // pub fn set_surface(self, surface: Surface) {
    //     Self {
    //         surface,
    //         ..self
    //     }
    // }

    fn get_surface(device: Arc<Device>) -> Surface {
        Surface::new(device.clone(), Vector3::new(0.0, 0.0, 3.0), Vector2::new(200.0, 100.0))
    }

    fn get_pipeline(
        device: Arc<Device>,
        vertex_shader: &vs::Shader,
        fragment_shader: &fs::Shader,
        render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> PatternTableGraphicsPipeline {
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vertex_shader.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fragment_shader.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap()
        )
    }

    fn get_texture_and_future(queue: Arc<Queue>, pixels: &[u8; 32768]) -> (
        Arc<ImmutableImage<Format>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>
    ) {
        let mut image_data: [u8; 32768] = [0u8; 32768];
        
        for (i, x) in pixels.iter().enumerate() {
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

    fn get_descriptor_set(
        // device: Arc<Device>,
        pipeline: PatternTableGraphicsPipeline,
        texture: Arc<ImmutableImage<Format>>,
        sampler: Arc<Sampler>
    ) -> Arc<(dyn DescriptorSet + Send + Sync + 'static)> {
        // let full_palette_buffer = CpuAccessibleBuffer::from_data(
        //     device.clone(), BufferUsage::uniform_buffer(), palette::FULL_PALETTE
        // ).expect("Failed to create buffer!");

        // let samples_color_indices_buffer = CpuAccessibleBuffer::from_data(
        //     device.clone(), BufferUsage::uniform_buffer(), FIX_ME
        // ).expect("Failed to create buffer!");
        
        Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
            // .add_buffer(full_palette_buffer).unwrap()
            // .add_buffer(samples_color_indices_buffer).unwrap();
            .build().unwrap()
        )
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

layout(push_constant) uniform push_constants {
    mat4 mvp;
    vec2 mouse;
    uint active_sample;
} pc;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out vec2 mouse_out;
layout(location = 2) out uint active_sample_out;

void main() {
    gl_Position = pc.mvp * vec4(position, 1);

    uv_out = uv;
    mouse_out = pc.mouse;
    active_sample_out = pc.active_sample;
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
layout(location = 1) in vec2 mouse;
layout(location = 2) in flat uint active_sample;

layout(set = 0, binding = 0) uniform sampler2D tex;
// layout(set = 0, binding = 1) uniform UBO {
    // uvec4 full_palette[64];
//     uvec4 samples_color_indices[8];
// } ubo;

layout(location = 0) out vec4 color;

// uint sample_index = active_sample / 3 * 3;
// uvec4 sample_data = ubo.samples_color_indices[sample_index];
// uint tone = uint(texture(tex, uv).x * 4.0);

void main() {
    color = mouse.xxxx; // dummy
    // color = ubo.full_palette[sample_data[tone]];
}
"
    }
}