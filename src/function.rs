
use std::{f32::consts::{PI}};

use crate::{voxel::{LENGTH, THRESHOLD}, camera::Camera, orbitals::Orbital};

use instant::{Instant, Duration};
use crate::instance;

//ORBITAL FUNCTION – Instances the given function with a resolution and a size [LENGTH]
    pub fn orbital(resolution: f32, bohr: f32, function_index: i16, orbital_array: &Vec<Orbital>, camera: &Camera) -> (Vec<instance::Instance>, i16) {

    //Variable instancing
        let mut new_instances: Vec<instance::Instance> = vec![];

        let mut new_function_index: i16 = LENGTH * resolution as i16;

        let mut bias: (f32, f32, f32) = (0.0, 0.0, 0.0);

        let mut ignore: (bool, bool, bool) = (false, false, false);

        let start = Instant::now();

    //Instancing loop – checks every possible coordinate and decides whether to create a voxel or not
        ((function_index + 1)..=((LENGTH + 1) * resolution as i16)).try_for_each (|a| {
        ((-LENGTH * resolution as i16)..=((LENGTH + 1) * resolution as i16)).try_for_each (|b| {
        ((-LENGTH * resolution as i16)..=((LENGTH + 1) * resolution as i16)).try_for_each (|c| {

            let mut x = 0;
            let mut y = 0;
            let mut z = 0;

            if b == LENGTH * resolution as i16 && c == LENGTH * resolution as i16{
                new_function_index = a;
            }  

        //Faces get ordered from near to far, that way it renders properly
            if (camera.position.x - camera.target.x).abs() > (camera.position.z - camera.target.z).abs() {
                match (camera.position.x - camera.target.x).signum() as i8 {
                    1 => {x = a} -1 => {x = -a} _ => {}
                }
                match (camera.position.z - camera.target.z).signum() as i8 {
                    1 => {z = -b} -1 => {z = b} _ => {}
                }  
            } else if (camera.position.z - camera.target.z).abs() > (camera.position.x - camera.target.x).abs() {
                x = b;
                match (camera.position.z - camera.target.z).signum() as i8 {
                    1 => {z = a} -1 => {z = -a} _ => {}
                }
            } match (camera.position.y - camera.target.y).signum() as i8 {
                1 => {y = c} -1 => {y = -c} _ => {}
            }

        //Faces change so that they are always closer to you.
            match (camera.position.x - (x as f32 - ((resolution - 1.0) / 2.0)) / resolution).signum() as i8 {
                1 => {bias.0 = 1.0} -1 => {bias.0 = -1.0} _ => {}
            }
            match (camera.position.y - (y as f32 - ((resolution - 1.0) / 2.0)) / resolution).signum() as i8 {
                1 => {bias.1 = 1.0} -1 => {bias.1 = -1.0} _ => {}
            }
            match (camera.position.z - (z as f32 - ((resolution - 1.0) / 2.0)) / resolution).signum() as i8 {
                1 => {bias.2 = 1.0} -1 => {bias.2 = -1.0} _ => {}
            }

        //Calculate the alpha value at each voxel with the CALC_FUNCTION function
            let result = calc_function(resolution, bohr, (x as f32 - ((resolution - 1.0) / 2.0)) / resolution, (y as f32 - ((resolution - 1.0) / 2.0)) / resolution, (z as f32 - ((resolution - 1.0) / 2.0)) / resolution, orbital_array);
            let alpha = if result.0 <= 1.0 {result.0} else if result.0 > 1.0 {1.0} else {0.0};
            let sign = result.1;

        //Renders only the faces that have more than a minimum alpha
            if alpha > THRESHOLD{
                ignore.0 = if (camera.position.x - (x as f32 - ((resolution - 1.0) / 2.0)) / resolution).abs() < 1.0 / ((resolution - 1.0) * 2.0) {true} else {false};
                ignore.1 = if (camera.position.y - (y as f32 - ((resolution - 1.0) / 2.0)) / resolution).abs() < 1.0 / ((resolution - 1.0) * 2.0) {true} else {false};
                ignore.2 = if (camera.position.z - (z as f32 - ((resolution - 1.0) / 2.0)) / resolution).abs() < 1.0 / ((resolution - 1.0) * 2.0) {true} else {false};
                new_instances.append(&mut instance::instantiate(resolution, x, y, z, alpha, sign, bias, ignore));
            }

        //Breakes the loop if it's taking too much, this way it can render things in multiple frames
            let now = Instant::now();
            if now - start >= Duration::new(0, 14000000) && b == LENGTH * resolution as i16 && c == LENGTH * resolution as i16 {
                None
            } else {Some(())}
        })
        })
        }); 

        return (new_instances, new_function_index);
    }

