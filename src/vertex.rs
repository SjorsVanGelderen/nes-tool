#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position, uv);