use crate::{
    linalg::{Mat, Vct},
    Deserialize, Flt, Serialize,
};
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransformType {
    Shift { x: Flt, y: Flt, z: Flt },
    Scale { x: Flt, y: Flt, z: Flt },
    Rotate { axis: String, degree: Flt },
    RotateRadian { axis: String, radian: Flt },
}

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub seq: Vec<TransformType>,
    pub value: Mat,
    pub inv: Mat,
}

impl Transform {
    pub fn new(seq: Vec<TransformType>) -> Self {
        let mut ret = Self { seq, value: Mat::identity(), inv: Mat::identity() };
        ret.compute();
        ret
    }

    pub fn compute(&mut self) {
        let mut ans = Mat::identity();
        self.seq.iter().for_each(|trans| {
            let m = match trans {
                TransformType::Shift { x, y, z } => Mat::shift(*x, *y, *z),
                TransformType::Scale { x, y, z } => Mat::scale(*x, *y, *z),
                TransformType::Rotate { axis, degree } => Mat::rot_degree(axis, *degree),
                TransformType::RotateRadian { axis, radian } => Mat::rot(axis, *radian),
            };
            ans = m * ans;
        });
        self.value = ans;

        let mut ans = Mat::identity();
        self.seq.iter().for_each(|trans| {
            let m = match trans {
                TransformType::Shift { x, y, z } => Mat::shift(-x, -y, -z),
                TransformType::Scale { x, y, z } => Mat::scale(1.0 / x, 1.0 / y, 1.0 / z),
                TransformType::Rotate { axis, degree } => Mat::rot_degree(axis, -degree),
                TransformType::RotateRadian { axis, radian } => Mat::rot(axis, -radian),
            };
            ans = ans * m;
        });
        self.inv = ans;
    }

    pub fn pos(&self) -> Vct {
        Vct::new(self.value.m03, self.value.m13, self.value.m23)
    }

    pub fn x(&self) -> Vct {
        Vct::new(self.value.m00, self.value.m10, self.value.m20)
    }

    pub fn y(&self) -> Vct {
        Vct::new(self.value.m01, self.value.m11, self.value.m21)
    }

    pub fn z(&self) -> Vct {
        Vct::new(self.value.m02, self.value.m12, self.value.m22)
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.seq.len()))?;
        for e in self.seq.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransformVisitor;

        impl<'de> Visitor<'de> for TransformVisitor {
            type Value = Transform;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of Transform")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Transform, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut tmp: Vec<TransformType> = Vec::new();

                while let Some(value) = seq.next_element()? {
                    tmp.push(value);
                }

                let mut trans = Transform::new(tmp);
                trans.compute();
                Ok(trans)
            }
        }

        deserializer.deserialize_seq(TransformVisitor {})
    }
}
