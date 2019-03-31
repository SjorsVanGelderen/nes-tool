// Copyright 2019, Sjors van Gelderen

extern crate cgmath;
extern crate image;
extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

mod attribute_table;
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

use crate::palette::Palette;
use crate::pattern_table::PatternTable;
use crate::samples::Samples;

use crate::system::{
    Mouse,
    View,
};

use cgmath::{
    Matrix4,
    Vector2,
    Vector3,
};

use std::{
    path::Path,
    sync::Arc,
};

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder,
        DynamicState,
    },
    device::Device,
    sampler::{
        Filter,
        MipmapMode,
        Sampler,
        SamplerAddressMode,
    },
    swapchain::{
        AcquireError,
        acquire_next_image,
        SwapchainCreationError,
    },
    sync::{
        FlushError,
        GpuFuture,
        now,
    },
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

    let sampler = get_sampler(device.clone());
    let (mut swapchain, images) = system::get_swapchain_and_images(
        surface.clone(), 
        physical, 
        window, 
        device.clone(), 
        queue.clone()
    );

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

    let palette = Palette::new(
        device.clone(), queue.clone(), render_pass.clone(), sampler.clone()
    ).set_position(Vector3::new(-80.0, -80.0, 0.0));

    let samples = Samples::new(
        device.clone(), queue.clone(), render_pass.clone(), sampler.clone()
    ).set_position(Vector3::new(80.0, -80.0, 0.0));
    
    let pattern_table = PatternTable::new(
        device.clone(), queue.clone(), render_pass.clone(), sampler.clone()
    ).load_from_file(
        device.clone(), Path::new("mario.chr"), queue.clone(), sampler.clone()
    );

    let mut dynamic_state = DynamicState {
        line_width: None, 
        viewports: None, 
        scissors: None
    };

    let mut framebuffers = system::get_window_size_dependent_setup(
        &images, render_pass.clone(), &mut dynamic_state
    );

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Box::new(
        pattern_table.tex_future
            .join(palette.tex_future)
            .join(samples.tex_future)
    ) as Box<GpuFuture>;

    let mut view = View::new(Vector2::new(1600, 900));
    let mut mouse = Mouse::new();

    view.update_projection();

    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                view = View::new(Vector2::new(dimensions.0, dimensions.1));

                [dimensions.0, dimensions.1]
            }
            else {
                return;
            };

            let (new_swapchain, new_images) =
                match swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                    Err(e) => panic!("{:?}", e)
                };

            swapchain = new_swapchain;
            framebuffers = system::get_window_size_dependent_setup(
                &new_images, render_pass.clone(), &mut dynamic_state
            );
            
            recreate_swapchain = false;
        }

        // TODO: Move the logic below to the correct modules
        // TODO: Figure out a better way to supply a mat4 as a push constant
        let pattern_table_mouse = get_mouse_position_on_surface(
            mouse.position,
            Vector2::new(
                pattern_table.surface.position.x,
                pattern_table.surface.position.y
            ),
            pattern_table.surface.dimensions
        );

        let mvp = view.mvp(Matrix4::from_translation(pattern_table.surface.position) * Matrix4::from_scale(view.zoom));
        let pattern_table_push_constants = pattern_table::vs::ty::push_constants {
            mvp: [
                [ mvp.x.x, mvp.x.y, mvp.x.z, mvp.x.w ],
                [ mvp.y.x, mvp.y.y, mvp.y.z, mvp.y.w ],
                [ mvp.z.x, mvp.z.y, mvp.z.z, mvp.z.w ],
                [ mvp.w.x, mvp.w.y, mvp.w.z, mvp.w.w ],
            ],
            mouse: [ pattern_table_mouse.x, pattern_table_mouse.y ],
        };

        let palette_mouse = get_mouse_position_on_surface(
            mouse.position,
            Vector2::new(
                palette.surface.position.x,
                palette.surface.position.y
            ),
            palette.surface.dimensions
        );

        let mvp = view.mvp(Matrix4::from_translation(palette.surface.position));
        let palette_push_constants = palette::vs::ty::push_constants {
            mvp: [
                [ mvp.x.x, mvp.x.y, mvp.x.z, mvp.x.w ],
                [ mvp.y.x, mvp.y.y, mvp.y.z, mvp.y.w ],
                [ mvp.z.x, mvp.z.y, mvp.z.z, mvp.z.w ],
                [ mvp.w.x, mvp.w.y, mvp.w.z, mvp.w.w ],
            ],
            mouse: [ palette_mouse.x, palette_mouse.y ],
        };

        let samples_mouse = get_mouse_position_on_surface(
            mouse.position,
            Vector2::new(
                samples.surface.position.x,
                samples.surface.position.y
            ),
            samples.surface.dimensions
        );

        let mvp = view.mvp(Matrix4::from_translation(samples.surface.position));
        let samples_push_constants = samples::vs::ty::push_constants {
            mvp: [
                [ mvp.x.x, mvp.x.y, mvp.x.z, mvp.x.w ],
                [ mvp.y.x, mvp.y.y, mvp.y.z, mvp.y.w ],
                [ mvp.z.x, mvp.z.y, mvp.z.z, mvp.z.w ],
                [ mvp.w.x, mvp.w.y, mvp.w.z, mvp.w.w ],
            ],
            mouse: [ samples_mouse.x, samples_mouse.y ],
        };

        let (image_number, acquire_future) =
            match acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                },
                Err(e) => panic!("{:?}", e)
            };

        let clear_values = vec!([0.16, 0.05, 0.32, 1.0].into());

        // TODO: Investigate getting the parts of the builder from their respective modules
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
            pattern_table.pipeline.clone(),
            &dynamic_state,
            pattern_table.surface.vertex_buffer.clone(),
            pattern_table.surface.index_buffer.clone(),
            pattern_table.descriptor_set.clone(),
            pattern_table_push_constants
        ).unwrap()
        .draw_indexed(
            palette.pipeline.clone(),
            &dynamic_state,
            palette.surface.vertex_buffer.clone(),
            palette.surface.index_buffer.clone(),
            palette.descriptor_set.clone(),
            palette_push_constants
        ).unwrap()
        .draw_indexed(
            samples.pipeline.clone(),
            &dynamic_state,
            samples.surface.vertex_buffer.clone(),
            samples.surface.index_buffer.clone(),
            samples.descriptor_set.clone(),
            samples_push_constants
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
                previous_frame_end = Box::new(now(device.clone())) as Box<_>;
            },
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(now(device.clone())) as Box<_>;
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
                    let mp = position.to_physical(window.get_hidpi_factor());
                    let mp = Vector2::new(mp.x as f32, mp.y as f32);
                    let wd = Vector2::new(view.window_dimensions.x as f32, view.window_dimensions.y as f32);
                    let pd = view.projection_dimensions;
                    let aspect = wd.x / wd.y;

                    mouse.position = Vector2::new(
                        mp.x / wd.x * pd.x * aspect - pd.x * aspect / 2.0,
                        mp.y / wd.y * pd.y - pd.y / 2.0,
                    );
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    match button {
                        MouseButton::Left => {
                            // mouse.left_down = state == ElementState::Pressed;

                            if state == ElementState::Pressed {
                                mouse.drag_start = mouse.position;
                            }
                        },
                        MouseButton::Right => {
                            // mouse.left_down = state == ElementState::Pressed;
                        },
                        _ => ()
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseWheel { delta, .. },
                    ..
                } => {
                    match delta {
                        winit::MouseScrollDelta::LineDelta(_, y) => {
                            view = view.zoom(y * 0.075);
                        },
                        _ => {},
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
                        mouse.dragging = state == ElementState::Pressed;
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

// TODO: Move to system module
fn get_sampler(device: Arc<Device>) -> Arc<Sampler> {
    Sampler::new(
        device.clone(),
        Filter::Nearest,
        Filter::Nearest,
        MipmapMode::Nearest,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        SamplerAddressMode::ClampToEdge,
        0.0, 1.0, 0.0, 0.0
    ).unwrap()
}

// TODO: Move to system module
fn get_mouse_position_on_surface(
    mouse_position: Vector2<f32>,
    surface_position: Vector2<f32>,
    surface_dimensions: Vector2<f32>,
) -> Vector2<f32> {
    let mp = Vector2::new(mouse_position.x, -mouse_position.y);
    let sp = surface_position;
    let sd = Vector2::new(surface_dimensions.x, surface_dimensions.y);

    if (mp.x - sp.x).abs() < sd.x / 2.0
    && (mp.y - sp.y).abs() < sd.y / 2.0 {
        let x = (mp.x - (sp.x - sd.x / 2.0)).abs() / surface_dimensions.x;
        let y = (mp.y - (sp.y - sd.y / 2.0)).abs() / surface_dimensions.y;

        Vector2::new(x, 1.0 - y)
    }
    else {
        // Negative means not on the surface
        Vector2::new(-1.0, -1.0)
    }
}