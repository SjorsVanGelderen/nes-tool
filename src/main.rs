// Copyright 2019, Sjors van Gelderen

extern crate cgmath;
extern crate image;
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use crate::attribute_table::AttributeTable;
use crate::character::Character;
use crate::nametable::Nametable;
use crate::samples::Samples;
use crate::surface::Surface;
use crate::vertex::Vertex;

mod attribute_table;
mod character;
mod media;
mod mode;
mod nametable;
mod palette;
mod samples;
mod surface;
mod tool;
mod vertex;

use cgmath::{
    Matrix4,
    Point3,
    SquareMatrix,
    Vector2,
    Vector3,
    ortho,
};

// use image::{
//     ImageBuffer,
//     Rgba,
// };

use std::path::Path;

use std::sync::Arc;

use vulkano::buffer::{
    BufferUsage,
    CpuAccessibleBuffer,
};

use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBuffer,
    DynamicState,
};

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::device::{
    Device,
    DeviceExtensions,
    Features,
    Queue,
    QueuesIter,
};

use vulkano::format::{
    Format,
    ClearValue,
};

use vulkano::framebuffer::{
    Framebuffer,
    FramebufferAbstract,
    RenderPassAbstract,
    Subpass,
};

use vulkano::image::{
    Dimensions,
    ImmutableImage,
    StorageImage,
    SwapchainImage,
};

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    PhysicalDevice,
    QueueFamily,
};

use vulkano::pipeline::{
    ComputePipeline,
    GraphicsPipeline,
    viewport::Viewport,
};

use vulkano::sampler::{
    Sampler,
    SamplerAddressMode,
    Filter,
    MipmapMode
};

use vulkano::swapchain::{
    AcquireError,
    PresentMode,
    SurfaceTransform,
    Swapchain,
    SwapchainCreationError,
    acquire_next_image,
};

use vulkano::sync::{
    FlushError,
    GpuFuture,
};

use vulkano_win::VkSurfaceBuild;

use winit::{
    Event,
    EventsLoop,
    Window,
    WindowBuilder,
    WindowEvent,
    KeyboardInput,
    VirtualKeyCode,
};

// mod cs {
//     vulkano_shaders::shader!{
//         ty: "compute",
//         src:
// "
// #version 450

// layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

// layout(set = 0, binding = 0) buffer Data {
//     uint data[];
// } buf;

// void main() {
//     uint index = gl_GlobalInvocationID.x;
//     buf.data[index] *= 12;
// }
// "
//     }
// }

mod vs {
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

// TODO: Add uniform sampler here

layout(location = 0) out vec2 uv_out;

void main() {
    // gl_Position = vec4(uv, uv); // Just so the compiler won't complain about uv
    gl_Position = ubo.mvp * vec4(position, 1);

    uv_out = uv;
}
"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src:
"
#version 450

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(1.0, uv.x, uv.y, 1.0);
    // color = vec4(1.0, 0.0, 1.0, 1.0);
}
"
    }
}

fn get_physical_device(i: &Arc<Instance>) -> PhysicalDevice {
    PhysicalDevice::enumerate(i).next()
        .expect("No device found")
}

// fn enumerate_queues(p: PhysicalDevice) {
//     for family in p.queue_families() {
//         println!("Found a queue family with {:?} queue(s)", family.queues_count());
//     }
// }

fn get_queue_family(p: PhysicalDevice) -> QueueFamily {
    p.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("No graphical queues found")
}

fn get_device_and_queues(p: PhysicalDevice, q: QueueFamily) -> (Arc<Device>, QueuesIter) {
    let extensions = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        .. vulkano::device::DeviceExtensions::none()
    };

    Device::new(
        p,
        p.supported_features(),
        &extensions,
        [(q, 0.5)].iter().cloned()
    ).expect("Failed to create device")
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState
) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };

    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}

