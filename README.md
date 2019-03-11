# CG Tracing

[![Build Status](https://travis-ci.org/xalanq/cg_tracing.svg?branch=master)](https://travis-ci.org/xalanq/cg_tracing)
[![license](https://img.shields.io/badge/license-MIT-%23373737.svg)](https://raw.githubusercontent.com/xalanq/cg_tracing/master/LICENSE)

A Rust implement of path tracing and ray tracing in computer graphics.

# Feature

- Very fast: Render with multithreads (Use [rayon](https://github.com/rayon-rs/rayon/))
- Expandable: Easy to add a geometric object (see example)
- Json build: Build from json file, see example
- Structural: More OOP
- Progress bar: Use [a8m/pb](https://github.com/a8m/pb)

# Usage

## example

see [./example/smallpt.rs](./example/smallpt.rs)

```rust
extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let mut p = Pic::new(1024, 768);
    let sample = 200;
    let z = vct!(0, 0, 0);
    let l = vct!(12, 12, 12);
    let (c1, c2) = (vct!(0.75, 0.25, 0.25), vct!(0.25, 0.25, 0.75));
    let (c3, c4) = (vct!(0.75, 0.75, 0.75), vct!(1, 1, 1) * 0.999);
    let cam = ray!(vct!(50.0, 52.0, 295.6), vct!(0.0, -0.042612, -1.0));
    let max_depth = 10;
    let thread_num = 0; // if set 0. thread number is the number of CPUs available(logical cores).
    let stack_size = 256 * 1024 * 1024;
    let ratio = 0.5135;
    let (na, ng) = (1.0, 1.5);
    World::new(cam, sample, max_depth, thread_num, stack_size, ratio, na, ng)
        .add(sphere!(vct!(1e5 + 1.0, 40.8, 81.6), 1e5, geo!(z, c1, Texture::Diffuse)))
        .add(sphere!(vct!(-1e5 + 99.0, 40.8, 81.6), 1e5, geo!(z, c2, Texture::Diffuse)))
        .add(sphere!(vct!(50.0, 40.8, 1e5), 1e5, geo!(z, c3, Texture::Diffuse)))
        .add(sphere!(vct!(50.0, 40.8, -1e5 + 170.0), 1e5, geo!(z, z, Texture::Diffuse)))
        .add(sphere!(vct!(50.0, 1e5, 81.6), 1e5, geo!(z, c3, Texture::Diffuse)))
        .add(sphere!(vct!(50.0, -1e5 + 81.6, 81.6), 1e5, geo!(z, c3, Texture::Diffuse)))
        .add(sphere!(vct!(27.0, 16.5, 47.0), 16.5, geo!(z, c4, Texture::Specular)))
        .add(sphere!(vct!(73.0, 16.5, 78.0), 16.5, geo!(z, c4, Texture::Refractive)))
        .add(sphere!(vct!(50.0, 681.33, 81.6), 600, geo!(l, z, Texture::Diffuse)))
        .render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", sample));
}
```

## example (from json)

see [./example/smallpt_json.rs](./example/smallpt_json.rs) and [./example/smallpt.json](./example/smallpt.json)

```rust
extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let (w, mut p) = utils::from_json("smallpt.json", register! {});
    w.render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", w.sample));
}
```

```json
{
    "width": 1024,
    "height": 768,
    "sample": 200,
    "thread_num": 0,
    "stack_size": 267386880,
    "max_depth": 10,
    "ratio": 0.5135,
    "Na": 1.0,
    "Ng": 1.5,
    "camera": {
        "origin": { "x": 50.0, "y": 52.0,      "z": 295.6 },
        "direct": { "x": 0.0,  "y": -0.042612, "z": -1.0  }
    },
    "objects": [{
        "type": "Plane",
        "p": { "x": 1.0, "y": 0.0, "z": 0.0 },
        "n": { "x": 1.0, "y": 0.0, "z": 0.0 },
        "g": {
            "emission": { "x": 0.0,  "y": 0.0,  "z": 0.0  },
            "color":    { "x": 0.75, "y": 0.25, "z": 0.25 },
            "texture": "Diffuse"
        }
    }, {
      //snip
    }, {
        "type": "Sphere",
        "c": { "x": 50.0, "y": 681.33, "z": 81.6 },
        "r": 600,
        "g": {
            "emission": { "x": 12.0, "y": 12.0, "z": 12.0 },
            "color":    { "x": 0.0,  "y": 0.0,  "z": 0.0  },
            "texture": "Diffuse"
        }
    }]
}
```

## Add your geometric object

see [./src/geo/plane.rs](./src/geo/plane.rs)

```rust
use crate::{geo::*, ray::*, utils::*};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub p: Vct, // any point at plane
    pub n: Vct, // normal vector of plane
    pub g: Geo, // geometric info
}

impl Plane {
    pub fn new(p: Vct, n: Vct, g: Geo) -> Box<dyn Hittable> {
        Box::new(Self { p, n, g })
    }
}

impl Hittable for Plane {
    // calculate intersection point value t, which means r.origin + r.direct * t is that point
    fn hit_t(&self, r: &Ray) -> Option<Flt> {
        let d = self.n.dot(&r.direct);
        if d.abs() > EPS {
            let t = self.n.dot(&(self.p - r.origin)) / d;
            if t > EPS {
                return Some(t);
            }
        }
        None
    }

    // return geo, hit position, normal vector
    fn hit(&self, r: &Ray, t: Flt) -> (&Geo, Vct, Vct) {
        (
            &self.g,
            r.origin + r.direct * t,
            if self.n.dot(&r.direct) > 0.0 { self.n } else { -self.n },
        )
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
