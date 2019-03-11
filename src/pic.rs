use crate::{utils::to_byte, vct::Vct};
use pbr::ProgressBar;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Pic {
    pub w: usize,
    pub h: usize,
    pub c: Vec<Vct>,
}

impl Pic {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, c: vec![Vct::zero(); w * h] }
    }

    pub fn set(&mut self, x: usize, y: usize, c: &Vct) {
        self.c[y * self.w + x] = *c;
    }

    pub fn get(&self, x: usize, y: usize) -> Vct {
        let mut y = y % self.w;
        let mut x = x % self.h;
        if y < 0 {
            y += self.w
        }
        if x < 0 {
            x += self.h
        }
        self.c[(self.h - y + 1) * self.w + x]
    }

    pub fn save_ppm(&self, filename: &str) {
        println!("Writing to {}", filename);
        let errmsg = &format!("cannot save PPM to {}", filename);
        let mut file = File::create(filename).expect(errmsg);
        let mut pb = ProgressBar::new((self.c.len() + 2) as u64);
        let mut data = String::new();
        data.push_str(&format!("P3\n{} {}\n255\n", self.w, self.h));
        pb.set_max_refresh_rate(Some(Duration::from_secs(1)));
        pb.inc();
        self.c.iter().for_each(|t| {
            pb.inc();
            data.push_str(&format!("{} {} {} ", to_byte(t.x), to_byte(t.y), to_byte(t.z)));
        });
        file.write_all(data.as_bytes()).expect(errmsg);
        file.flush().expect(errmsg);
        pb.inc();
        pb.finish_println("Done!");
    }
}

impl Default for Pic {
    fn default() -> Self {
        Self { w: 0, h: 0, c: Vec::new() }
    }
}
