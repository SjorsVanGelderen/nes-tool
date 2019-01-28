// Copyright 2019, Sjors van Gelderen

use crate::vertex::Vertex;

pub struct Surface {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Surface {
    pub fn zero() -> Surface {
        let positions: [[f32; 3]; 4] = [
            [-0.5, -0.5, 1.0],
            [-0.5,  0.5, 1.0],
            [ 0.5,  0.5, 1.0],
            [ 0.5, -0.5, 1.0],
        ];

        let uvs: [[f32; 2]; 4] = [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ];

        let indices: [u32; 6] = [
            0, 1, 2, 2, 3, 0
        ];

        Surface {
            vertices: positions.iter().zip(uvs.iter()).map(
                |(p, u)| Vertex { position: *p, uv: *u } 
            ).collect(),
            indices: indices.to_vec(), // TODO: Check if the CpuAccessibleBuffer accepts slices also
        }
    }
}