# CG Tracing

[![Build Status](https://travis-ci.org/xalanq/cg_tracing.svg?branch=master)](https://travis-ci.org/xalanq/cg_tracing)
[![license](https://img.shields.io/badge/license-MIT-%23373737.svg)](https://raw.githubusercontent.com/xalanq/cg_tracing/master/LICENSE)

A Rust implement of path tracing and ray tracing in computer graphics.

# Feature

- Very fast: Render with multithreads (Use [rayon](https://github.com/rayon-rs/rayon/))
- Structural: More OOP
- Expandable: Easy to add a geometric object
- Progress bar: Use [a8m/pb](https://github.com/a8m/pb)

# Usage

```rust
extern crate cg_tracing;

use cg_tracing::geo::sphere::Sphere;
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
    World::new(cam, sample, max_depth, thread_num, stack_size, ratio, 1.0, 1.5)
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
    p.save_ppm(&format!("./result/example_{}.ppm", sample));
}
```

# Reference

- [smallpt](http://www.kevinbeason.com/smallpt/)
- [go-tracing](https://github.com/xalanq/go-tracing)
