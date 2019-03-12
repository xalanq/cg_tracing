extern crate cg_tracing;

use cg_tracing::prelude::*;

fn main() {
    let mut p = Pic::new(1024, 768);
    let sample = 200;
    let z = vct!(0, 0, 0);
    let l = vct!(12, 12, 12);
    let (c1, c2) = (vct!(0.75, 0.25, 0.25), vct!(0.25, 0.25, 0.75));
    let (c3, c4) = (vct!(0.75, 0.75, 0.75), vct!(1, 1, 1) * 0.999);
    let camera = cam!(vct!(50, 52, 295.6), vct!(0, -0.042612, -1), 0.5135);
    let max_depth = 10;
    let thread_num = 0; // if set 0. thread number is the number of CPUs available(logical cores).
    let stack_size = 256 * 1024 * 1024;
    let (na, ng) = (1.0, 1.5);
    World::new(camera, sample, max_depth, thread_num, stack_size, na, ng)
        .add(sphere!(vct!(1e5 + 1.0, 40.8, 81.6), 1e5, sphere_raw!(z, c1, Material::Diffuse)))
        .add(sphere!(vct!(-1e5 + 99.0, 40.8, 81.6), 1e5, sphere_raw!(z, c2, Material::Diffuse)))
        .add(sphere!(vct!(50, 40.8, 1e5), 1e5, sphere_raw!(z, c3, Material::Diffuse)))
        .add(sphere!(vct!(50, 40.8, -1e5 + 170.0), 1e5, sphere_raw!(z, z, Material::Diffuse)))
        .add(sphere!(vct!(50, 1e5, 81.6), 1e5, sphere_raw!(z, c3, Material::Diffuse)))
        .add(sphere!(vct!(50, -1e5 + 81.6, 81.6), 1e5, sphere_raw!(z, c3, Material::Diffuse)))
        .add(sphere!(vct!(27, 16.5, 47), 16.5, sphere_raw!(z, c4, Material::Specular)))
        .add(sphere!(vct!(73, 16.5, 78), 16.5, sphere_raw!(z, c4, Material::Refractive)))
        .add(sphere!(vct!(50, 681.33, 81.6), 600, sphere_raw!(l, z, Material::Diffuse)))
        .render(&mut p);
    p.save_ppm(&format!("example_{}.ppm", sample));
}
