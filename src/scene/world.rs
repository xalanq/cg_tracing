use super::sppm::{KDTree, Pixel, Point};
use crate::{
    geo::Material,
    geo::{Geo, HitResult},
    linalg::{Camera, Ray, Vct},
    utils::{clamp, Image, Rng},
    Deserialize, Flt, Serialize, EPS, PI,
};

use pbr::ProgressBar;
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;
use std::time;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PT {
    pub sample: usize,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SPPM {
    pub view_point_sample: usize,
    pub photon_sample: usize,
    pub radius: Flt,
    pub radius_decay: Flt,
    pub rounds: usize,
    pub light_pos: Vct,
    pub light_r: Flt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Renderer {
    PT(PT),
    SPPM(SPPM),
}

pub struct World {
    pub objs: Vec<Box<dyn Geo>>,
    pub camera: Camera,
    pub max_depth: usize,
    pub thread_num: usize,
    pub stack_size: usize,
    pub n1: Flt,
    pub n2: Flt,
    pub r0: Flt,
    pub renderer: Renderer,
}

impl World {
    pub fn new(
        camera: Camera,
        max_depth: usize,
        thread_num: usize,
        stack_size: usize,
        na: Flt,
        ng: Flt,
        renderer: Renderer,
    ) -> Self {
        Self {
            objs: Vec::new(),
            camera,
            max_depth,
            thread_num,
            stack_size,
            n1: na / ng,
            n2: ng / na,
            r0: ((na - ng) * (na - ng)) / ((na + ng) * (na + ng)),
            renderer,
        }
    }

    pub fn render(&self, p: &mut Image) {
        match self.renderer {
            Renderer::PT(cfg) => self.path_tracing(p, cfg),
            Renderer::SPPM(cfg) => self.stochastic_progressive_photon_mapping(p, cfg),
        };
    }

    fn gen(rng: &mut Rng) -> Flt {
        let r = 2.0 * rng.gen();
        if r < 1.0 {
            r.sqrt() - 1.0
        } else {
            1.0 - (2.0 - r).sqrt()
        }
    }

    pub fn add(&mut self, obj: Box<dyn Geo>) -> &mut Self {
        self.objs.push(obj);
        self
    }

    fn find<'a>(&'a self, r: &Ray) -> Option<HitResult> {
        let mut t: Flt = 1e30;
        let mut obj = None;
        let mut gg = None;
        self.objs.iter().for_each(|o| {
            if let Some(d) = o.hit_t(r) {
                if d.0 < t {
                    t = d.0;
                    gg = d.1;
                    obj = Some(o);
                }
            }
        });
        if let Some(o) = obj {
            return Some(o.hit(r, (t, gg)));
        }
        None
    }

    fn pt(&self, r: &Ray, mut depth: usize, rng: &mut Rng) -> Vct {
        if let Some(HitResult { pos, norm, ref texture }) = self.find(r) {
            depth += 1;
            if depth > self.max_depth {
                return texture.emission;
            }
            let mut color = texture.color;
            if depth > 5 {
                let p = color.x.max(color.y.max(color.z));
                if rng.gen() < p {
                    color /= p;
                } else {
                    return texture.emission;
                }
            }
            let mut ff = || {
                let nd = norm.dot(r.direct);
                if texture.material == Material::Diffuse {
                    let w = if nd < 0.0 { norm } else { -norm };
                    let (r1, r2) = (PI * 2.0 * rng.gen(), rng.gen());
                    let r2s = r2.sqrt();
                    let u = (if w.x.abs() <= 0.1 {
                        Vct::new(1.0, 0.0, 0.0)
                    } else {
                        Vct::new(0.0, 1.0, 0.0)
                    } % w)
                        .norm();
                    let v = w % u;
                    let d = (u * r1.cos() + v * r1.sin()) * r2s + w * (1.0 - r2).sqrt();
                    return self.pt(&Ray::new(pos, d.norm()), depth, rng);
                }
                let refl = Ray::new(pos, r.direct - norm * (2.0 * nd));
                if texture.material == Material::Specular {
                    return self.pt(&refl, depth, rng);
                }
                let w = if nd < 0.0 { norm } else { -norm };
                let (it, ddw) = (norm.dot(w) > 0.0, r.direct.dot(w));
                let (n, sign) = if it { (self.n1, 1.0) } else { (self.n2, -1.0) };
                let cos2t = 1.0 - n * n * (1.0 - ddw * ddw);
                if cos2t < 0.0 {
                    return self.pt(&refl, depth, rng);
                }
                let td = (r.direct * n - norm * ((ddw * n + cos2t.sqrt()) * sign)).norm();
                let refr = Ray::new(pos, td);
                let c = if it { 1.0 + ddw } else { 1.0 - td.dot(norm) };
                let cc = c * c;
                let re = self.r0 + (1.0 - self.r0) * cc * cc * c;
                let tr = 1.0 - re;
                if depth > 2 {
                    let p = 0.25 + 0.5 * re;
                    if rng.gen() < p {
                        self.pt(&refl, depth, rng) * (re / p)
                    } else {
                        self.pt(&refr, depth, rng) * (tr / (1.0 - p))
                    }
                } else {
                    self.pt(&refl, depth, rng) * re + self.pt(&refr, depth, rng) * tr
                }
            };
            return texture.emission + color * ff();
        }
        Vct::zero()
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn path_tracing(&self, p: &mut Image, cfg: PT) {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.thread_num)
            .stack_size(self.stack_size)
            .build()
            .unwrap();
        pool.install(|| {
        let (w, h) = (p.w, p.h);
        let (fw, fh) = (w as Flt, h as Flt);
        let cx = Vct::new(fw * self.camera.view_angle_scale / fh, 0.0, 0.0);
        let cy = (cx % self.camera.direct).norm() * self.camera.view_angle_scale;
        let sample = cfg.sample / 4;
        let inv = 1.0 / sample as Flt;
        let camera_direct = self.camera.direct.norm();
        let max_dim = camera_direct.x.abs().max(camera_direct.y.abs().max(camera_direct.z.abs()));
        let choose = if max_dim == camera_direct.x.abs() { 0 } else if max_dim == camera_direct.y.abs() { 1 } else { 2 };
        let mut pb = ProgressBar::new((w * h) as u64);
        pb.set_max_refresh_rate(Some(Duration::from_secs(1)));
        let mut data: Vec<(usize, usize)> = Vec::new();
        (0..w).for_each(|x| (0..h).for_each(|y| data.push((x, y))));
        data.shuffle(&mut rand::thread_rng());
        let pb = Mutex::new(pb);
        let p = Mutex::new(p);

        println!("w: {}, h: {}, sample: {}, actual sample: {}", w, h, cfg.sample, sample * 4);
        println!("Start rendering with {} threads.", pool.current_num_threads());
        let s_time = time::Instant::now();

        data.into_par_iter().for_each(|(x, y)| {
            let mut sum = Vct::zero();
            let (fx, fy) = (x as Flt, y as Flt);
            let mut rng = Rng::new((y * w + x) as u32);
            for sx in 0..2 {
                for sy in 0..2 {
                    let mut c = Vct::zero();
                    for _ in 0..sample {
                        let (fsx, fsy) = (sx as Flt, sy as Flt);
                        let ccx = cx * (((fsx + 0.5 + Self::gen(&mut rng)) / 2.0 + fx) / fw - 0.5);
                        let ccy = cy * (((fsy + 0.5 + Self::gen(&mut rng)) / 2.0 + fy) / fh - 0.5);
                        let rand_b = rng.gen() - 0.5;
                        let rand_a = rng.gen() - 0.5;
                        let d = camera_direct;
                        let r = if choose == 0 {
                            let (y, z) = (rand_a * d.y, rand_b * d.z);
                            Vct::new(-(y + z) / d.x, rand_a, rand_b)
                        } else if choose == 1 {
                            let (x, z) = (rand_a * d.x, rand_b * d.z);
                            Vct::new(rand_a, -(x + z) / d.y, rand_b)
                        } else {
                            let (x, y) = (rand_a * d.x, rand_b * d.y);
                            Vct::new(rand_a, rand_b, -(x + y) / d.z)
                        }.norm() * self.camera.aperture * rng.gen();
                        let d = ccx + ccy + d;
                        let o = self.camera.origin + r + d * self.camera.plane_distance;
                        let d = (d.norm() * self.camera.focal_distance - r).norm();
                        c += self.pt(&Ray::new(o, d), 0, &mut rng) * inv;
                    }
                    sum += Vct::new(clamp(c.x), clamp(c.y), clamp(c.z)) * 0.25;
                }
            }
            p.lock().unwrap().set(x, h - y - 1, sum);
            pb.lock().unwrap().inc();
        });
        pb.lock().unwrap().finish_println("...done\n");
        let mils = (time::Instant::now() - s_time).as_millis();
        let days = mils / 1000 / 60 / 60 / 24;
        let hours = mils / 1000 / 60 / 60 - days * 24;
        let mins = mils / 1000 / 60 - days * 24 * 60 - hours * 60;
        let secs = mils / 1000 - days * 24 * 60 * 60 - hours * 60 * 60 - mins * 60;
        println!("Total cost {}d {}h {}m {}s.", days, hours, mins, secs);
        });
    }

    fn sppm_1(
        &self,
        r: &Ray,
        mut depth: usize,
        rng: &mut Rng,
        points: &mut Vec<Point>,
        prod: Vct,
        index: usize,
    ) {
        if prod.x.max(prod.y.max(prod.z)) < EPS {
            return;
        }
        depth += 1;
        if depth > self.max_depth {
            return;
        }
        if let Some(HitResult { pos, norm, ref texture }) = self.find(r) {
            let mut color = texture.color;
            if depth > 5 {
                let p = color.x.max(color.y.max(color.z));
                if rng.gen() < p {
                    color /= p;
                } else {
                    return;
                }
            }
            let prod = prod * color;
            if texture.material == Material::Diffuse {
                points.push(Point::new(pos, norm, prod, index));
                return;
            }
            let nd = norm.dot(r.direct);
            let refl = Ray::new(pos, r.direct - norm * (2.0 * nd));
            if texture.material == Material::Specular {
                self.sppm_1(&refl, depth, rng, points, prod, index);
                return;
            }
            let w = if nd < 0.0 { norm } else { -norm };
            let (it, ddw) = (norm.dot(w) > 0.0, r.direct.dot(w));
            let (n, sign) = if it { (self.n1, 1.0) } else { (self.n2, -1.0) };
            let cos2t = 1.0 - n * n * (1.0 - ddw * ddw);
            if cos2t < 0.0 {
                self.sppm_1(&refl, depth, rng, points, prod, index);
                return;
            }
            let td = (r.direct * n - norm * ((ddw * n + cos2t.sqrt()) * sign)).norm();
            let refr = Ray::new(pos, td);
            let c = if it { 1.0 + ddw } else { 1.0 - td.dot(norm) };
            let cc = c * c;
            let re = self.r0 + (1.0 - self.r0) * cc * cc * c;
            let tr = 1.0 - re;
            if depth > 2 {
                let p = 0.25 + 0.5 * re;
                if rng.gen() < p {
                    self.sppm_1(&refl, depth, rng, points, prod * (re / p), index);
                } else {
                    self.sppm_1(&refr, depth, rng, points, prod * (tr / (1.0 - p)), index);
                }
            } else {
                self.sppm_1(&refl, depth, rng, points, prod * re, index);
                self.sppm_1(&refr, depth, rng, points, prod * tr, index);
            }
        }
    }

    fn sppm_2(
        &self,
        r: &Ray,
        mut depth: usize,
        rng: &mut Rng,
        tree: &KDTree,
        pixels: &mut Vec<Pixel>,
        prod: Vct,
    ) {
        if prod.x.max(prod.y.max(prod.z)) < EPS {
            return;
        }
        depth += 1;
        if depth > self.max_depth {
            return;
        }
        if let Some(HitResult { pos, norm, ref texture }) = self.find(r) {
            let mut color = texture.color;
            if depth > 5 {
                let p = color.x.max(color.y.max(color.z));
                if rng.gen() < p {
                    color /= p;
                } else {
                    return;
                }
            }
            let nd = norm.dot(r.direct);
            if texture.material == Material::Diffuse {
                tree.update(&pos, &norm, &prod, pixels);
                let w = if nd < 0.0 { norm } else { -norm };
                let (r1, r2) = (PI * 2.0 * rng.gen(), rng.gen());
                let r2s = r2.sqrt();
                let u = (if w.x.abs() <= 0.1 {
                    Vct::new(1.0, 0.0, 0.0)
                } else {
                    Vct::new(0.0, 1.0, 0.0)
                } % w)
                    .norm();
                let v = w % u;
                let d = (u * r1.cos() + v * r1.sin()) * r2s + w * (1.0 - r2).sqrt();
                self.sppm_2(&Ray::new(pos, d.norm()), depth, rng, tree, pixels, prod * color);
                return;
            }
            let prod = prod * color;
            let refl = Ray::new(pos, r.direct - norm * (2.0 * nd));
            if texture.material == Material::Specular {
                self.sppm_2(&refl, depth, rng, tree, pixels, prod);
                return;
            }
            let w = if nd < 0.0 { norm } else { -norm };
            let (it, ddw) = (norm.dot(w) > 0.0, r.direct.dot(w));
            let (n, sign) = if it { (self.n1, 1.0) } else { (self.n2, -1.0) };
            let cos2t = 1.0 - n * n * (1.0 - ddw * ddw);
            if cos2t < 0.0 {
                self.sppm_2(&refl, depth, rng, tree, pixels, prod);
                return;
            }
            let td = (r.direct * n - norm * ((ddw * n + cos2t.sqrt()) * sign)).norm();
            let refr = Ray::new(pos, td);
            let c = if it { 1.0 + ddw } else { 1.0 - td.dot(norm) };
            let cc = c * c;
            let re = self.r0 + (1.0 - self.r0) * cc * cc * c;
            let tr = 1.0 - re;
            if depth > 2 {
                let p = 0.25 + 0.5 * re;
                if rng.gen() < p {
                    self.sppm_2(&refl, depth, rng, tree, pixels, prod * (re / p));
                } else {
                    self.sppm_2(&refr, depth, rng, tree, pixels, prod * (tr / (1.0 - p)));
                }
            } else {
                self.sppm_2(&refl, depth, rng, tree, pixels, prod * re);
                self.sppm_2(&refr, depth, rng, tree, pixels, prod * tr);
            }
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn stochastic_progressive_photon_mapping(&self, p: &mut Image, cfg: SPPM) {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.thread_num)
            .stack_size(self.stack_size)
            .build()
            .unwrap();
        pool.install(|| {
        let (w, h) = (p.w, p.h);
        let (fw, fh) = (w as Flt, h as Flt);
        let cx = Vct::new(fw * self.camera.view_angle_scale / fh, 0.0, 0.0);
        let cy = (cx % self.camera.direct).norm() * self.camera.view_angle_scale;
        let thread_num = pool.current_num_threads();
        let sample = cfg.view_point_sample / 4;
        let camera_direct = self.camera.direct.norm();
        let max_dim = camera_direct.x.abs().max(camera_direct.y.abs().max(camera_direct.z.abs()));
        let choose = if max_dim == camera_direct.x.abs() { 0 } else if max_dim == camera_direct.y.abs() { 1 } else { 2 };

        let mut radius = cfg.radius;
        let radius_decay = cfg.radius_decay;
        let rounds = cfg.rounds;
        let photon_sample = cfg.photon_sample / thread_num;

        println!("w: {}, h: {}, view point sample: {}, actual sample: {}", w, h, cfg.view_point_sample, sample * 4);
        println!("photon samples: {}, total rounds: {}, init radius: {}, radius decay: {}", photon_sample * thread_num, rounds, radius, radius_decay);
        println!("Start rendering with {} threads.", pool.current_num_threads());
        let s_time = time::Instant::now();
        let mut final_pixel = vec![Pixel::default(); w * h];

        for iter in 0..rounds {
            println!("Round: {}, radius: {}", iter + 1, radius);
            let mut data: Vec<(usize, usize)> = Vec::new();
            (0..w).for_each(|x| (0..h).for_each(|y| data.push((x, y))));
            data.shuffle(&mut rand::thread_rng());
            let total_points = Mutex::new(vec![]);
            println!("Running sppm1");
            data.into_par_iter().for_each(|(x, y)| {
                let (fx, fy) = (x as Flt, y as Flt);
                let index = y * w + x;
                let mut rng = Rng::new((index + iter * w * h) as u32);
                let mut points = vec![];
                for sx in 0..2 {
                    for sy in 0..2 {
                        for _ in 0..sample {
                            let (fsx, fsy) = (sx as Flt, sy as Flt);
                            let ccx = cx * (((fsx + 0.5 + Self::gen(&mut rng)) / 2.0 + fx) / fw - 0.5);
                            let ccy = cy * (((fsy + 0.5 + Self::gen(&mut rng)) / 2.0 + fy) / fh - 0.5);
                            let rand_b = rng.gen() - 0.5;
                            let rand_a = rng.gen() - 0.5;
                            let d = camera_direct;
                            let r = if choose == 0 {
                                let (y, z) = (rand_a * d.y, rand_b * d.z);
                                Vct::new(-(y + z) / d.x, rand_a, rand_b)
                            } else if choose == 1 {
                                let (x, z) = (rand_a * d.x, rand_b * d.z);
                                Vct::new(rand_a, -(x + z) / d.y, rand_b)
                            } else {
                                let (x, y) = (rand_a * d.x, rand_b * d.y);
                                Vct::new(rand_a, rand_b, -(x + y) / d.z)
                            }.norm() * self.camera.aperture * rng.gen();
                            let d = ccx + ccy + d;
                            let o = self.camera.origin + r + d * self.camera.plane_distance;
                            let d = (d.norm() * self.camera.focal_distance - r).norm();
                            self.sppm_1(&Ray::new(o, d), 0, &mut rng, &mut points, Vct::one() * 0.25, index);
                        }
                    }
                }
                total_points.lock().unwrap().append(&mut points);
            });
            println!("...done");
            let mut total_points = total_points.lock().unwrap();
            println!("Total view points: {}", total_points.len());

            println!("Building tree");
            let tree = KDTree::new(&mut total_points, radius);
            println!("...done");

            let mut pb = ProgressBar::new((photon_sample * thread_num) as u64);
            pb.set_max_refresh_rate(Some(Duration::from_secs(1)));
            let pb = Mutex::new(pb);
            let total_pixel = Mutex::new(vec![Pixel::default(); w * h]);
            println!("Running sppm2");
            (0..thread_num).into_par_iter().for_each(|index| {
                let mut pixels = vec![Pixel::default(); w * h];
                let mut rng = Rng::new((iter * thread_num + index) as u32);
                for i in 1..=photon_sample {
                    let ang = rng.gen() * PI * 2.0;
                    let r = rng.gen() * cfg.light_r;
                    let o = Vct::new(cfg.light_pos.x + r * ang.cos(), cfg.light_pos.y, cfg.light_pos.z + r * ang.sin());
                    let t1 = rng.gen() * PI * 2.0;
                    let t2 = rng.gen() * PI * 2.0;
                    let mut d = Vct::new(t1.sin() * t2.cos(), t1.sin() * t2.sin(), t1.cos()).norm();
                    if d.y < 0.0 {
                        d.y = -d.y;
                    }
                    self.sppm_2(&Ray::new(o, d), 0, &mut rng, &tree, &mut pixels, Vct::one() * 8.0);
                    if i % 100 == 0 {
                        pb.lock().unwrap().add(100);
                    }
                }
                let mut total = total_pixel.lock().unwrap();
                for i in 0..pixels.len() {
                    total[i].add(pixels[i].col, pixels[i].sum);
                }
                pb.lock().unwrap().add(photon_sample as u64 % 100);
            });
            pb.lock().unwrap().finish_println("...done\n");

            let total = total_pixel.lock().unwrap();
            for i in 0..total.len() {
                final_pixel[i].add(total[i].get(), 1.0);
            }
            radius *= radius_decay;

            for x in 0..w {
                for y in 0..h {
                    p.set(x, h - y - 1, final_pixel[y * w + x].get());
                }
            }
            p.save_png(&format!("./result/test/test_{}.png", iter));
        }

        for x in 0..w {
            for y in 0..h {
                p.set(x, h - y - 1, final_pixel[y * w + x].get());
            }
        }

        let mils = (time::Instant::now() - s_time).as_millis();
        let days = mils / 1000 / 60 / 60 / 24;
        let hours = mils / 1000 / 60 / 60 - days * 24;
        let mins = mils / 1000 / 60 - days * 24 * 60 - hours * 60;
        let secs = mils / 1000 - days * 24 * 60 * 60 - hours * 60 * 60 - mins * 60;
        println!("Total cost {}d {}h {}m {}s.", days, hours, mins, secs);
        });
    }
}
