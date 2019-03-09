extern crate cg_tracing;

fn main() {
    let (w, mut p) = cg_tracing::from_json("./example/smallpt.json");
    w.render(&mut p);
    p.save_ppm(&format!("./result/example_{}.ppm", w.sample));
}
