pub type Flt = f64;
pub const PI: Flt = std::f64::consts::PI as Flt;

pub mod geo;
pub mod pic;
pub mod ray;
pub mod utils;
pub mod vct;
pub mod world;

use geo::*;
use pic::Pic;
use ray::Ray;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use world::World;

pub type FromJsonFunc = fn(Value) -> Box<dyn Hittable>;

pub fn from_json(filename: &str, custom: Option<&HashMap<String, FromJsonFunc>>) -> (World, Pic) {
    let data = fs::read_to_string(filename).expect(&format!("Unable to read {}", filename));
    let mut data: Value = serde_json::from_str(&data).expect("Cannot convert to json");
    let w: usize = serde_json::from_value(data["width"].take()).expect("Invalid width");
    let h: usize = serde_json::from_value(data["height"].take()).expect("Invalid height");
    let p = Pic::new(w, h);
    let cam: Ray = serde_json::from_value(data["camera"].take()).expect("Invalid camera");
    let sample: usize = serde_json::from_value(data["sample"].take()).expect("Invalid sample");
    let max_depth: usize =
        serde_json::from_value(data["max_depth"].take()).expect("Invalid maximum depth");
    let thread_num: usize =
        serde_json::from_value(data["thread_num"].take()).expect("Invalid thread number");
    let stack_size: usize =
        serde_json::from_value(data["stack_size"].take()).expect("Invalid stack size");
    let ratio: Flt = serde_json::from_value(data["ratio"].take()).expect("Invalid ratio");
    let na: Flt = serde_json::from_value(data["Na"].take()).expect("Invalid Na");
    let ng: Flt = serde_json::from_value(data["Ng"].take()).expect("Invalid Ng");
    let mut w = world::World::new(cam, sample, max_depth, thread_num, stack_size, ratio, na, ng);
    match data["objects"].take() {
        Value::Array(objs) => {
            objs.into_iter().for_each(|_obj| {
                let mut obj = _obj;
                match obj["type"].take() {
                    Value::String(tp) => match tp.as_ref() {
                        "Sphere" => w.add(Sphere::from_json(obj)),
                        "Plane" => w.add(Plane::from_json(obj)),
                        _ => {
                            if let Some(mp) = custom {
                                if let Some(f) = mp.get(&tp) {
                                    w.add(f(obj));
                                    return;
                                }
                            }
                            panic!("Unknown obj");
                        }
                    },
                    _ => panic!("Invalid obj"),
                };
            });
            (w, p)
        }
        _ => panic!("objs is not an array"),
    }
}
