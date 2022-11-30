
//RESOLUTION – Inverse of the size of every individual voxel. CANNOT be 1 or less
pub const RES: f32 = 6.0;

//LENGTH – The length of the side of the bounding box where the functions are drawn
pub const LENGTH: i16 = 7;

//THRESHOLD – The orbital's threshold
pub const THRESHOLD: f32 = 0.0001;

//VERTEX STUFF
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
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

//Actually doest't generate voxels, just creates squares. Voxels are made of squares tho!
    pub fn generate_voxel() -> Vec<Vertex> {
        let mut vertices: Vec::<Vertex> = vec![];
        for x in 0..(2) {
            for y in 0..(1) {
                for z in 0..(2) {
                    vertices.push(Vertex{position: [((x) as f32 / RES - 1.0/(RES * 2.0)),
                                                    //((y) as f32 / RES - 1.0/(RES * 2.0)),
                                                    y as f32,
                                                    ((z) as f32 / RES - 1.0/(RES * 2.0))]});
                }
            }
        }
        return vertices;
    }

//Indices for the square to render triangles properly
pub const INDICES: &[u16] = &[
    0, 1, 2,
    1, 2, 3,
];