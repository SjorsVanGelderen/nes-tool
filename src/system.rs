// Copyright 2019, Sjors van Gelderen

use vulkano::{
    command_buffer::{
        DynamicState,
    },
    device::{
        Device,
        DeviceExtensions,
        Queue,
        QueuesIter,
    },
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPassAbstract,
    },
    image::{
        SwapchainImage,
    },
    instance::{
        Instance,
        PhysicalDevice,
        QueueFamily,
    },
    pipeline::{
        viewport::Viewport,
    },
    swapchain::{
        PresentMode,
        Surface,
        SurfaceTransform,
        Swapchain,
    },
};

use vulkano_win::{
    VkSurfaceBuild,
};

use winit::{
    EventsLoop,
    Window,
    WindowBuilder,
};

use std::sync::Arc;

pub fn get_instance() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();

    Instance::new(None, &extensions, None).expect("Failed to create instance")
}

pub fn get_physical(instance: &Arc<Instance>) -> PhysicalDevice {
    PhysicalDevice::enumerate(instance).next()
        .expect("Failed to find device")
}

pub fn get_queue_family(physical: PhysicalDevice) -> QueueFamily {
    physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Failed to find a graphical queue")
}

pub fn get_device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    }
}

pub fn get_device_and_queues(physical: PhysicalDevice, extensions: DeviceExtensions, queue_family: QueueFamily) -> (Arc<Device>, QueuesIter) {
    Device::new(
        physical,
        physical.supported_features(),
        &extensions,
        [(queue_family, 0.5)].iter().cloned()
    ).expect("Failed to get device and queues")
}

pub fn get_surface(events_loop: &EventsLoop, instance: Arc<Instance>) -> Arc<Surface<Window>> {
    WindowBuilder::new()
        .with_title("NES tool")
        .build_vk_surface(&events_loop, instance)
        .unwrap()
}

pub fn get_swapchain_and_images(
    surface: Arc<Surface<Window>>, 
    physical: PhysicalDevice, 
    window: &Window,
    device: Arc<Device>,
    queue: Arc<Queue>
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    let capabilities = surface.capabilities(physical).expect("Failed to get surface capabilities");

    let alpha = capabilities.supported_composite_alpha.iter().next()
        .expect("Failed to get supported composite alpha capability");

    let format = capabilities.supported_formats[0].0;

    let dimensions = if let Some(dimensions) = window.get_inner_size() {
        let dimensions: (u32, u32) = dimensions
            .to_physical(window.get_hidpi_factor())
            .into();
        
        [dimensions.0, dimensions.1]
    }
    else {
        panic!("Failed to acquire window dimensions");
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
    ).expect("Failed to create swapchain")
}

pub fn get_window_size_dependent_setup(
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