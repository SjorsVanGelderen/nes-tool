// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Matrix4,
    Point3,
    SquareMatrix,
    Vector2,
    Vector3,
};

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
    dpi::LogicalSize,
    EventsLoop,
    Window,
    WindowBuilder,
};

use std::sync::Arc;

pub struct View {
    pub dimensions: Vector2<u32>,
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

impl View {
    pub fn new(dimensions: Vector2<u32>) -> Self {
        let model = Matrix4::identity();

        let view = Matrix4::look_at(
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0)
        );

        let aspect = dimensions.x as f32 / dimensions.y as f32;

        let projection = cgmath::ortho(
            -100.0 * aspect, 100.0 * aspect,
            -100.0, 100.0,
            0.01, 100.0
        );

        Self {
            dimensions,
            model,
            view,
            projection,
        }
    }

    pub fn mvp(&self) -> Matrix4<f32> {
        self.model * self.view * self.projection
    }

    pub fn mvp_from_model(&self, model: Matrix4<f32>) -> Matrix4<f32> {
        model * self.view * self.projection
    }

    pub fn update_model(&self, model: Matrix4<f32>) -> View {
        View {
            model,
            ..*self
        }
    }

    pub fn update_projection(&self) -> View {
        let aspect = self.dimensions.x as f32 / self.dimensions.y as f32;

        let projection = cgmath::ortho(
            -100.0 * aspect, 100.0 * aspect,
            -100.0, 100.0,
            0.01, 100.0
        );

        View {
            projection,
            ..*self
        }
    }
}

pub struct Mouse {
    pub position: Vector2<f32>,
    pub dragging: bool,
    pub drag_start: Vector2<f32>,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            position: Vector2::new(0.0, 0.0),
            dragging: false,
            drag_start: Vector2::new(0.0, 0.0),
        }
    }
}

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
        .with_dimensions(LogicalSize::new(1280.0, 720.0))
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