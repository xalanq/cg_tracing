use crate::{utils::to_byte, vct::Vct};
use pbr::ProgressBar;
use std::fs::File;
use std::io::Write;

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
        self.c[y * self.h + x] = *c;
    }

    pub fn save_ppm(&self, filename: &str) {
        println!("Writing to {}", filename);
        let errmsg = &format!("cannot save PPM to {}", filename);
        let mut file = File::create(filename).expect(errmsg);
        let mut pb = ProgressBar::new((self.c.len() + 2) as u64);
        write!(file, "P3\n{} {}\n255\n", self.w, self.h).expect(errmsg);
        pb.inc();
        self.c.iter().for_each(|t| {
            pb.inc();
            write!(file, "{} {} {} ", to_byte(t.x), to_byte(t.y), to_byte(t.z)).expect(errmsg);
        });
        file.flush().expect(errmsg);
        pb.inc();
        println!("Done!");
    }
}
