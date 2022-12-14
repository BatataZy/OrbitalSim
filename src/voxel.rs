use egui_wgpu::wgpu;

//LENGTH – The length of the side of the bounding box where the functions are drawn
pub const LENGTH: i16 = 7;

//THRESHOLD – The orbital's threshold
pub const THRESHOLD: f32 = 0.0003;

//VERTEX STUFF
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> egui_wgpu::wgpu::VertexBufferLayout<'a> {
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }
            ],
        }
    }
}

//Creates the individual faces for the voxels –basically squares–.
    pub fn generate_face(resolution: f32) -> Vec<Vertex> {
        let mut vertices: Vec::<Vertex> = vec![];
        for x in 0..=1 {
        for z in 0..=1 {
            vertices.push(Vertex{position: [(x as f32 - 0.5 )/ resolution,
                                             0.0 as f32,
                                            (z as f32 - 0.5 )/ resolution]});
        }}
        return vertices;
    }

//Indices for the square to render triangles properly
pub const INDICES: &[u16] = &[
    0, 1, 2,
    1, 2, 3,
];