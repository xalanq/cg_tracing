use crate::{linalg::Vct, utils::Image, Deserialize, Flt, Serialize};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use std::fmt;

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

#[derive(Clone, Debug)]
pub struct TextureImage {
    pub path: String,
    pub material: Material,
    pub width_ratio: Flt,
    pub height_ratio: Flt,
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
        let image = Self::load(&path);
        Self {
            path,
            material,
            image,
            width_ratio: 1.0 / width_ratio,
            height_ratio: 1.0 / height_ratio,
        }
    }

    pub fn load(path: &str) -> Image {
        let img = image::open(path).expect(&format!("Cannot open {}", path));
        let (w, h) = (img.width(), img.height());
        let mut image = Image::new(w as usize, h as usize);
        for (x, y, p) in img.pixels() {
            image.c[((h - 1 - y) * w + x) as usize] = (
                p.data[0] as Flt / 255.0,
                p.data[1] as Flt / 255.0,
                p.data[2] as Flt / 255.0,
                p.data[3] as Flt / 255.0,
            );
        }
        image
    }
}

impl Serialize for TextureImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("texture_image", 4)?;
        s.serialize_field("path", &self.path)?;
        s.serialize_field("material", &self.material)?;
        s.serialize_field("width_ratio", &self.width_ratio)?;
        s.serialize_field("height_ratio", &self.height_ratio)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for TextureImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TextureImageVisitor;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Path,
            Material,
            WidthRatio,
            HeightRatio,
        }

        impl<'de> Visitor<'de> for TextureImageVisitor {
            type Value = TextureImage;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("TextureImage")
            }

            fn visit_map<V>(self, mut map: V) -> Result<TextureImage, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut path = None;
                let mut material = None;
                let mut width_ratio = None;
                let mut height_ratio = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Material => {
                            if material.is_some() {
                                return Err(de::Error::duplicate_field("material"));
                            }
                            material = Some(map.next_value()?);
                        }
                        Field::WidthRatio => {
                            if width_ratio.is_some() {
                                return Err(de::Error::duplicate_field("width_ratio"));
                            }
                            width_ratio = Some(map.next_value()?);
                        }
                        Field::HeightRatio => {
                            if height_ratio.is_some() {
                                return Err(de::Error::duplicate_field("height_ratio"));
                            }
                            height_ratio = Some(map.next_value()?);
                        }
                    }
                }
                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
                let material = material.ok_or_else(|| de::Error::missing_field("material"))?;
                let width_ratio =
                    width_ratio.ok_or_else(|| de::Error::missing_field("width_ratio"))?;
                let height_ratio =
                    height_ratio.ok_or_else(|| de::Error::missing_field("height_ratio"))?;
                Ok(TextureImage::new(path, material, width_ratio, height_ratio))
            }
        }

        deserializer.deserialize_map(TextureImageVisitor {})
    }
}
