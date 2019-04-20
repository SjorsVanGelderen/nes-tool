// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
};

use crate::palette::FULL_PALETTE;
use crate::surface::Surface;
use crate::vertex::Vertex;

use std::{
    boxed::Box,
    marker::{
        Send,
        Sync,
    },
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
            SingleBufferDefinition,
        }
    },
    sampler::Sampler,
    sync::NowFuture,
};

type SamplesGraphicsPipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<(dyn PipelineLayoutAbstract + Send + Sync + 'static)>,
        Arc<(dyn RenderPassAbstract + Send + Sync + 'static)>
    >
>;

pub struct Samples {
    pub color_indices: [u8; 26],
    pub descriptor_set: Option<Arc<(dyn DescriptorSet + Send + Sync + 'static)>>,
    pub fragment_shader: fs::Shader,
    pub pipeline: SamplesGraphicsPipeline,
    pub surface: Surface,
    pub texture: Option<Arc<ImmutableImage<Format>>>,
    pub vertex_shader: vs::Shader,
}

impl Samples {
    pub fn new(
        device: Arc<Device>, render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> Self {
        let color_indices = [0u8; 26];
        let surface = Self::get_surface(device.clone());
        let vertex_shader = vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Failed to create fragment shader");
        let pipeline = Self::get_pipeline(device.clone(), &vertex_shader, &fragment_shader, render_pass.clone());
        let descriptor_set = None;
        let texture = None;

        Self {
            color_indices,
            descriptor_set,
            fragment_shader,
            pipeline,
            surface,
            texture,
            vertex_shader,
        }
    }

    pub fn get_texture_and_future(self, queue: Arc<Queue>, color_indices: &[u8; 26]) -> (
        Self, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
    ) {
        let image_data: Vec<u8> = color_indices.iter().flat_map(
            |x| {
                let index = (*x * 3) as usize;
                vec![FULL_PALETTE[index], FULL_PALETTE[index + 1], FULL_PALETTE[index + 2], 255u8]
            }
        ).collect();

        let (tex, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 13, height: 2 },
            Format::R8G8B8A8Unorm,
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

    pub fn set_position(self, position: Vector3<f32>) -> Self {
        let surface = Surface {
            position,
            ..self.surface
        };

        Self {
            surface,            
            ..self
        }
    }

    // pub fn set_color_index(self, which: usize, to_color_index: u8) -> Self {
    //     let mut color_indices = self.color_indices;
    //     color_indices[which] = to_color_index;

    //     Self {
    //         color_indices,
    //         ..self
    //     }
    // }

    fn get_surface(device: Arc<Device>) -> Surface {
        Surface::new(device.clone(), Vector3::new(0.0, 0.0, 2.0), Vector2::new(52.0, 8.0))
    }

    fn get_pipeline(
        device: Arc<Device>,
        vertex_shader: &vs::Shader,
        fragment_shader: &fs::Shader,
        render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> SamplesGraphicsPipeline {
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
} pc;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out vec2 mouse_out;

void main() {
    gl_Position = pc.mvp * vec4(position, 1.0);

    uv_out = uv;
    mouse_out = pc.mouse;
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

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) out vec4 color;

vec2 total_size = vec2(13.0, 2.0);
vec2 color_square_size = vec2(1.0 / total_size.x, 1.0 / total_size.y);

vec2 color_center = vec2(
    floor(mouse.x / color_square_size.x) * color_square_size.x + color_square_size.x / 2.0,
    floor(mouse.y / color_square_size.y) * color_square_size.y + color_square_size.y / 2.0
);

void main() {
    if( abs(uv.x - color_center.x) < color_square_size.x / 1.5
     && abs(uv.y - color_center.y) < color_square_size.y / 1.5
      )
    {
        color = vec4(texture(tex, color_center).xyz, 1.0);
    }
    else
    {
        color = vec4(texture(tex, uv).xyz, 1.0);
    }
}
"
    }
}