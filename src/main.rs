// Copyright 2019, Sjors van Gelderen

extern crate cgmath;
extern crate image;
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

// use crate::attribute_table::AttributeTable;
use crate::app_state::AppState;
use crate::pattern_table::PatternTable;
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
mod system;
mod tool;
mod vertex;

use cgmath::{
    Matrix4,
    Vector2,
    Vector3,
};

use std::path::Path;
use std::sync::Arc;

use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    DynamicState,
};

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::framebuffer::{
    Subpass,
};

use vulkano::pipeline::{
    GraphicsPipeline,
};

use vulkano::sampler::{
    Sampler,
    SamplerAddressMode,
    Filter,
    MipmapMode
};

use vulkano::swapchain::{
    AcquireError,
    SwapchainCreationError,
    acquire_next_image,
};

use vulkano::sync::{
    FlushError,
    GpuFuture,
};

use winit::{
    ElementState,
    Event,
    EventsLoop,
    MouseButton,
    WindowEvent,
    KeyboardInput,
    VirtualKeyCode,
};

fn main() {
    let instance = system::get_instance();
    let physical = system::get_physical(&instance);
    let queue_family = system::get_queue_family(physical);
    let extensions = system::get_device_extensions();
    let (device, mut queues) = system::get_device_and_queues(physical, extensions, queue_family);
    let queue = queues.next().unwrap();
    let mut events_loop = EventsLoop::new();
    let surface = system::get_surface(&events_loop, instance.clone());
    let window = surface.window();

    let (mut swapchain, images) = system::get_swapchain_and_images(
        surface.clone(), 
        physical, 
        window, 
        device.clone(), 
        queue.clone()
    );

    let pattern_table = PatternTable::zero(device.clone()).load_from_file(Path::new("mario.chr"));

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
            .vertex_shader(pattern_table.vertex_shader.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(pattern_table.fragment_shader.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
    );

    // TODO: Perhaps this can be moved to the pattern table module
    let (texture, tex_future) = pattern_table.get_texture_and_future(queue.clone());

    let sampler = Sampler::new(
        device.clone(),
        Filter::Nearest,
        Filter::Nearest,
        MipmapMode::Nearest,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        0.0, 1.0, 0.0, 0.0
    ).unwrap();

    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );

    // let (texture, tex_future) = {
    //     let image_data: Vec<u8> = palette::FULL_PALETTE.chunks(3).flat_map(
    //         |x| vec![x[0], x[1], x[2], 255u8]
    //     ).collect();

    //     ImmutableImage::from_iter(
    //         image_data.iter().cloned(),
    //         Dimensions::Dim2d { width: 16, height: 4 },
    //         Format::R8G8B8A8Unorm,
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

    let mut dynamic_state = DynamicState {
        line_width: None, 
        viewports: None, 
        scissors: None
    };

    let mut framebuffers = system::get_window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    let mut recreate_swapchain = false;

    // TODO: Read up on this
    let mut previous_frame_end = Box::new(tex_future) as Box<GpuFuture>;
    // let mut previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;

    let mut app_state: AppState = AppState::new(4.0 / 3.0);

    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();

                app_state.aspect = dimensions.0 as f32 / dimensions.1 as f32;
                app_state.view = app_state.view.update_projection(app_state.aspect);
                app_state.dimensions = Vector2::new(dimensions.0, dimensions.1);

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

            framebuffers = system::get_window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        let mvp = app_state.view.mvp();

        // TODO: Figure out a better way to supply a mat4 as a push constant
        let push_constants = pattern_table::vs::ty::Matrices {
            mvp: [
                [ mvp.x.x, mvp.x.y, mvp.x.z, mvp.x.w ],
                [ mvp.y.x, mvp.y.y, mvp.y.z, mvp.y.w ],
                [ mvp.z.x, mvp.z.y, mvp.z.z, mvp.z.w ],
                [ mvp.w.x, mvp.w.y, mvp.w.z, mvp.w.w ],
            ],
        };

        let (image_number, acquire_future) =
            match acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

        let clear_values = vec!([0.16, 0.05, 0.32, 1.0].into());

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
            pattern_table.vertex_buffer.clone(),
            pattern_table.index_buffer.clone(),
            descriptor_set.clone(),
            push_constants
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
                } => {
                    app_state.mouse.position = Vector2::new(position.x as f32, position.y as f32);

                    if app_state.dragging {
                        app_state.view = app_state.view.update_model(
                            Matrix4::from_translation(
                                Vector3::new(
                                    (app_state.mouse.position.x - app_state.dimensions.x as f32 / 2.0) / 1000.0,
                                    (app_state.mouse.position.y - app_state.dimensions.y as f32 / 2.0) / 1000.0, 
                                    0.0
                                )
                            )
                        );

                        app_state.view = app_state.view.update_projection(app_state.aspect);
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    match button {
                        MouseButton::Left => {
                            // app_state.mouse.left_down = state == ElementState::Pressed;

                            if state == ElementState::Pressed {
                                app_state.drag_start = app_state.mouse.position;
                            }
                        },
                        MouseButton::Right => {
                            // app_state.mouse.left_down = state == ElementState::Pressed;
                        },
                        _ => ()
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { 
                        input: KeyboardInput {
                            virtual_keycode: Some(code),
                            state,
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    if code == VirtualKeyCode::Escape {
                        done = true;
                    }
                    else if code == VirtualKeyCode::Space {
                        app_state.dragging = state == ElementState::Pressed;
                    }
                },
                _ => ()
            }
        });

        if done {
            return;
        }
    }
}