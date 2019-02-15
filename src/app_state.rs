// Copyright 2019, Sjors van Gelderen

// use crate::attribute_table::AttributeTable;
// use crate::nametable::Nametable;
// use crate::palette::Palette;
use crate::pattern_table::PatternTable;

use std::option::Option;
use std::sync::Arc;

use vulkano::device::{
    Device,
    DeviceExtensions,
    QueuesIter,
};

use vulkano::instance::{
    Instance,
    PhysicalDevice,
    QueueFamily,
};

pub struct AppState<'a> {
    pub instance: Option<Arc<Instance>>,
    pub physical: Option<PhysicalDevice<'a>>,
    pub queue_family: Option<QueueFamily<'a>>,
    pub device: Option<Arc<Device>>,
    pub queues: Option<QueuesIter>,
    pub pattern_table: Option<PatternTable>,
}

impl<'a> AppState<'a> {
    pub fn new() -> AppState<'a> {
        let app_state: AppState = AppState {
            instance: None,
            physical: None,
            queue_family: None,
            device: None,
            queues: None,
            pattern_table: None,
        };

        app_state

        // app_state
        //     .setup_instance()
        //     .setup_physical()
        //     .setup_queue_family()
        //     .setup_device_and_queues()
    }

    fn setup_instance(&self) -> AppState {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            
            Instance::new(None, &extensions, None).expect("Failed to create instance")
        };

        AppState {
            instance: Some(instance),
            ..self
        }
    }

    fn setup_physical(&self) -> AppState {
        match self.instance {
            Some(i) => {
                let physical = PhysicalDevice::enumerate(i).next()
                    .expect("No device found");

                AppState {
                    physical: Some(physical), // Perhaps a better syntax exists for this
                    ..self
                }
            },
            None => self,
        }
    }

    fn _enumerate_queues(p: PhysicalDevice) {
        for family in p.queue_families() {
            println!("Found a queue family with {:?} queue(s)", family.queues_count());
        }
    }

    fn setup_queue_family(&self, p: PhysicalDevice) -> AppState {
        let queue_family = p.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("No graphical queues found");

        AppState {
            queue_family: queue_family,
            ..self
        }
    }

    fn setup_device_and_queues(&self, p: PhysicalDevice, q: QueueFamily) -> AppState {
        let extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, queues) = Device::new(
            p,
            p.supported_features(),
            &extensions,
            [(q, 0.5)].iter().cloned()
        ).expect("Failed to create device");

        AppState {
            device: device,
            queues: queues,
            ..self
        }
    }
}