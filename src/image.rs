use crate::{
    utils::{to_byte, Flt},
    vct::Vct,
};
use image;
use std::fs::File;
use std::io::Write;

pub type RGBA = (Flt, Flt, Flt, Flt);

#[derive(Clone, Debug, Default)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub c: Vec<RGBA>,
}

impl Image {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, c: vec![(0.0, 0.0, 0.0, 0.0); w * h] }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Vct) {
        self.c[y * self.w + x] = (c.x, c.y, c.z, 0.0);
    }

    pub fn get(&self, x: usize, y: usize) -> RGBA {
        self.c[y * self.w + x]
    }

    pub fn get_repeat(&self, x: isize, y: isize) -> RGBA {
        let (w, h) = (self.w as isize, self.h as isize);
        let mut y = y % h;
        let mut x = x % w;
        if y < 0 {
            y += h
        }
        if x < 0 {
            x += w
        }
        self.c[(y * w + x) as usize]
    }

    pub fn save_ppm(&self, path: &str) {
        println!("Writing to {}", path);
        let errmsg = &format!("cannot save PPM to {}", path);
        let mut file = File::create(path).expect(errmsg);
        let mut data = String::new();
        data.push_str(&format!("P3\n{} {}\n255\n", self.w, self.h));
        self.c.iter().for_each(|t| {
            data.push_str(&format!("{} {} {} ", to_byte(t.0), to_byte(t.1), to_byte(t.2)));
        });
        file.write_all(data.as_bytes()).expect(errmsg);
        file.flush().expect(errmsg);
        println!("Done!");
    }

    pub fn save_png(&self, path: &str) {
        println!("Writing to {}", path);
        let mut imgbuf = image::ImageBuffer::new(self.w as u32, self.h as u32);
        let mut it = self.c.iter();
        for p in imgbuf.pixels_mut() {
            if let Some(&t) = it.next() {
                *p = image::Rgb([to_byte(t.0), to_byte(t.1), to_byte(t.2)]);
            }
        }
        imgbuf.save(&path).expect(&format!("cannot save PNG to {}", path));
        println!("Done!");
    }
}
