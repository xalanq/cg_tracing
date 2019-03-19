use crate::{linalg::Vct, utils::Image, Deserialize, Flt, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Material {
    Diffuse,
    Specular,
    Refractive,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TextureRaw {
    pub emission: Vct,
    pub color: Vct,
    pub material: Material,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureImage {
    pub path: String,
    pub material: Material,
    pub width_ratio: Flt,
    pub height_ratio: Flt,
    #[serde(skip_serializing, skip_deserializing)]
    pub image: Image,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Texture {
    Raw(TextureRaw),
    Image(TextureImage),
}

impl TextureRaw {
    pub fn new(emission: Vct, color: Vct, material: Material) -> Self {
        Self { emission, color, material }
    }
}

use image::GenericImageView;

impl TextureImage {
    pub fn new(path: String, material: Material, width_ratio: Flt, height_ratio: Flt) -> Self {
        Self { path, material, image: Image::default(), width_ratio, height_ratio }
    }

    pub fn load(&mut self) {
        let img = image::open(&self.path).expect(&format!("Cannot open {}", self.path));
        let (w, h) = (img.width(), img.height());
        self.image = Image::new(w as usize, h as usize);
        for (x, y, p) in img.pixels() {
            self.image.c[((h - 1 - y) * w + x) as usize] = (
                p.data[0] as Flt / 255.0,
                p.data[1] as Flt / 255.0,
                p.data[2] as Flt / 255.0,
                p.data[3] as Flt / 255.0,
            );
        }
        self.width_ratio = 1.0 / self.width_ratio;
        self.height_ratio = 1.0 / self.height_ratio;
    }
}