fn main() {
    // TODO: Separate this into functions

    let character = media::load_character(Path::new("./graphics.chr"));

    let instance = {
        let extensions = vulkano_win::required_extensions();
        
        Instance::new(None, &extensions, None).unwrap()
            //.expect("Failed to create instance")
    };

    let physical = get_physical_device(&instance);

    // enumerate_queues();

    let mut events_loop = EventsLoop::new();

    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let window = surface.window();

    let queue_family = get_queue_family(physical);

    let (device, mut queues) = get_device_and_queues(physical, queue_family);

    let queue = queues.next().unwrap();

    // simple_cpu_buffer_example(device.clone(), queue.clone());

    // compute_shader_example(device.clone(), queue.clone());

    // image_example(device.clone(), queue.clone());

    // render_example(device.clone(), queue.clone());

    let (mut swapchain, images) = {
        let capabilities = surface.capabilities(physical).unwrap();
            // .expect("Failed to get surface capabilities");

        let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();

        let format = capabilities.supported_formats[0].0;

        let dimensions = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions
                .to_physical(window.get_hidpi_factor())
                .into();
            
            [dimensions.0, dimensions.1]
        }
        else {
            return;
        };

        Swapchain::new(
            device.clone(),
            surface.clone(),
            capabilities.min_image_count,
            format,
            dimensions,
            1,
            capabilities.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None
        ).unwrap() //.expect("Failed to create swapchain")
    };

    let my_surface = Surface::zero(Vector2::new(0.0, 0.0), Vector2::new(50.0, 50.0));

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(), 
        BufferUsage::all(),
        my_surface.vertices.iter().cloned()
    ).unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        my_surface.indices.iter().cloned()
    ).unwrap();

    let vs = vs::Shader::load(device.clone()).unwrap(); //.expect("Failed to create vertex shader");
    let fs = fs::Shader::load(device.clone()).unwrap(); //.expect("Failed to create fragment shader");

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap()
    );

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
    );

    let projection: Matrix4<f32> = ortho(
        -100.0, 100.0,
        -100.0, 100.0,
        0.01, 100.0
    );
    
    let view: Matrix4<f32> = Matrix4::look_at(
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0)
    );

    let model: Matrix4<f32> = Matrix4::identity();

    let mvp: Matrix4<f32> = model * view * projection;

    let data_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        mvp
    ).expect("Failed to create buffer");

    // let (texture, tex_future) = {
    //     let image = image::load_from_memory_with_format(
    //         include_bytes!("image.png"),
    //         ImageFormat::PNG
    //     ).unwrap().to_rgba();

    //     let image_data = image.into_raw().clone();

    //     ImmutableImage::from_iter(
    //         image_data.iter().cloned(),
    //         Dimensions::Dim2d { width: 256, height: 256 },
    //         Format::R8G8B8A8Srgb,
    //         queue.clone()
    //     ).unwrap()
    // };

    // let sampler = Sampler::new(
    //     device.clone(),
    //     Filter::Linear,
    //     Filter::Linear,
    //     MipmapMode::Nearest,
    //     SamplerAddressMode::Repeat,
    //     SamplerAddressMode::Repeat,
    //     SamplerAddressMode::Repeat,
    //     0.0,
    //     1.0,
    //     0.0,
    //     0.0
    // ).unwrap();

    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_buffer(data_buffer.clone()).unwrap()
        // .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );

    let mut dynamic_state = DynamicState {
        line_width: None, 
        viewports: None, 
        scissors: None
    };

    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    let mut recreate_swapchain = false;

    // let mut previous_frame_end = Box::new(tex_future) as Box<GpuFuture>;

    let mut previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;

    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            }
            else {
                return;
            };

            let (new_swapchain, new_images) =
                match swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                    Err(err) => panic!("{:?}", err)
                };

            swapchain = new_swapchain;

            framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        let (image_number, acquire_future) =
            match acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            device.clone(),
            queue.family()
        ).unwrap()
        .begin_render_pass(
            framebuffers[image_number].clone(),
            false,
            clear_values
        ).unwrap()
        .draw_indexed(
            pipeline.clone(),
            &dynamic_state,
            vertex_buffer.clone(),
            index_buffer.clone(),
            descriptor_set.clone(),
            ()
        ).unwrap()
        .end_render_pass().unwrap()
        .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_number)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            },
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            },
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;

        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => done = true,
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => recreate_swapchain = true,
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => println!("Cursor position: {0}, {1}", position.x, position.y),
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { 
                        input: KeyboardInput {
                            virtual_keycode: Some(code),
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    if code == VirtualKeyCode::Escape {
                        done = true;
                    }
                },
                _ => ()
            }
        });

        if done {
            return;
        }
    }

    // For reference:
    // https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/triangle.rs
}
