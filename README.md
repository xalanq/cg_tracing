# CG Tracing

[![Build Status](https://travis-ci.org/xalanq/cg_tracing.svg?branch=master)](https://travis-ci.org/xalanq/cg_tracing)
[![license](https://img.shields.io/badge/license-MIT-%23373737.svg)](https://raw.githubusercontent.com/xalanq/cg_tracing/master/LICENSE)

A Rust implement of path tracing and ray tracing in computer graphics.

# Feature

- Very fast: Render with multithreads (Use [rayon](https://github.com/rayon-rs/rayon/))
- Expandable: Easy to add a geometric object
- Json build: Build from json file, see example
- Structural: More OOP
- Progress bar: Use [a8m/pb](https://github.com/a8m/pb)

# Usage

## example

see [./example/smallpt.rs](./example/smallpt.rs)

```rust
extern crate cg_tracing;

use cg_tracing::{geo::*, pic::*, ray::*, vct::*, world::*};

fn main() {
    let sample = 200;
    let z = Vct::zero();
    let (c1, c2) = (Vct::new(0.75, 0.25, 0.25), Vct::new(0.25, 0.25, 0.75));
    let (c3, c4) = (Vct::new(0.75, 0.75, 0.75), Vct::one() * 0.999);
    let mut p = Pic::new(1024, 768);
    let cam = Ray::new(Vct::new(50.0, 52.0, 295.6), Vct::new(0.0, -0.042612, -1.0));
    let max_depth = 10;
    let thread_num = 0; // if set 0. thread number is the number of CPUs available(logical cores).
    let stack_size = 256 * 1024 * 1024;
    let ratio = 0.5135;
    let (na, ng) = (1.0, 1.5);
    World::new(cam, sample, max_depth, thread_num, stack_size, ratio, na, ng)
        .add(Sphere::new(1e5, Geo::new(Vct::new(1e5 + 1., 40.8, 81.6), z, c1, Texture::Diffuse)))
        .add(Sphere::new(1e5, Geo::new(Vct::new(-1e5 + 99., 40.8, 81.6), z, c2, Texture::Diffuse)))
        .add(Sphere::new(1e5, Geo::new(Vct::new(50., 40.8, 1e5), z, c3, Texture::Diffuse)))
        .add(Sphere::new(1e5, Geo::new(Vct::new(50., 40.8, -1e5 + 170.0), z, z, Texture::Diffuse)))
        .add(Sphere::new(1e5, Geo::new(Vct::new(50., 1e5, 81.6), z, c3, Texture::Diffuse)))
        .add(Sphere::new(1e5, Geo::new(Vct::new(50., -1e5 + 81.6, 81.6), z, c3, Texture::Diffuse)))
        .add(Sphere::new(16.5, Geo::new(Vct::new(27., 16.5, 47.), z, c4, Texture::Specular)))
        .add(Sphere::new(16.5, Geo::new(Vct::new(73., 16.5, 78.), z, c4, Texture::Refractive)))
        .add(Sphere::new(
            600.,
            Geo::new(
                Vct::new(50., 681.6 - 0.27, 81.6),
                Vct::new(12., 12., 12.),
                z,
                Texture::Diffuse,
            ),
        ))
        .render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", sample));
}
```

## example (from json)

see [./src/main.rs](./src/main.rs) and [./example/smallpt.json](./example/smallpt.json)

```rust
extern crate cg_tracing;

fn main() {
    let (w, mut p) = cg_tracing::from_json("./example/smallpt.json");
    w.render(&mut p);
    p.save_ppm(&format!("./result/example_{}.ppm", w.sample));
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
        "type": "Sphere",
        "r": 100000.0,
        "g": {
            "position": { "x": 100001.0, "y": 40.8, "z": 81.6 },
            "emission": { "x": 0.0,      "y": 0.0,  "z": 0.0  },
            "color":    { "x": 0.75,     "y": 0.25, "z": 0.25 },
            "texture": "Diffuse"
        }
    }, {
/* snip */
/* see more details in ./example/smallpt.json */
    }, {
        "type": "Sphere",
        "r": 600,
        "g": {
            "position": { "x": 50.0, "y": 681.33, "z": 81.6  },
            "emission": { "x": 12.0, "y": 12.0,   "z": 12.0  },
            "color":    { "x": 0.0,  "y": 0.0,    "z": 0.0   },
            "texture": "Diffuse"
        }
    }]
}
```

# Reference

- [smallpt](http://www.kevinbeason.com/smallpt/)
- [go-tracing](https://github.com/xalanq/go-tracing)
