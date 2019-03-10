extern crate cg_tracing;

fn main() {
    let (w, mut p) = cg_tracing::from_json("smallpt.json");
    w.render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", w.sample));
}
