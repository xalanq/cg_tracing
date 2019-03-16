use crate::geo::*;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub t: TextureRaw,
    pub filename: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub pos: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub norm: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub root: Node,
}

impl Geo for Mesh {
    fn init(&mut self) {
        let file = File::open(&self.filename).expect(&format!("Cannot open {}", self.filename));
        let mut tri: Vec<(usize, usize, usize)> = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.expect("Failed to load mesh object");
            let mut ws = line.split_whitespace();
            macro_rules! nxt {
                ($w:expr) => {
                    $w.next().unwrap().parse().unwrap()
                };
            }
            match ws.next() {
                Some("v") => self.pos.push(Vct::new(nxt!(ws), nxt!(ws), nxt!(ws))),
                Some("vn") => self.norm.push(Vct::new(nxt!(ws), nxt!(ws), nxt!(ws))),
                Some("f") => {
                    let mut p = ws.next().unwrap().split('/');
                    tri.push((nxt!(p), nxt!(p), nxt!(p)));
                }
                _ => (),
            }
        }
    }
    fn hit_t(&self, r: &Ray) -> Option<Flt> {
        None
    }
    fn hit(&self, r: &Ray, t: Flt) -> HitResult {
        HitResult {
            pos: Vct::zero(),
            norm: Vct::zero(),
            texture: TextureRaw::new(Vct::zero(), Vct::zero(), Material::Diffuse),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Range {
    pub l: Vct,
    pub r: Vct,
}

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub range: Range,
}
