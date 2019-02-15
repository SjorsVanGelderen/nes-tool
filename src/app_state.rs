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

pub struct AppState {
    // pub pattern_table: PatternTable,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            // pattern_table: ,
        }
    }
}