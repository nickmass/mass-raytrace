use super::geom::{BoundingBox, BvhNode, Hit, Intersect};
use super::material::Background;
use crate::math::V3;

pub struct Camera {
    origin: V3,
    lower_left_corner: V3,
    horizontal: V3,
    vertical: V3,
    u: V3,
    v: V3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        vertical_fov: f64,
        look_from: V3,
        look_at: V3,
        view_up: V3,
        aspect_ratio: f64,
        aperture: f64,
        focus_distance: f64,
    ) -> Self {
        let vertical_fov_rads = vertical_fov * std::f64::consts::PI / 180.0;
        let half_height = (vertical_fov_rads / 2.0).tan();
        let viewport_height = half_height * 2.0;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (look_from - look_at).unit();
        let u = view_up.cross(&w).unit();
        let v = w.cross(&u);

        let origin = look_from;
        let horizontal = u * viewport_width * focus_distance;
        let vertical = v * viewport_height * focus_distance;
        let lower_left_corner =
            origin - (horizontal / 2.0) - (vertical / 2.0) - (w * focus_distance);

        let lens_radius = aperture / 2.0;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
        }
    }

    pub fn ray(&self, s: f64, t: f64) -> Ray {
        let blur = V3::random_in_unit_disk() * self.lens_radius;
        let offset = self.u * blur.x() + self.v * blur.y();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + (self.horizontal * s) + (self.vertical * t)
                - self.origin
                - offset,
        )
    }

    pub fn trace<I: Intersect + Background>(&self, scene: &I, ray: Ray, depth: i32) -> (V3, i32) {
        if depth <= 0 {
            (V3::zero(), depth)
        } else if let Some(hit) = scene.intersect(ray, 0.001, f64::INFINITY) {
            let emitted = hit.emit();
            if let Some(scatter) = hit.scatter(ray) {
                let result = self.trace(scene, scatter.scattered, depth - 1);
                ((result.0 * scatter.attenuation + emitted), result.1)
            } else {
                (emitted, depth)
            }
        } else {
            (scene.background(ray), depth)
        }
    }
}

pub struct World<B: Background> {
    background: B,
    objects: Vec<Box<dyn Intersect>>,
}

impl<B: Background> World<B> {
    pub fn new(background: B) -> Self {
        Self {
            background,
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add<O: 'static + Intersect>(&mut self, object: O) {
        let b = Box::new(object);
        self.objects.push(b);
    }

    pub fn build_bvh(&mut self) {
        let new_items = Vec::new();
        let objects = std::mem::replace(&mut self.objects, new_items);
        let nodes = BvhNode::new(objects);
        self.objects.push(Box::new(nodes));
    }
}

impl<B: Background> Background for World<B> {
    fn background(&self, ray: Ray) -> V3 {
        self.background.background(ray)
    }
}

impl<B: Background> Intersect for World<B> {
    fn intersect(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let mut found_hit = None;
        let mut closest_so_far = t_max;

        for obj in &self.objects {
            if let Some(hit) = obj.intersect(ray, t_min, closest_so_far) {
                closest_so_far = hit.t;
                found_hit = Some(hit);
            }
        }

        found_hit
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        if self.objects.len() == 0 {
            None
        } else {
            let mut group_box: Option<BoundingBox> = None;
            for obj in &self.objects {
                if let Some(bb) = obj.bounding_box() {
                    if let Some(group_box) = group_box.as_mut() {
                        *group_box = group_box.join(bb);
                    } else {
                        group_box = Some(bb);
                    }
                } else {
                    return None;
                }
            }

            group_box
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: V3,
    pub direction: V3,
}

impl Ray {
    pub fn new(origin: V3, direction: V3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f64) -> V3 {
        self.origin + (self.direction * t)
    }
}
