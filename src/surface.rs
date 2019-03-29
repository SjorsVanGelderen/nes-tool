// Copyright 2019, Sjors van Gelderen

use cgmath::{
    Vector2,
    Vector3,
};

use crate::vertex::Vertex;

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
    },
    device::Device,
};

use std::sync::Arc;

pub struct Surface {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    pub position: Vector3<f32>,
}

impl Surface {
    pub fn new(device: Arc<Device>, position: Vector3<f32>, dimensions: Vector2<f32>) -> Self {
        let d = dimensions;

        let positions: [[f32; 3]; 4] = [
            [ -d.x / 2.0,  d.y / 2.0, 1.0],
            [ -d.x / 2.0, -d.y / 2.0, 1.0],
            [  d.x / 2.0, -d.y / 2.0, 1.0],
            [  d.x / 2.0,  d.y / 2.0, 1.0],
        ];

        let uvs: [[f32; 2]; 4] = [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0];

        let vertices: Vec<Vertex> = positions.iter().zip(uvs.iter()).map(
            |(p, u)| Vertex { position: *p, uv: *u }
        ).collect();

        let vertex_buffer = Self::get_vertex_buffer(device.clone(), vertices);
        let index_buffer = Self::get_index_buffer(device.clone(), indices);

        Self {
            vertex_buffer,
            index_buffer,
            position,
        }
    }

    pub fn set_position(self, position: Vector3<f32>) -> Self {
        Self {
            position,
            ..self
        }
    }

    // TODO: Find alternative to CpuAccessibleBuffer as it will be deprecated
    fn get_vertex_buffer(device: Arc<Device>, vertices: Vec<Vertex>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(), 
            BufferUsage::all(),
            vertices.iter().cloned()
        ).unwrap()
    }

    fn get_index_buffer(device: Arc<Device>, indices: Vec<u32>) -> Arc<CpuAccessibleBuffer<[u32]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            indices.iter().cloned()
        ).unwrap()
    }
}