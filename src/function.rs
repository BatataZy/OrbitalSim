
use crate::voxel::{LENGTH, RES, THRESHOLD};

use instant::{Instant, Duration};
use crate::instance;

pub fn orbital(function_index: i16, faces: &Vec<(i16, i16, i16)>, a: f32) -> (Vec<instance::Instance>, i16, Vec<(i16, i16, i16)>) {

    let mut new_instances: Vec<instance::Instance> = vec![];

    let mut new_function_index: i16 = LENGTH * RES as i16;

    let mut new_faces: Vec<(i16, i16, i16)> = faces.to_vec();

    let start = Instant::now();

    if function_index == (LENGTH * RES as i16) {} 
    
    else {
        ((function_index)..=(LENGTH * RES as i16)).try_for_each (|x| {
            ((-LENGTH * RES as i16)..=(LENGTH * RES as i16)).try_for_each (|y| {
                ((-LENGTH * RES as i16)..=(LENGTH * RES as i16)).try_for_each (|z| {

                    //let alpha = ((x + y + z + 9 * RES as i16) as f32 /(18.0 * RES)).powf(1.0);
                    let alpha = 2.0 / ((((x as f32 + a) - (RES-1.0) / 2.0).powf(2.0) + (y as f32 - (RES-1.0) / 2.0).powf(2.0) + (z as f32 - (RES-1.0) / 2.0).powf(2.0))/(RES*RES)).powf(3.0);

                    if alpha > THRESHOLD {
                        new_faces.push((x, y, z));
                        new_instances.append(&mut instance::instantiate(x, y, z, alpha, &new_faces));
                    }
                    else {}

                    let now = Instant::now();

                    if now - start >= Duration::new(0, 8000000) && y == -LENGTH * RES as i16 && z == -LENGTH * RES as i16 {
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
        
    return (new_instances, new_function_index, new_faces);
}