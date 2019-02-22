// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
    Matrix4,
    Point3,
    SquareMatrix,
};

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

pub struct View {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
}

impl View {
    pub fn new(aspect: f32) -> View {
        let model = Matrix4::identity();

        let view = Matrix4::look_at(
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0)
        );

        let projection = cgmath::ortho(
            -100.0 * aspect, 100.0 * aspect,
            -100.0, 100.0,
            0.01, 100.0
        );

        View {
            model,
            view,
            projection,
        }
    }

    pub fn mvp(&self) -> Matrix4<f32> {
        self.model * self.view * self.projection
    }

    pub fn update_model(&self, model: Matrix4<f32>) -> View {
        View {
            model,
            ..*self
        }
    }

    pub fn update_projection(&self, aspect: f32) -> View {
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
    // pub left_down: bool,
    // pub right_down: bool,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            position: Vector2::new(0.0, 0.0),
            // left_down: false,
            // right_down: false,
        }
    }
}

pub struct AppState {
    pub aspect: f32,
    pub dimensions: Vector2<u32>,
    pub view: View,
    pub mouse: Mouse,
    pub drag_start: Vector2<f32>,
    pub dragging: bool,
    // pub pattern_table: PatternTable,
}

impl AppState {
    pub fn new(aspect: f32) -> AppState {
        AppState {
            aspect,
            dimensions: Vector2::new(0, 0),
            view: View::new(aspect),
            mouse: Mouse::new(),
            drag_start: Vector2::new(0.0, 0.0),
            dragging: false,
            // pattern_table: ,
        }
    }
}