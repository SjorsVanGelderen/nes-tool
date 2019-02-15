// Copyright 2019, Sjors van Gelderen

extern crate cgmath;
extern crate image;
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

// use crate::attribute_table::AttributeTable;
use crate::app_state::AppState;
// use crate::pattern_table::PatternTable;
// use crate::nametable::Nametable;
// use crate::samples::Samples;
// use crate::surface::Surface;
// use crate::vertex::Vertex;

mod attribute_table;
mod app_state;
mod media;
mod mode;
mod nametable;
mod palette;
mod pattern_table;
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

use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use vulkano::buffer::{
    BufferUsage,
    CpuAccessibleBuffer,
    CpuBufferPool,
    DeviceLocalBuffer,
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

fn main() {
    let state: AppState = AppState::new();

    // // enumerate_queues();

    // let mut events_loop = EventsLoop::new();

    // let surface = WindowBuilder::new()
    //     .with_title("NES Tool")
    //     .build_vk_surface(&events_loop, instance.clone())
    //     .unwrap();

    // let window = surface.window();

    // let queue_family = get_queue_family(physical);

    // let (device, mut queues) = get_device_and_queues(physical, queue_family);

    // let queue = queues.next().unwrap();

    // let (mut swapchain, images) = {
    //     let capabilities = surface.capabilities(physical).expect("Failed to get surface capabilities");
    //     let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
    //     let format = capabilities.supported_formats[0].0;

    //     let dimensions = if let Some(dimensions) = window.get_inner_size() {
    //         let dimensions: (u32, u32) = dimensions
    //             .to_physical(window.get_hidpi_factor())
    //             .into();
            
    //         [dimensions.0, dimensions.1]
    //     }
    //     else {
    //         return;
    //     };

    //     Swapchain::new(
    //         device.clone(),
    //         surface.clone(),
    //         capabilities.min_image_count,
    //         format,
    //         dimensions,
    //         1,
    //         capabilities.supported_usage_flags,
    //         &queue,
    //         SurfaceTransform::Identity,
    //         alpha,
    //         PresentMode::Fifo,
    //         true,
    //         None
    //     ).expect("Failed to create swapchain")
    // };

    // let pattern_table_surface: surface::Surface = pattern_table::surface_zero();

    // // TODO: Get rid of CpuAccessibleBuffer as it will probably be deprecated
    // // Perhaps check https://docs.rs/vulkano/0.11.1/vulkano/pipeline/vertex/index.html ?
    // let vertex_buffer = CpuAccessibleBuffer::from_iter(
    //     device.clone(), 
    //     BufferUsage::all(),
    //     pattern_table_surface.vertices.iter().cloned()
    // ).unwrap();

    // let index_buffer = CpuAccessibleBuffer::from_iter(
    //     device.clone(),
    //     BufferUsage::all(),
    //     pattern_table_surface.indices.iter().cloned()
    // ).unwrap();

    // let pattern_table_vs = pattern_table::vs::Shader::load(device.clone()).expect("Failed to create vertex shader");
    // let pattern_table_fs = pattern_table::fs::Shader::load(device.clone()).expect("Failed to create fragment shader");

    // // TODO: The palette table needs its own shaders
    // let palette_surface: surface::Surface = palette::surface_zero();

    // let render_pass = Arc::new(
    //     vulkano::single_pass_renderpass!(
    //         device.clone(),
    //         attachments: {
    //             color: {
    //                 load: Clear,
    //                 store: Store,
    //                 format: swapchain.format(),
    //                 samples: 1,
    //             }
    //         },
    //         pass: {
    //             color: [color],
    //             depth_stencil: {}
    //         }
    //     ).unwrap()
    // );

    // // TODO: Need to add information about the palette table here
    // let pipeline = Arc::new(
    //     GraphicsPipeline::start()
    //         .vertex_input_single_buffer()
    //         .vertex_shader(pattern_table_vs.main_entry_point(), ())
    //         .triangle_list()
    //         .viewports_dynamic_scissors_irrelevant(1)
    //         .fragment_shader(pattern_table_fs.main_entry_point(), ())
    //         .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
    //         .build(device.clone())
    //         .unwrap()
    // );

    // let mut aspect: f32 = 4.0 / 3.0;

    // let mut projection: Matrix4<f32> = ortho(
    //     -100.0 * aspect, 100.0 * aspect,
    //     -100.0, 100.0,
    //     0.01, 100.0
    // );
    
    // let mut view: Matrix4<f32> = Matrix4::look_at(
    //     Point3::new(0.0, 0.0, -1.0),
    //     Point3::new(0.0, 0.0, 0.0),
    //     Vector3::new(0.0, -1.0, 0.0)
    // );

    // let mut model: Matrix4<f32> = Matrix4::identity();

    // let mut mvp: Matrix4<f32> = model * view * projection;

    // // TODO: Move this logic to the pattern table module
    // let (texture, tex_future) = {
    //     let pattern: pattern_table::PatternTable =
    //         match media::load_pattern_table(Path::new("mario.chr")) {
    //             Ok(p) => p,
    //             Err(e) => panic!(e),
    //         };

    //     let mut image_data: [u8; 32768] = [0u8; 32768];
        
    //     for (i, x) in pattern.pixels.iter().enumerate() {
    //         let pixel: u8 = (*x as f32 * (255.0 / 4.0)) as u8;

    //         image_data[i] = pixel;
    //     }

    //     ImmutableImage::from_iter(
    //         image_data.iter().cloned(),
    //         Dimensions::Dim2d { width: 256, height: 128 },
    //         Format::R8Unorm,
    //         queue.clone()
    //     ).unwrap()
    // };

    // let sampler = Sampler::new(
    //     device.clone(),
    //     Filter::Nearest,
    //     Filter::Nearest,
    //     MipmapMode::Nearest,
    //     SamplerAddressMode::ClampToEdge,
    //     SamplerAddressMode::ClampToEdge,
    //     SamplerAddressMode::ClampToEdge,
    //     0.0, 1.0, 0.0, 0.0
    // ).unwrap();

    // let descriptor_set = Arc::new(
    //     PersistentDescriptorSet::start(pipeline.clone(), 0)
    //     .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
    //     .build().unwrap()
    // );

    // // let (texture, tex_future) = {
    // //     let image_data: Vec<u8> = palette::FULL_PALETTE.chunks(3).flat_map(
    // //         |x| vec![x[0], x[1], x[2], 255u8]
    // //     ).collect();

    // //     ImmutableImage::from_iter(
    // //         image_data.iter().cloned(),
    // //         Dimensions::Dim2d { width: 16, height: 4 },
    // //         Format::R8G8B8A8Unorm,
    // //         queue.clone()
    // //     ).unwrap()
    // // };

    // // let sampler = Sampler::new(
    // //     device.clone(),
    // //     Filter::Nearest,
    // //     Filter::Nearest,
    // //     MipmapMode::Nearest,
    // //     SamplerAddressMode::ClampToEdge,
    // //     SamplerAddressMode::ClampToEdge,
    // //     SamplerAddressMode::ClampToEdge,
    // //     0.0, 1.0, 0.0, 0.0
    // // ).unwrap();

    // let mut dynamic_state = DynamicState {
    //     line_width: None, 
    //     viewports: None, 
    //     scissors: None
    // };

    // let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // let mut recreate_swapchain = false;

    // // TODO: Read up on this
    // let mut previous_frame_end = Box::new(tex_future) as Box<GpuFuture>;
    // // let mut previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;

    // loop {
    //     previous_frame_end.cleanup_finished();

    //     if recreate_swapchain {
    //         let dimensions = if let Some(dimensions) = window.get_inner_size() {
    //             let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();

    //             aspect = dimensions.0 as f32 / dimensions.1 as f32;

    //             projection = ortho(
    //                 -100.0 * aspect, 100.0 * aspect,
    //                 -100.0, 100.0,
    //                 0.01, 100.0
    //             );

    //             mvp = model * view * projection;

    //             [dimensions.0, dimensions.1]
    //         }
    //         else {
    //             return;
    //         };

    //         let (new_swapchain, new_images) =
    //             match swapchain.recreate_with_dimension(dimensions) {
    //                 Ok(r) => r,
    //                 Err(SwapchainCreationError::UnsupportedDimensions) => continue,
    //                 Err(err) => panic!("{:?}", err)
    //             };

    //         swapchain = new_swapchain;

    //         framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

    //         recreate_swapchain = false;
    //     }

    //     // TODO: Figure out a better way to supply a mat4 as a push constant
    //     let push_constants = pattern_table::vs::ty::Matrices {
    //         mvp: [
    //             [ mvp.x.x, mvp.x.y, mvp.x.z, mvp.x.w ],
    //             [ mvp.y.x, mvp.y.y, mvp.y.z, mvp.y.w ],
    //             [ mvp.z.x, mvp.z.y, mvp.z.z, mvp.z.w ],
    //             [ mvp.w.x, mvp.w.y, mvp.w.z, mvp.w.w ],
    //         ],
    //     };

    //     let (image_number, acquire_future) =
    //         match acquire_next_image(swapchain.clone(), None) {
    //             Ok(r) => r,
    //             Err(AcquireError::OutOfDate) => {
    //                 recreate_swapchain = true;
    //                 continue;
    //             },
    //             Err(err) => panic!("{:?}", err)
    //         };

    //     let clear_values = vec!([0.16, 0.05, 0.32, 1.0].into());

    //     let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
    //         device.clone(),
    //         queue.family()
    //     ).unwrap()
    //     .begin_render_pass(
    //         framebuffers[image_number].clone(),
    //         false,
    //         clear_values
    //     ).unwrap()
    //     .draw_indexed(
    //         pipeline.clone(),
    //         &dynamic_state,
    //         vertex_buffer.clone(),
    //         index_buffer.clone(),
    //         descriptor_set.clone(),
    //         push_constants
    //     ).unwrap()
    //     .end_render_pass().unwrap()
    //     .build().unwrap();

    //     let future = previous_frame_end.join(acquire_future)
    //         .then_execute(queue.clone(), command_buffer).unwrap()
    //         .then_swapchain_present(queue.clone(), swapchain.clone(), image_number)
    //         .then_signal_fence_and_flush();

    //     match future {
    //         Ok(future) => {
    //             previous_frame_end = Box::new(future) as Box<_>;
    //         },
    //         Err(FlushError::OutOfDate) => {
    //             recreate_swapchain = true;
    //             previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
    //         },
    //         Err(e) => {
    //             println!("{:?}", e);
    //             previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
    //         }
    //     }

    //     let mut done = false;

    //     events_loop.poll_events(|event| {
    //         match event {
    //             Event::WindowEvent {
    //                 event: WindowEvent::CloseRequested,
    //                 ..
    //             } => done = true,
    //             Event::WindowEvent {
    //                 event: WindowEvent::Resized(_),
    //                 ..
    //             } => recreate_swapchain = true,
    //             Event::WindowEvent {
    //                 event: WindowEvent::CursorMoved { position, .. },
    //                 ..
    //             } => (), //println!("Cursor position: {0}, {1}", position.x, position.y),
    //             Event::WindowEvent {
    //                 event: WindowEvent::KeyboardInput { 
    //                     input: KeyboardInput {
    //                         virtual_keycode: Some(code),
    //                         ..
    //                     },
    //                     ..
    //                 },
    //                 ..
    //             } => {
    //                 if code == VirtualKeyCode::Escape {
    //                     done = true;
    //                 }
    //             },
    //             _ => ()
    //         }
    //     });

    //     if done {
    //         return;
    //     }
    // }
}

// fn window_size_dependent_setup(
//     images: &[Arc<SwapchainImage<Window>>],
//     render_pass: Arc<RenderPassAbstract + Send + Sync>,
//     dynamic_state: &mut DynamicState
// ) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
//     let dimensions = images[0].dimensions();

//     let viewport = Viewport {
//         origin: [0.0, 0.0],
//         dimensions: [dimensions[0] as f32, dimensions[1] as f32],
//         depth_range: 0.0 .. 1.0,
//     };

//     dynamic_state.viewports = Some(vec!(viewport));

//     images.iter().map(|image| {
//         Arc::new(
//             Framebuffer::start(render_pass.clone())
//                 .add(image.clone()).unwrap()
//                 .build().unwrap()
//         ) as Arc<FramebufferAbstract + Send + Sync>
//     }).collect::<Vec<_>>()
// }