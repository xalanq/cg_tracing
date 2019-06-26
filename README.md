# CG Tracing

[![Build Status](https://travis-ci.org/xalanq/cg_tracing.svg?branch=master)](https://travis-ci.org/xalanq/cg_tracing)
[![license](https://img.shields.io/badge/license-MIT-%23373737.svg)](https://raw.githubusercontent.com/xalanq/cg_tracing/master/LICENSE)

A Rust implement of path tracing and ray tracing in computer graphics.

# Feature

- Fast: Render with multithreads (Use [rayon](https://github.com/rayon-rs/rayon/))
- Texture: Pin what you want!
- Expandable: Easy to add a geometric object (see example)
- Json: Build from json file, see example
- Structural: More OOP
- Progress bar: Use [a8m/pb](https://github.com/a8m/pb)

# Result

## Textures (sample 25000)

See [./result/result_1.json](./result/result_1.json)

![](./result/result_1.png)

## Mesh dragon.obj (sample 25000)

See [./result/result_2.json](./result/result_2.json)

![](./result/result_2.png)

## Rotate Surface by Bezier Curve (sample 25000)

See [./result/result_3.json](./result/result_3.json)

![](./result/result_3.png)

## Depth of field (sample 25000)

See [./result/result_4.json](./result/result_4.json)

![](./result/result_4.png)

See [./result/result_5.json](./result/result_5.json)

![](./result/result_5.png)

See [./result/result_6.json](./result/result_6.json)

![](./result/result_6.png)

# Bezier

$x(t), y(t)$
$$
k = \frac{y(t) - y_o}{y_d}, x(t) = \sqrt{(x_o + k x_d)^2 + (z_o + k z_d)^2}
$$

平方后两边乘$y_d^2$，有
$$
y_d^2 x^2(t) = [x_o y_d + (y(t) - y_o) x_d]^2 + [z_o y_d + (y(t) - y_o) z_d]^2
$$
则令

$$
\begin{align}
f(t) 
= & [x_o y_d + (y(t) - y_o) x_d]^2 + [z_o y_d + (y(t) - y_o) z_d]^2 - y_d^2 x^2(t) \\
= & (x_d^2 + z_d^2) y^2(t) + 2[(x_o y_d - y_o x_d) x_d + (z_o y_d - y_o z_d) z_d] y(t) + \\
& (x_o y_d - y_o x_d)^2 + (z_o y_d - y_o z_d)^2 - y_d^2 x^2(t) \\
= & a y^2(t) + b y(t) + c + w x^2(t)
\end{align}
$$

其中
$$
a = x_d^2 + z_d^2, \quad b = 2[(x_o y_d - y_o x_d) x_d + (z_o y_d - y_o z_d) z_d] \\
c = (x_o y_d - y_o x_d)^2 + (z_o y_d - y_o z_d)^2, \quad w = -y_d^2
$$
则

$$
f'(t) = 2 a y(t) y'(t) + b y'(t) + 2 w x(t) x'(t)
$$

牛顿迭代求出 $t$ 后，再推 $k$ 即可。但若 $y_d = 0$，则再解个方程即可（此时 $t$ 已经求出）。
$$
\begin{align}
&
(x_o + k x_d)^2 + (z_o + k z_d)^2 = x^2(t) \\
\Rightarrow &
(x_d^2 + z_d^2) k^2 + 2(x_o x_d + z_o z_d) k + x_o^2 + x_d^2 - x^2(t) = 0
\end{align}
$$

若 $x(t) \neq 0$ 法向量

$$
P(t, \theta) = (x(t) \cos \theta, y(t), x(t) \sin \theta)
$$

$$
\begin{align}
\frac{\partial P(t, \theta)}{\partial t} \times \frac{\partial P(t, \theta)}{\partial \theta}
& = (x'(t) \cos \theta, y'(t), x'(t) \sin \theta)) \times (-x(t) \sin \theta, 0, x(t) \cos \theta)
\end{align}
$$

当 $x(t) = 0$ 时，法向量直接为 $(0, -y_d, 0)$

# Usage

## example (from json, recommended)

see [./result/result_2.json](./result/result.json).

```rust
extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let (w, mut p) = utils::from_json("./result/result_2.json", register! {});
    w.render(&mut p);
    p.save_png(&format!("./result/result_2.png", w.sample));
}
```

## Add your geometric object

see [./src/geo/plane.rs](./src/geo/plane.rs)

```rust
use crate::{
    geo::{Geo, HitResult, HitTemp, Texture, TextureRaw},
    linalg::{Ray, Transform, Vct},
    Deserialize, Serialize, EPS,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub transform: Transform,
    pub texture: Texture,
}

impl Plane {
    pub fn new(texture: Texture, transform: Transform) -> Self {
        Self { texture, transform }
    }
}

impl Geo for Plane {
    // calculate t, which means r.origin + r.direct * t is the intersection point
    fn hit_t(&self, r: &Ray) -> Option<HitTemp> {
        let n = self.transform.z();
        let d = n.dot(r.direct);
        if d.abs() > EPS {
            let t = n.dot(self.transform.pos() - r.origin) / d;
            if t > EPS {
                return Some((t, None));
            }
        }
        None
    }

    // return the hit result
    fn hit(&self, r: &Ray, tmp: HitTemp) -> HitResult {
        let pos = r.origin + r.direct * tmp.0;
        let n = self.transform.z();
        HitResult {
            pos,
            norm: if n.dot(r.direct) > 0.0 { n } else { -n },
            texture: match self.texture {
                Texture::Raw(ref raw) => *raw,
                Texture::Image(ref img) => {
                    let v = pos - self.transform.pos();
                    let px = self.transform.x().dot(v) * img.width_ratio;
                    let py = self.transform.y().dot(v) * img.height_ratio;
                    let col = img.image.get_repeat(px as isize, py as isize);
                    TextureRaw {
                        emission: Vct::zero(),
                        color: Vct::new(col.0, col.1, col.2),
                        material: img.material,
                    }
                }
            },
        }
    }
}
```

if you want to use `from_json` with your object

```rust
use cg_tracing::prelude::*;
let (w, mut p) = cg_tracing::from_json("some.json", register! {
    "YourObject1" => YourObjectClass1,
    "YourObject2" => YourObjectClass2
});
```

# Reference

- [smallpt](http://www.kevinbeason.com/smallpt/)
- [go-tracing](https://github.com/xalanq/go-tracing)
