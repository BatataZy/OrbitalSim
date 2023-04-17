use std::f32::consts::PI;

use cgmath::{Vector3};

pub const ALLOWED_ORBITALS: &[(&str, (u8, u8))] = &[
    ("[-]", (0, 0)), ("1s", (1, 0)), ("2s", (2, 0)), ("2p", (2, 1)),("3s", (3, 0)), ("3p", (3, 1)), ("3d", (3, 2)),
];

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Orbital {
    pub position: Vector3<f32>,
    pub euler: (f32, f32, f32),
    pub quaternion: (f32, f32, f32, f32),
    pub quantum: (u8, u8),
    pub magnetic: i8,
    pub phase: bool,
}

impl Orbital {
    pub fn new(position: Vector3<f32>, euler: (f32, f32, f32), quantum: (u8, u8), magnetic: i8, phase: bool) -> Orbital {

        let cr = (euler.0/360.0 * PI).cos();
        let sr = (euler.0/360.0 * PI).sin();
        let cp = (euler.1/360.0 * PI).cos();
        let sp = (euler.1/360.0 * PI).sin();
        let cy = (euler.2/360.0 * PI).cos();
        let sy = (euler.2/360.0 * PI).sin();

        let quaternion = (cr * cp * cy + sr * sp * sy,
                                                sr * cp * cy - cr * sp * sy,
                                                cr * sp * cy + sr * cp * sy,
                                                cr * cp * sy - sr * sp * cy);

        let orbital: Orbital = Orbital {position, euler, quaternion, quantum, magnetic, phase};

        return orbital;
    }
}

pub fn orbital_to_name(quantum: (u8, u8)) -> &'static str {
    let mut name: &str = "";

    let orbital_names: (Vec<&str>, Vec<(u8, u8)>) = ALLOWED_ORBITALS.to_vec().into_iter().unzip();

    orbital_names.1.into_iter().enumerate().for_each(|valid_quantum| {
        if quantum == valid_quantum.1 {name = ALLOWED_ORBITALS[valid_quantum.0].0}
    });
    return name;
}