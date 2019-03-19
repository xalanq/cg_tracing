use crate::{
    geo::Material,
    geo::{Geo, HitResult},
    linalg::{Camera, Ray, Vct},
    utils::{clamp, Image, Rng},
    Flt, PI,
};
use pbr::ProgressBar;
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;
use std::time;
use std::time::Duration;

pub struct World {
    pub objs: Vec<Box<dyn Geo>>,
    pub camera: Camera,
    pub sample: usize,
    pub max_depth: usize,
    pub thread_num: usize,
    pub stack_size: usize,
    pub n1: Flt,
    pub n2: Flt,
    pub r0: Flt,
}

impl World {
    pub fn new(
        camera: Camera,
        sample: usize,
        max_depth: usize,
        thread_num: usize,
        stack_size: usize,
        na: Flt,
        ng: Flt,
    ) -> Self {
        Self {
            objs: Vec::new(),
            camera,
            sample,
            max_depth,
            thread_num,
            stack_size,
            n1: na / ng,
            n2: ng / na,
            r0: ((na - ng) * (na - ng)) / ((na + ng) * (na + ng)),
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

    fn trace(&self, r: &Ray, mut depth: usize, rng: &mut Rng) -> Vct {
        if let Some(HitResult { pos, norm, ref texture }) = self.find(r) {
            let mut color = texture.color;
            depth += 1;
            if depth > self.max_depth {
                return texture.emission;
            }
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
                    return self.trace(&Ray::new(pos, d.norm()), depth, rng);
                }
                let refl = Ray::new(pos, r.direct - norm * (2.0 * nd));
                if texture.material == Material::Specular {
                    return self.trace(&refl, depth, rng);
                }
                let w = if nd < 0.0 { norm } else { -norm };
                let (it, ddw) = (norm.dot(w) > 0.0, r.direct.dot(w));
                let (n, sign) = if it { (self.n1, 1.0) } else { (self.n2, -1.0) };
                let cos2t = 1.0 - n * n * (1.0 - ddw * ddw);
                if cos2t < 0.0 {
                    return self.trace(&refl, depth, rng);
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
                        self.trace(&refl, depth, rng) * (re / p)
                    } else {
                        self.trace(&refr, depth, rng) * (tr / (1.0 - p))
                    }
                } else {
                    self.trace(&refl, depth, rng) * re + self.trace(&refr, depth, rng) * tr
                }
            };
            return texture.emission + color * ff();
        }
        Vct::zero()
    }

    fn gend(rng: &mut Rng) -> Flt {
        let r = 2.0 * rng.gen();
        if r < 1.0 {
            r.sqrt() - 1.0
        } else {
            1.0 - (2.0 - r).sqrt()
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn render(&self, p: &mut Image) {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.thread_num)
            .stack_size(self.stack_size)
            .build()
            .unwrap();
        pool.install(|| {
        let (w, h) = (p.w, p.h);
        let (fw, fh) = (w as Flt, h as Flt);
        let cx = Vct::new(fw * self.camera.ratio / fh, 0.0, 0.0);
        let cy = (cx % self.camera.direct).norm() * self.camera.ratio;
        let sample = self.sample / 4;
        let inv = 1.0 / sample as Flt;
        let mut pb = ProgressBar::new((w * h) as u64);
        pb.set_max_refresh_rate(Some(Duration::from_secs(1)));
        let mut data: Vec<(usize, usize)> = Vec::new();
        (0..w).for_each(|x| (0..h).for_each(|y| data.push((x, y))));
        data.shuffle(&mut rand::thread_rng());
        let pb = Mutex::new(pb);
        let p = Mutex::new(p);

        println!("w: {}, h: {}, sample: {}, actual sample: {}", w, h, self.sample, sample * 4);
        println!("start render with {} threads.", self.thread_num);
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
                        let ccx = cx * (((fsx + 0.5 + Self::gend(&mut rng)) / 2.0 + fx) / fw - 0.5);
                        let ccy = cy * (((fsy + 0.5 + Self::gend(&mut rng)) / 2.0 + fy) / fh - 0.5);
                        let d = ccx + ccy + self.camera.direct;
                        let r = Ray::new(self.camera.origin + d * 130.0, d.norm());
                        c += self.trace(&r, 0, &mut rng) * inv;
                    }
                    sum += Vct::new(clamp(c.x), clamp(c.y), clamp(c.z)) * 0.25;
                }
            }
            p.lock().unwrap().set(x, h - y - 1, sum);
            pb.lock().unwrap().inc();
        });
        pb.lock().unwrap().finish_println("Rendering completed\n");
        let mils = (time::Instant::now() - s_time).as_millis();
        let days = mils / 1000 / 60 / 60 / 24;
        let hours = mils / 1000 / 60 / 60 - days * 24;
        let mins = mils / 1000 / 60 - days * 24 * 60 - hours * 60;
        let secs = mils / 1000 - days * 24 * 60 * 60 - hours * 60 * 60 - mins * 60;
        println!("Total cost {}d {}h {}m {}s.", days, hours, mins, secs);
        });
    }
}
