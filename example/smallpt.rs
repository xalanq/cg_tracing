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
        .add(Sphere::new(Vct::new(1e5 + 1., 40.8, 81.6), 1e5, Geo::new(z, c1, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(-1e5 + 99., 40.8, 81.6), 1e5, Geo::new(z, c2, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(50., 40.8, 1e5), 1e5, Geo::new(z, c3, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(50., 40.8, -1e5 + 170.0), 1e5, Geo::new(z, z, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(50., 1e5, 81.6), 1e5, Geo::new(z, c3, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(50., -1e5 + 81.6, 81.6), 1e5, Geo::new(z, c3, Texture::Diffuse)))
        .add(Sphere::new(Vct::new(27., 16.5, 47.), 16.5, Geo::new(z, c4, Texture::Specular)))
        .add(Sphere::new(Vct::new(73., 16.5, 78.), 16.5, Geo::new(z, c4, Texture::Refractive)))
        .add(Sphere::new(
            Vct::new(50., 681.6 - 0.27, 81.6),
            600.,
            Geo::new(
                Vct::new(12., 12., 12.),
                z,
                Texture::Diffuse,
            ),
        ))
        .render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", sample));
}
