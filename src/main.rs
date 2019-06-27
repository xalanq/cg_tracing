extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let (w, mut p, path) = utils::from_json("./example/test.json", register! {});
    w.path_tracing(&mut p);
    p.save_png(&path);
}
