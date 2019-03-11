extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let (w, mut p) = utils::from_json("./example/test.json", register! {});
    w.render(&mut p);
    p.save_ppm(&format!("./result/example_{}.ppm", w.sample));
}