//CALC FUNCTION – Calculates the value of an orbital at a given coordinate
    pub fn calc_function(resolution: f32, bohr: f32, x: f32, y: f32, z: f32, orbital_array: &Vec<Orbital>) -> (f32, f32) {

        let mut calc_array: Vec<f32> = vec![];
        let size = bohr / 0.529 * 2.0;

    //Loop that calculates the wavefunction of each orbital
        orbital_array.into_iter().for_each(|orbital|{

        //Creates necessary variables
            //Radius from the center
            let r = ((x - orbital.position.x * size).powi(2) + (y - orbital.position.y * size).powi(2) + (z - orbital.position.z * size).powi(2)).sqrt();

            //Rotation cosines (what is usually cos(θ), sin(θ) and sin(φ) in actual formulae)
            let x_rot = (x - orbital.position.x * size)/r;
            let y_rot = (y - orbital.position.y * size)/r;
            let z_rot = (z - orbital.position.z * size)/r;

            //Each of the 4 components of the rotation quaternion
            let w = orbital.quaternion.0;
            let i = orbital.quaternion.1;
            let j = orbital.quaternion.2;
            let k = orbital.quaternion.3;

            //Normalization constant!
            let n = 1.0/PI.sqrt();

            //Converts true/false into 1/-1
            let phase = if orbital.phase == true {1.0} else {-1.0};

            //A bit that repeats a lot so it's condensed here for convenience
            let p = 1.0/(bohr * orbital.quantum.0 as f32);

            //This is the main part of every orbital. Compacted here to take less space
            let core = -phase as f32 * p.powi(3).sqrt() * (-p * r).exp();

        //This decides which orbital shall be rendered depending on its quantum numbers
            match (orbital.quantum.0, orbital.quantum.1) {
                
                //1s
                (1, 0) => {calc_array.push(n * core)}

                //2s
                (2, 0) => {calc_array.push(n * core * (1.0 - p * r))}

                //2p
                (2, 1) => {calc_array.push(n * core * p * r * ((x_rot * (w.powi(2) - i.powi(2) - j.powi(2) + k.powi(2))) +
                                                               2.0 * y_rot * (w * j - i * k) - 2.0 * z_rot * (w * i + j * k)))}

                //3s
                (3, 0) => {calc_array.push((2.0 / (32.0 as f32).sqrt()) * n * core * (6.0 - 12.0 * p * r + (2.0 * p * r).powi(2)))}

                //3p
                (3, 1) => {calc_array.push((2.0 / (4.5 as f32).sqrt()) * n * core * (2.0 - p * r) * p * r * ((x_rot * (w.powi(2) - i.powi(2) - j.powi(2) + k.powi(2))) +
                                                                                                              2.0 * y_rot * (w * j - i * k) - 2.0 * z_rot * (w * i + j * k)))}

                //3d - the if statement determines if it is a 3dz2 or all the other ones
                (3, 2) => {if orbital.magnetic == 0 {calc_array.push((2.0 / (31.0 as f32).sqrt()) * n * core * (p * r).powi(2) * (3.0 * ((x_rot * (w.powi(2) - i.powi(2) - j.powi(2) + k.powi(2))) +
                                                                                                                                          2.0 * y_rot * (w * j - i * k) - 2.0 * z_rot * (w * i + j * k)).powi(2) - 1.0));
                        } else {calc_array.push(n * core * (p * r).powi(2)

                             * ((x_rot * (w.powi(2) - i.powi(2) - j.powi(2) + k.powi(2))) +
                             2.0 * y_rot * (w * j - i * k) - 2.0 * z_rot * (w * i + j * k))
                            
                            * ((z_rot * (w.powi(2) - i.powi(2) + j.powi(2) - k.powi(2))) -
                            2.0 * y_rot * (w * k + i * j) + 2.0 * x_rot * (w * i - j * k)));
                }}
                _ => return   
            }
        });

        //Here we sum the orbitals so they combine. Also its all multiplied by some bits so the alpha and shape looks consistent across resolutions and sizes
        let mut calc: f32 = calc_array.iter().sum::<f32>().powi(2) * (bohr / 0.25).powi(2)  * (2.0 / resolution).sqrt();

        let mut sign: f32 = calc_array.iter().sum::<f32>() * (bohr / 0.25).powi(2) * (2.0 / resolution).sqrt();

        //Kind of a threshold
        if sign.abs() <= 0.01 {calc = 0.0}
        sign = sign.signum();

        return (calc, sign);
    }