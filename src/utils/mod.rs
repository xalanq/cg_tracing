pub mod image;

pub use self::image::Image;

use crate::{
    geo::{
        collection::{Mesh, Plane, Sphere},
        Geo,
    },
    linalg::Camera,
    scene::World,
    Flt,
};
use pbr::ProgressBar;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::thread;

pub fn clamp(x: Flt) -> Flt {
    if x < 0.0 {
        0.0
    } else {
        if x > 1.0 {
            1.0
        } else {
            x
        }
    }
}

pub fn to_byte(x: Flt) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

pub struct Rng {
    pub seed: u32,
}

const INV32: Flt = 1.0 / std::u32::MAX as Flt;

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { seed: if seed == 0 { 233 } else { seed } }
    }

    pub fn gen(&mut self) -> Flt {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        x as Flt * INV32
    }
}

pub type FromJsonFunc = fn(Value) -> Box<dyn Geo>;

pub fn new_from_json<T: Geo + DeserializeOwned + 'static>(v: Value) -> Box<dyn Geo> {
    let obj = serde_json::from_value::<T>(v).expect("Invalid Value");
    Box::new(obj)
}

pub fn from_json(path: &str, custom: HashMap<String, FromJsonFunc>) -> (World, Image) {
    let data = fs::read_to_string(path).expect(&format!("Unable to read {}", path));
    let mut data: Value = serde_json::from_str(&data).expect("Cannot convert to json");
    let w: usize = serde_json::from_value(data["width"].take()).expect("Invalid width");
    let h: usize = serde_json::from_value(data["height"].take()).expect("Invalid height");
    let p = Image::new(w, h);
    let camera: Camera = serde_json::from_value(data["camera"].take()).expect("Invalid camera");
    let sample: usize = serde_json::from_value(data["sample"].take()).expect("Invalid sample");
    let max_depth: usize =
        serde_json::from_value(data["max_depth"].take()).expect("Invalid maximum depth");
    let thread_num: usize =
        serde_json::from_value(data["thread_num"].take()).expect("Invalid thread number");
    let stack_size: usize =
        serde_json::from_value(data["stack_size"].take()).expect("Invalid stack size");
    let na: Flt = serde_json::from_value(data["Na"].take()).expect("Invalid Na");
    let ng: Flt = serde_json::from_value(data["Ng"].take()).expect("Invalid Ng");
    let mut w = World::new(camera, sample, max_depth, thread_num, stack_size, na, ng);
    match data["objects"].take() {
        Value::Array(objs) => thread::Builder::new()
            .stack_size(stack_size)
            .spawn(move || {
                println!("Loading objects...");
                let mut pb = ProgressBar::new(objs.len() as u64);
                objs.into_iter().for_each(|_obj| {
                    let mut obj = _obj;
                    match obj["type"].take() {
                        Value::String(tp) => match tp.as_ref() {
                            "sphere" => w.add(new_from_json::<Sphere>(obj)),
                            "plane" => w.add(new_from_json::<Plane>(obj)),
                            "mesh" => w.add(new_from_json::<Mesh>(obj)),
                            _ => {
                                if let Some(f) = custom.get(&tp) {
                                    w.add(f(obj));
                                    return;
                                }
                                panic!("Unknown obj");
                            }
                        },
                        _ => panic!("Invalid obj"),
                    };
                    pb.inc();
                });
                pb.finish_println("Loaded.\n");
                (w, p)
            })
            .unwrap()
            .join()
            .unwrap(),
        _ => panic!("objs is not an array"),
    }
}
