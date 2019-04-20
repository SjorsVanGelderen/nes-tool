// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
};

use crate::media;
use crate::surface::Surface;
use crate::system;
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
    command_buffer::{
        AutoCommandBuffer,
        CommandBufferExecFuture,
    },
    descriptor::{
        descriptor_set::{
            DescriptorSet,
            PersistentDescriptorSet,
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
    pub descriptor_set: Option<Arc<(dyn DescriptorSet + Send + Sync + 'static)>>,
    pub fragment_shader: fs::Shader,
    pub pipeline: PatternTableGraphicsPipeline,
    pub surface: Surface,
    pub texture: Option<Arc<ImmutableImage<Format>>>,
    pub vertex_shader: vs::Shader,
}

impl PatternTable {
    pub fn new(
        device: Arc<Device>, render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> Self {
        let bytes = [0; 8192];
        let surface = Self::get_surface(device.clone());
        let vertex_shader = vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Failed to create fragment shader");
        let pipeline = Self::get_pipeline(device.clone(), &vertex_shader, &fragment_shader, render_pass.clone());
        let descriptor_set = None;
        let texture = None;

        Self {
            bytes,
            descriptor_set,
            fragment_shader,
            pipeline,
            surface,
            texture,
            vertex_shader,
        }
    }

    pub fn load_from_file(
        self, path: &Path, queue: Arc<Queue>
    ) -> (
        Self, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>
    ) {
        let (_bytes, pixels) = match media::load_pattern_table_bytes_and_pixels(path) {
            Ok(result) => result,
            Err(_) => panic!("Failed to load bytes and pixels for pattern table!")
        };

        self.get_texture_and_future(queue.clone(), &pixels)
    }

    pub fn get_texture_and_future(self, queue: Arc<Queue>, pixels: &[u8; 32768]) -> (
        Self, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>
    ) {
        let mut image_data: [u8; 32768] = [0u8; 32768];
        
        for (i, x) in pixels.iter().enumerate() {
            let pixel: u8 = (*x as f32 * (255.0 / 4.0)) as u8;

            image_data[i] = pixel;
        }

        let (tex, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 256, height: 128 },
            Format::R8Unorm,
            queue.clone()
        ).unwrap();

        (
            Self {
                texture: Some(tex),
                ..self
            },
            tex_future
        )
    }

    pub fn get_descriptor_set(
        self,
        sampler: Arc<Sampler>
    ) -> Self {
        let texture = self.texture.clone().unwrap();

        let descriptor_set: Option<Arc<(dyn DescriptorSet + Send + Sync + 'static)>> =
            Some(
                Arc::new(
                    PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                    .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
                    .build().unwrap()
                )
            );

        Self {
            descriptor_set,
            ..self
        }
    }

    pub fn click(&self, mouse_position: Vector2<f32>) -> bool {
        let click_position = system::get_mouse_position_on_surface(
            mouse_position,
            Vector2::new(
                self.surface.position.x,
                self.surface.dimensions.x
            ),
            self.surface.dimensions
        );

        println!("{:?}", click_position);

        click_position.x >= 0.0 && click_position.y >= 0.0
    }

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
    float sample_colors[12];
    uint active_sample;
} pc;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out vec2 mouse_out;
layout(location = 2) out uint active_sample_out;
layout(location = 3) out float sample_colors_out[12];

void main() {
    gl_Position = pc.mvp * vec4(position, 1);

    uv_out = uv;
    mouse_out = pc.mouse;
    active_sample_out = pc.active_sample;

    // Copy operation fails to compile, so here I'm doing it manually
    // sample_colors_out = pc.sample_colors;
    for(int i = 0; i < 12; i++)
    {
        sample_colors_out[i] = pc.sample_colors[i];
    }
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
layout(location = 3) in float sample_colors[12];

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(sample_colors[0]); // dummy
    color = mouse.xxxx; // dummy
    color = texture(tex, uv).xxxx;
}
"
    }
}