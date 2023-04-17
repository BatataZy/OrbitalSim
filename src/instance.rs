
use cgmath::{Vector3, Quaternion, Matrix4, Rotation3, Deg};
use egui_wgpu::wgpu;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub color: [f32; 4]
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    color: [f32; 4],
    model: [[f32; 4]; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {color: self.color, model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into()}
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> egui_wgpu::wgpu::VertexBufferLayout<'a> {
        use std::mem;
        egui_wgpu::wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as egui_wgpu::wgpu::BufferAddress,
            step_mode: egui_wgpu::wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
            ],
        }
    }
}

//Creates voxels in a specified x, y, z coordinate face by face.
    pub fn instantiate(resolution: f32, x: i16, y: i16, z: i16, a: f32, s: f32, bias: (f32, f32, f32), ignore: (bool, bool, bool)) -> Vec<Instance> {

        //Creates an array of faces that will then be instanced
        let mut voxels: Vec<Instance> = Vec::with_capacity(3);

        let alpha:f32 = a;

        //Instances the three faces of a voxel closest to the camera
            //X
                if ignore.0 == false {voxels.push(Instance{position: Vector3::new((x as f32 + 0.5 + 0.5 * bias.0) / resolution - 0.5, (y as f32 + 0.5) / resolution - 0.5, (z as f32 + 0.5) / resolution - 0.5),
                                    rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(90.0)),
                                    color: [s, 0.7, -s, alpha]})}
        
            //Y
                if ignore.1 == false {voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / resolution - 0.5, (y as f32 + 0.5 + 0.5 * bias.1) / resolution - 0.5, (z as f32 + 0.5) / resolution - 0.5),
                                    rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                                    color: [s, 0.7, -s, alpha]})}

            //Z
                if ignore.2 == false {voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / resolution - 0.5, (y as f32 + 0.5) / resolution - 0.5, (z as f32 + 0.5 + 0.5 * bias.2) / resolution - 0.5),
                                    rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(90.0)),
                                    color: [s, 0.7, -s, alpha]})}
        return voxels;
    }