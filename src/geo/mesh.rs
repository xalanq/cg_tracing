use crate::geo::*;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub coord: Coord,
    pub path: String,
    pub texture: TextureRaw,
    #[serde(skip_serializing, skip_deserializing)]
    pub pos: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub norm: Vec<Vct>,
    #[serde(skip_serializing, skip_deserializing)]
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
pub struct Cube {
    pub min: Vct,
    pub max: Vct,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub cube: Cube,
    pub data: NodeKind,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    A((usize, usize)),
    B((usize, usize, usize)),
}

impl Cube {
    fn hit(&self, origin: Vct, inv_direct: Vct) -> Option<(Flt, Flt)> {
        let a = (self.min - origin) * inv_direct;
        let b = (self.max - origin) * inv_direct;
        let min = a.min(b);
        let max = a.max(b);
        let t_min = min.x.max(min.y).max(min.z).max(0.0);
        let t_max = max.x.min(max.y).min(max.z);
        if t_min < t_max {
            Some((t_min, t_max))
        } else {
            None
        }
    }
}

impl Mesh {
    fn new_node(&mut self, tri: &mut [(usize, usize, usize)]) -> usize {
        let (pos, norm) = (&self.pos, &self.norm);
        let cube = {
            let mut min = Vct::new(1e30, 1e30, 1e30);
            let mut max = Vct::new(-1e30, -1e30, -1e30);
            tri.iter().for_each(|(a, b, c)| {
                min = min.min(pos[*a]).min(pos[*b]).min(pos[*c]);
                max = max.max(pos[*a]).max(pos[*b]).max(pos[*c]);
            });
            Cube { min, max }
        };
        if tri.len() == 1 {
            self.nodes.push(Node { cube, data: NodeKind::B(tri[0]) });
            return self.nodes.len() - 1;
        }
        0
    }
}

impl Geo<(u32)> for Mesh {
    fn init(&mut self) {
        let file = File::open(&self.path).expect(&format!("Cannot open {}", self.path));
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
        self.new_node(&mut tri);
    }

    fn hit_t(&self, r: &Ray) -> Option<(Flt, Option<(u32)>)> {
        None
    }

    fn hit(&self, r: &Ray, t: (Flt, Option<(u32)>)) -> HitResult {
        HitResult {
            pos: Vct::zero(),
            norm: Vct::zero(),
            texture: TextureRaw::new(Vct::zero(), Vct::zero(), Material::Diffuse),
        }
    }
}
