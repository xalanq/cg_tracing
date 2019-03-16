use crate::{utils::to_byte, vct::Vct};
use image;
use pbr::ProgressBar;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Pic {
    pub w: usize,
    pub h: usize,
    pub c: Vec<(u8, u8, u8, u8)>,
}

impl Pic {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, c: vec![(0, 0, 0, 0); w * h] }
    }

    pub fn set(&mut self, x: usize, y: usize, c: &Vct) {
        self.c[y * self.w + x] = (to_byte(c.x), to_byte(c.y), to_byte(c.z), 0);
    }

    pub fn get(&self, x: isize, y: isize) -> (u8, u8, u8, u8) {
        let (w, h) = (self.w as isize, self.h as isize);
        let mut y = y % h;
        let mut x = x % w;
        if y < 0 {
            y += h
        }
        if x < 0 {
            x += w
        }
        self.c[((h - y - 1) * w + x) as usize]
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
            data.push_str(&format!("{} {} {} ", t.0, t.1, t.2));
        });
        file.write_all(data.as_bytes()).expect(errmsg);
        file.flush().expect(errmsg);
        pb.inc();
        pb.finish_println("Done!\n");
    }

    pub fn save_png(&self, filename: &str) {
        println!("Writing to {}", filename);
        let mut imgbuf = image::ImageBuffer::new(self.w as u32, self.h as u32);
        let mut pb = ProgressBar::new((self.c.len() + 2) as u64);
        pb.set_max_refresh_rate(Some(Duration::from_secs(1)));
        pb.inc();
        let mut it = self.c.iter();
        for p in imgbuf.pixels_mut() {
            if let Some(&t) = it.next() {
                *p = image::Rgb([t.0, t.1, t.2]);
            }
            pb.inc();
        }
        imgbuf.save(&filename).expect(&format!("cannot save PNG to {}", filename));
        pb.inc();
        pb.finish_println("Done!\n");
    }
}

impl Default for Pic {
    fn default() -> Self {
        Self { w: 0, h: 0, c: Vec::new() }
    }
}
