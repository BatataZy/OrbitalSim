
use crate::{voxel::{LENGTH, RES, THRESHOLD}};

use instant::{Instant, Duration};
use crate::instance;

pub fn orbital(function_index: i16, faces: &Vec<(i16, i16, i16)>, a: f32) -> (Vec<instance::Instance>, i16, Vec<(i16, i16, i16)>) {

    let mut new_instances: Vec<instance::Instance> = vec![];

    let mut new_function_index: i16 = LENGTH * RES as i16;

    let mut x_faces: Vec<(i16, i16, i16)> = faces.to_vec();
    let mut new_x_faces: Vec<(i16, i16, i16)> = vec![];

    let mut new_y_faces: Vec<(i16, i16, i16)> = vec![];
    let mut y_faces: Vec<(i16, i16, i16)> = vec![];

    let mut z_face: bool = false;

    let start = Instant::now();

    if function_index == (LENGTH * RES as i16) {} 
    
    else {
        ((function_index)..=(LENGTH * RES as i16)).try_for_each (|x| {
            ((-LENGTH * RES as i16)..=(LENGTH * RES as i16)).try_for_each (|y| {
                ((-LENGTH * RES as i16)..=(LENGTH * RES as i16)).try_for_each (|z| {

                    //let alpha = ((x + y + z + 9 * RES as i16) as f32 /(18.0 * RES)).powf(1.0);
                    let alpha = 2.0 / ((((x as f32 + a) - (RES-1.0) / 2.0).powf(2.0) + (y as f32 - (RES-1.0) / 2.0).powf(2.0) + (z as f32 - (RES-1.0) / 2.0).powf(2.0))/(RES*RES)).powf(3.0);

                    if y == LENGTH * RES as i16 && z == LENGTH * RES as i16{
                        x_faces.clear();
                        x_faces.append(&mut new_x_faces);
                    } 

                    if z == -LENGTH * RES as i16 {
                        y_faces.clear();
                        y_faces.append(&mut new_y_faces);
                    } 

                    if alpha > THRESHOLD {
                        new_instances.append(&mut instance::instantiate(x, y, z, alpha, &x_faces, &y_faces, z_face));
                        new_x_faces.push((x, y, z));
                        new_y_faces.push((x, y, z));
                        z_face = true;
                    }
                    else {z_face = false}

                    let now = Instant::now();

                    if now - start >= Duration::new(0, 6000000) && y == -LENGTH * RES as i16 && z == -LENGTH * RES as i16 {
                        println!("Instance Time: {:?}", now - start);
                        new_function_index = x;
                        None
                    } else {Some(())}
                })
            })
        }); 
    }

    //let end = Instant::now();

    //println!("{:?}", end - start);
    //println!("{:?}", instances.len());
        
    return (new_instances, new_function_index, x_faces);
}