use crate::{linalg::Vct, Flt, EPS};

const ALPHA: Flt = 0.7;

#[derive(Clone, Debug, Default)]
pub struct Counter {
    pub sum: Vct,
    pub n: Flt,
}

impl Counter {
    pub fn add(&mut self, val: Vct, n: Flt) {
        self.sum += val;
        self.n += n;
    }

    pub fn get(&self) -> Vct {
        if self.n == 0.0 {
            return Vct::zero();
        }
        return self.sum / self.n;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Pixel {
    pub n: Flt,
    pub r2: Flt,
    pub flux: Vct,
}

impl Pixel {
    pub fn add(&mut self, flux: Vct) {
        let t = (self.n + ALPHA) / (self.n + 1.0);
        self.n += ALPHA;
        self.r2 = self.r2 * t;
        self.flux = (self.flux + flux) * t;
    }
}

#[derive(Clone, Debug)]
pub struct Point {
    pub pos: Vct,
    pub norm: Vct,
    pub col: Vct,
    pub index: usize,
}

impl Point {
    pub fn new(pos: Vct, norm: Vct, col: Vct, index: usize) -> Point {
        Point { pos, norm, col, index }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub data: Point,
    pub min: Vct,
    pub max: Vct,
    pub dim: usize,
    pub l: usize,
    pub r: usize,
}

#[derive(Clone, Debug, Default)]
pub struct KDTree {
    nodes: Vec<Node>,
    radius: Flt,
    r2: Flt,
}

impl KDTree {
    pub fn new(points: &mut Vec<Point>, radius: Flt) -> KDTree {
        let mut ret = KDTree { nodes: vec![], radius, r2: radius * radius };
        ret.new_node(0, points.len() - 1, points);
        ret
    }

    fn up(&mut self, x: usize, y: usize) {
        self.nodes[x].min = self.nodes[x].min.min(self.nodes[y].min);
        self.nodes[x].max = self.nodes[x].max.max(self.nodes[y].max);
    }

    fn dist2(&self, x: usize, pos: &Vct) -> Flt {
        Vct::zero().max(*pos - self.nodes[x].max).max(self.nodes[x].min - *pos).len2()
    }

    fn new_node(&mut self, l: usize, r: usize, p: &mut Vec<Point>) -> usize {
        let dim = {
            let (mut var, mut avg) = ([0.0; 3], [0.0; 3]);
            p[l..=r].iter().for_each(|a| {
                avg.iter_mut().enumerate().for_each(|(d, t)| *t += a.pos[d]);
            });
            avg.iter_mut().for_each(|a| *a /= p.len() as Flt);
            p[l..=r].iter().for_each(|a| {
                var.iter_mut().enumerate().for_each(|(d, t)| *t += (a.pos[d] - avg[d]).powi(2));
            });
            var.iter().enumerate().max_by(|x, y| x.1.partial_cmp(y.1).unwrap()).unwrap().0
        };
        let mid = (l + r) >> 1;
        pdqselect::select_by(&mut p[l..=r], mid - l, |a, b| {
            a.pos[dim].partial_cmp(&b.pos[dim]).unwrap()
        });
        self.nodes.push(Node {
            data: p[mid].clone(),
            min: p[mid].pos - self.radius,
            max: p[mid].pos + self.radius,
            dim,
            l: 0,
            r: 0,
        });
        let x = self.nodes.len() - 1;
        if l < mid {
            self.nodes[x].l = self.new_node(l, mid - 1, p);
            self.up(x, self.nodes[x].l);
        }
        if mid < r {
            self.nodes[x].r = self.new_node(mid + 1, r, p);
            self.up(x, self.nodes[x].r);
        }
        x
    }

    fn _update(&self, pos: &Vct, norm: &Vct, col: &Vct, pixels: &mut Vec<Pixel>, x: usize) {
        let data = &self.nodes[x].data;
        let len = (*pos - data.pos).len2();
        if len <= self.r2 && data.norm.dot(*norm) >= 0.0 {
            pixels[data.index].add(*col * data.col);
        }
        let (l, r) = (self.nodes[x].l, self.nodes[x].r);
        if l != 0 && self.dist2(l, pos) < EPS {
            self._update(pos, norm, col, pixels, l);
        }
        if r != 0 && self.dist2(r, pos) < EPS {
            self._update(pos, norm, col, pixels, r);
        }
    }

    pub fn update(&self, pos: &Vct, norm: &Vct, col: &Vct, pixels: &mut Vec<Pixel>) {
        self._update(pos, norm, col, pixels, 0);
    }
}
