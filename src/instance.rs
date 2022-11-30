
use cgmath::{Vector3, Quaternion, Matrix4, Rotation3, Deg};

use crate::voxel::{RES};

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
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
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
    pub fn instantiate(x: i16, y: i16, z: i16, a: f32, faces: &Vec<(i16, i16, i16)>) -> Vec<Instance> {

        //Creates an array of faces that will then be instanced
        let mut voxels: Vec<Instance> = Vec::with_capacity(3);

        //Three primary faces: positive x, y and z. 
        //These are always instantiated.
            //RIGHT
                voxels.push(Instance{position: Vector3::new((x as f32 + 1.0) / RES - 0.5, (y as f32 + 0.5) / RES - 0.5, (z as f32 + 0.5) / RES - 0.5),
                                    rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(90.0)),
                                    color: [1.0, 0.7, 0.0, a]});
        
            //UP
                voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / RES - 0.5, (y as f32 + 1.0) / RES - 0.5, (z as f32 + 0.5) / RES - 0.5),
                                    rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                                    color: [1.0, 0.7, 0.0, a]});

            //FRONT
                voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / RES - 0.5, (y as f32 + 0.5) / RES - 0.5, (z as f32 + 1.0) / RES - 0.5),
                                    rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(90.0)),
                                    color: [1.0, 0.7, 0.0, a]});

            
        //Three secondary faces: negative x, y, z. 
        //These are only instantiated when the negative neighboring coordinate doesn't have a primary face already
        //(so, when the negative neighbour is empty).
            //LEFT
                match faces.binary_search(&(x - 1, y, z)) {
                    Ok(_) => {}

                    Err(_) => {
                        voxels.push(Instance{position: Vector3::new((x as f32) / RES - 0.5, (y as f32 + 0.5) / RES - 0.5, (z as f32 + 0.5) / RES - 0.5),
                                rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(90.0)),
                                color: [1.0, 0.7, 0.0, a]});
                    }
                }

            //DOWN
                match faces.binary_search(&(x, y - 1, z)) {
                    Ok(_) => {}

                    Err(_) => {
                        voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / RES - 0.5, (y as f32) / RES - 0.5, (z as f32 + 0.5) / RES - 0.5),
                                rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                                color: [1.0, 0.7, 0.0, a]});
                    }
                }

            //BACK
                match faces.binary_search(&(x, y, z - 1)) {
                    Ok(_) => {}

                    Err(_) => {
                        voxels.push(Instance{position: Vector3::new((x as f32 + 0.5) / RES - 0.5, (y as f32 + 0.5) / RES - 0.5, (z as f32) / RES - 0.5),
                                rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(90.0)),
                                color: [1.0, 0.7, 0.0, a]});
                    }
                }

        return voxels;
    }