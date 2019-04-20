// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
};

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

type PaletteGraphicsPipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<(dyn PipelineLayoutAbstract + Send + Sync + 'static)>,
        Arc<(dyn RenderPassAbstract + Send + Sync + 'static)>
    >
>;

pub struct Palette {
    pub descriptor_set: Option<Arc<(dyn DescriptorSet + Send + Sync + 'static)>>,
    pub fragment_shader: fs::Shader,
    pub pipeline: PaletteGraphicsPipeline,
    pub surface: Surface,
    pub texture: Option<Arc<ImmutableImage<Format>>>,
    pub vertex_shader: vs::Shader,
}

impl Palette {
    pub fn new(
        device: Arc<Device>, render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> Self {
        let surface = Self::get_surface(device.clone());
        let vertex_shader = vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
        let fragment_shader = fs::Shader::load(device.clone()).expect("Failed to create fragment shader");
        let pipeline = Self::get_pipeline(device.clone(), &vertex_shader, &fragment_shader, render_pass.clone());
        let descriptor_set = None;
        let texture = None;

        Self {
            descriptor_set,
            fragment_shader,
            pipeline,
            surface,
            texture,
            vertex_shader,
        }
    }

    pub fn get_texture_and_future(self, queue: Arc<Queue>) -> (
        Self,
        CommandBufferExecFuture<NowFuture, AutoCommandBuffer>
    ) {
        let image_data: Vec<u8> = FULL_PALETTE.chunks(3).flat_map(
            |x| vec![x[0], x[1], x[2], 255u8]
        ).collect();

        let (tex, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 16, height: 4 },
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

    fn get_surface(device: Arc<Device>) -> Surface {
        Surface::new(device.clone(), Vector3::new(0.0, 0.0, 1.0), Vector2::new(64.0, 16.0))
    }

    fn get_pipeline(
        device: Arc<Device>,
        vertex_shader: &vs::Shader,
        fragment_shader: &fs::Shader,
        render_pass: Arc<RenderPassAbstract + Send + Sync>
    ) -> PaletteGraphicsPipeline {
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

vec2 total_size = vec2(16.0, 4.0);
vec2 color_square_size = vec2(1.0 / total_size.x, 1.0 / total_size.y);

vec2 color_center = vec2(
    floor(mouse.x / color_square_size.x) * color_square_size.x + color_square_size.x / 2.0,
    floor(mouse.y / color_square_size.y) * color_square_size.y + color_square_size.y / 2.0
);

vec2 uv_to_color_center = vec2(abs(uv.x - color_center.x), abs(uv.y - color_center.y));

// TODO: Use this to provide a bit of animation
// vec2 mouse_to_color_center = mouse - color_center;

void main() {
    if ( uv_to_color_center.x < color_square_size.x
      && uv_to_color_center.y < color_square_size.y
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

pub static FULL_PALETTE: [u8; 192] = [ 
    101u8, 101u8, 101u8,
      3u8,  47u8, 103u8,
     21u8,  35u8, 125u8,
     60u8,  26u8, 122u8,
     95u8,  18u8,  97u8,
    114u8,  14u8,  55u8,
    112u8,  16u8,  13u8,
     89u8,  26u8,   5u8,
     52u8,  40u8,   3u8,
     13u8,  51u8,   3u8,
      3u8,  59u8,   4u8,
      4u8,  60u8,  19u8,
      3u8,  56u8,  63u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,
  
    174u8, 174u8, 174u8,
     15u8,  99u8, 179u8,
     64u8,  81u8, 208u8,
    120u8,  65u8, 204u8,
    167u8,  54u8, 169u8,
    192u8,  52u8, 112u8,
    189u8,  60u8,  48u8,
    159u8,  74u8,   0u8,
    109u8,  92u8,   0u8,
     54u8, 109u8,   0u8,
      7u8, 119u8,   4u8,
      0u8, 121u8,  61u8,
      0u8, 114u8, 125u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,

    254u8, 254u8, 255u8,
     93u8, 179u8, 255u8,
    143u8, 161u8, 255u8,
    200u8, 144u8, 255u8,
    247u8, 133u8, 250u8,
    255u8, 131u8, 192u8,
    255u8, 138u8, 127u8,
    239u8, 154u8,  73u8,
    189u8, 172u8,  44u8,
    133u8, 188u8,  47u8,
     85u8, 199u8,  83u8,
     60u8, 201u8, 140u8,
     62u8, 194u8, 205u8,
     78u8,  78u8,  78u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,

    254u8, 254u8, 255u8,
    188u8, 223u8, 255u8,
    209u8, 216u8, 255u8,
    232u8, 209u8, 255u8,
    251u8, 205u8, 253u8,
    255u8, 204u8, 229u8,
    255u8, 207u8, 202u8,
    248u8, 213u8, 180u8,
    228u8, 220u8, 168u8,
    204u8, 227u8, 169u8,
    185u8, 232u8, 184u8,
    174u8, 232u8, 208u8,
    175u8, 229u8, 234u8,
    182u8, 182u8, 182u8,
      0u8,   0u8,   0u8,
      0u8,   0u8,   0u8,
];