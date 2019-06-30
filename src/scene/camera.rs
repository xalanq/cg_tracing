use crate::{linalg::Vct, Deserialize, Flt, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub origin: Vct,
    pub direct: Vct,
    pub view_angle_scale: Flt,
    pub plane_distance: Flt,
    pub focal_distance: Flt,
    pub aperture: Flt,
}

impl Camera {
    pub fn new(
        origin: Vct,
        direct: Vct,
        view_angle_scale: Flt,
        plane_distance: Flt,
        focal_distance: Flt,
        aperture: Flt,
    ) -> Self {
        Self { origin, direct, view_angle_scale, plane_distance, focal_distance, aperture }
    }
}
