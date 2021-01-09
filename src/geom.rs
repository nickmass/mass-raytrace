use std::sync::Arc;

use super::material::{Material, Scatter};
use super::world::Ray;
use crate::math::{M4, V2, V3};

pub struct Hit<'a> {
    pub point: V3,
    pub normal: V3,
    pub uv: Option<V2>,
    pub t: f32,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> Hit<'a> {
    pub fn set_face_normal(&mut self, ray: Ray, outward_normal: V3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }

    pub fn scatter(&self, ray: Ray) -> Option<Scatter> {
        self.material.scatter(ray, &self)
    }

    pub fn emit(&self) -> V3 {
        self.material.emit(&self).unwrap_or(V3::zero())
    }
}

pub trait Intersect: Send + Sync {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>>;
    fn bounding_box(&self) -> Option<BoundingBox>;
}

pub struct Sphere<M: Material> {
    center: V3,
    radius: f32,
    material: M,
}

impl<M: Material> Sphere<M> {
    pub fn new(material: M, center: V3, radius: f32) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl<M: Material> Intersect for Sphere<M> {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>> {
        let offset_center = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = offset_center.dot(ray.direction);
        let c = offset_center.length_squared() - (self.radius * self.radius);
        let discriminant = (half_b * half_b) - (a * c);

        if discriminant < 0.0 {
            None
        } else {
            let sqrt_d = discriminant.sqrt();

            let mut root = (-half_b - sqrt_d) / a;
            if root < t_min || t_max < root {
                root = (-half_b + sqrt_d) / a;
                if root < t_min || t_max < root {
                    return None;
                }
            }

            let point = ray.at(root);
            let normal = (point - self.center) / self.radius;

            let mut hit = Hit {
                point,
                normal,
                t: root,
                uv: None,
                front_face: false,
                material: &self.material,
            };

            hit.set_face_normal(ray, normal);

            Some(hit)
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        Some(BoundingBox::new(
            self.center - V3::fill(self.radius.abs()),
            self.center + V3::fill(self.radius.abs()),
        ))
    }
}

pub struct BvhNode {
    left: Option<Box<dyn Intersect>>,
    right: Option<Box<dyn Intersect>>,
    bounding_box: BoundingBox,
}

impl BvhNode {
    pub fn new(mut items: Vec<Box<dyn Intersect>>) -> Self {
        let axis = fastrand::u8(0..3);

        let compare = match axis {
            0 => compare_x,
            1 => compare_y,
            2 => compare_z,
            _ => unreachable!(),
        };

        let (left, right) = if items.len() == 1 {
            (items.pop(), None)
        } else if items.len() == 2 {
            let a = items.pop().unwrap();
            let b = items.pop().unwrap();
            if compare(&a, &b) {
                (Some(a), Some(b))
            } else {
                (Some(b), Some(a))
            }
        } else {
            items.sort_by(|a, b| {
                if compare(a, b) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            let mid = items.len() / 2;
            let back_half = items.split_off(mid);
            (
                Some(Box::new(BvhNode::new(items)) as Box<dyn Intersect>),
                Some(Box::new(BvhNode::new(back_half)) as Box<dyn Intersect>),
            )
        };

        let bounding_box = match (
            left.as_ref().and_then(|l| l.bounding_box()),
            right.as_ref().and_then(|r| r.bounding_box()),
        ) {
            (Some(left), Some(right)) => left.join(right),
            (Some(left), None) => left,
            (None, Some(right)) => right,
            _ => unreachable!("Missing bounding box in bvh"),
        };

        Self {
            left,
            right,
            bounding_box,
        }
    }
}

fn compare_x(left: &Box<dyn Intersect>, right: &Box<dyn Intersect>) -> bool {
    match (left.bounding_box(), right.bounding_box()) {
        (Some(left), Some(right)) => left.minimum.x() < right.minimum.x(),
        _ => unreachable!("Missing bounding box in bvh"),
    }
}

fn compare_y(left: &Box<dyn Intersect>, right: &Box<dyn Intersect>) -> bool {
    match (left.bounding_box(), right.bounding_box()) {
        (Some(left), Some(right)) => left.minimum.y() < right.minimum.y(),
        _ => unreachable!("Missing bounding box in bvh"),
    }
}

fn compare_z(left: &Box<dyn Intersect>, right: &Box<dyn Intersect>) -> bool {
    match (left.bounding_box(), right.bounding_box()) {
        (Some(left), Some(right)) => left.minimum.z() < right.minimum.z(),
        _ => unreachable!("Missing bounding box in bvh"),
    }
}

impl Intersect for BvhNode {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>> {
        if self.bounding_box.hit(ray, t_min, t_max) {
            let left_hit = self
                .left
                .as_ref()
                .and_then(|left| left.intersect(ray, t_min, t_max));
            let t_max = left_hit.as_ref().map(|l| l.t).unwrap_or(t_max);
            self.right
                .as_ref()
                .and_then(|r| r.intersect(ray, t_min, t_max))
                .or(left_hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        Some(self.bounding_box)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    minimum: V3,
    maximum: V3,
}

impl BoundingBox {
    pub fn new(minimum: V3, maximum: V3) -> Self {
        Self { minimum, maximum }
    }

    pub fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> bool {
        let v_min = (self.minimum - ray.origin) / ray.direction;
        let v_max = (self.maximum - ray.origin) / ray.direction;

        let min = v_min.min(v_max);
        let max = v_min.max(v_max);

        let t_min = min.x().max(t_min);
        let t_max = max.x().min(t_max);

        if t_max < t_min {
            return false;
        }

        let t_min = min.y().max(t_min);
        let t_max = max.y().min(t_max);

        if t_max < t_min {
            return false;
        }

        let t_min = min.z().max(t_min);
        let t_max = max.z().min(t_max);

        if t_max < t_min {
            return false;
        }

        true
    }

    pub fn join(&self, other: BoundingBox) -> Self {
        let minimum = self.minimum.min(other.minimum);
        let maximum = self.maximum.max(other.maximum);

        Self::new(minimum, maximum)
    }

    pub fn corners(&self) -> impl Iterator<Item = V3> {
        let mut corner = 0;
        let min = self.minimum;
        let max = self.maximum;
        std::iter::from_fn(move || {
            if corner == 8 {
                None
            } else {
                let x = if corner & 1 == 0 { max.x() } else { min.x() };
                let y = if corner & 2 == 0 { max.y() } else { min.y() };
                let z = if corner & 4 == 0 { max.z() } else { min.z() };
                corner += 1;

                Some(V3::new(x, y, z))
            }
        })
    }
}

pub struct Model<M: Material> {
    material: M,
    triangles: Arc<BvhNode>,
}

impl<M: 'static + Clone + Material> Model<M> {
    pub fn new<T: IntoIterator<Item = Triangle<TM>>, TM: 'static + Material>(
        material: M,
        triangles: T,
    ) -> Self {
        let triangles = triangles
            .into_iter()
            .map(|t| Box::new(t) as Box<dyn Intersect>)
            .collect();
        let triangles = Arc::new(BvhNode::new(triangles));

        Self {
            triangles,
            material,
        }
    }

    pub fn instance<IM: Material>(
        &self,
        material: IM,
        translation: V3,
        rotation: V3,
        scale: V3,
    ) -> Instance<IM> {
        Instance::new(
            self.triangles.clone(),
            material,
            translation,
            rotation,
            scale,
        )
    }
}

impl<M: Material> Intersect for Model<M> {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>> {
        let hit = self.triangles.intersect(ray, t_min, t_max);
        if let Some(mut hit) = hit {
            hit.material = &self.material;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        self.triangles.bounding_box()
    }
}

pub struct Instance<M: Material> {
    triangles: Arc<BvhNode>,
    material: M,
    transform: M4,
    rotation: M4,
    ray_transform: M4,
    ray_rotation: M4,
    bounding_box: BoundingBox,
}

impl<M: Material> Instance<M> {
    pub fn new(
        triangles: Arc<BvhNode>,
        material: M,
        translation: V3,
        rotation: V3,
        scale: V3,
    ) -> Self {
        let ray_translation = translation * -1.0;
        let ray_rotation = rotation * -1.0;
        let ray_scale = V3::fill(1.0) / scale;

        let translation = M4::translation(translation);
        let ray_translation = M4::translation(ray_translation);

        let rotation = M4::rotation(rotation);
        let ray_rotation = M4::rotation_rev(ray_rotation);

        let scale = M4::scale(scale);
        let ray_scale = M4::scale(ray_scale);

        let transform = translation * scale * rotation;
        let ray_transform = ray_rotation * ray_scale * ray_translation;

        let mut minimum = V3::fill(f32::INFINITY);
        let mut maximum = V3::fill(f32::NEG_INFINITY);

        for corner in triangles.bounding_box.corners().map(|c| transform * c) {
            minimum = minimum.min(corner);
            maximum = maximum.max(corner);
        }

        let bounding_box = BoundingBox::new(minimum, maximum);

        Self {
            triangles,
            material,
            transform,
            rotation,
            ray_transform,
            ray_rotation,
            bounding_box,
        }
    }
}

impl<M: Material> Intersect for Instance<M> {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>> {
        let ray = Ray::new(
            self.ray_transform * ray.origin,
            self.ray_rotation * ray.direction,
        );
        let hit = self.triangles.intersect(ray, t_min, t_max);
        if let Some(mut hit) = hit {
            hit.point = self.transform * hit.point;
            hit.normal = self.rotation * hit.normal;
            hit.material = &self.material;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        Some(self.bounding_box)
    }
}

struct UV {
    uv_a: V2,
    uv_b: V2,
    uv_c: V2,
}

pub struct Triangle<M: Material> {
    vertex_a: V3,
    vertex_b: V3,
    vertex_c: V3,
    uvs: Option<UV>,
    material: M,
    normal_a: V3,
    normal_b: V3,
    normal_c: V3,
    tangent: V3,
    bitangent: V3,
}

impl<M: Material> Triangle<M> {
    pub fn new(material: M, vertex_a: V3, vertex_b: V3, vertex_c: V3) -> Self {
        let ab = vertex_b - vertex_a;
        let ac = vertex_c - vertex_a;
        let normal = ab.cross(ac).unit();

        Self {
            material,
            vertex_a,
            vertex_b,
            vertex_c,
            uvs: None,
            normal_a: normal,
            normal_b: normal,
            normal_c: normal,
            tangent: V3::zero(),
            bitangent: V3::zero(),
        }
    }

    pub fn with_norms_and_uvs(
        material: M,
        (vertex_a, normal_a, uv_a): (V3, V3, V2),
        (vertex_b, normal_b, uv_b): (V3, V3, V2),
        (vertex_c, normal_c, uv_c): (V3, V3, V2),
    ) -> Self {
        let ab = vertex_b - vertex_a;
        let ac = vertex_c - vertex_a;
        let uv_ab = uv_b - uv_a;
        let uv_ac = uv_c - uv_a;
        let r = (1.0 / (uv_ab.x() * uv_ac.y() - uv_ab.y() * uv_ac.x()))
            .min(1.0)
            .max(-1.0);
        let tangent = (ab * uv_ac.y() - ac * uv_ab.y()) * r;
        let bitangent = (ac * uv_ab.x() - ab * uv_ac.x()) * r;

        Self {
            material,
            vertex_a,
            vertex_b,
            vertex_c,
            uvs: Some(UV { uv_a, uv_b, uv_c }),
            normal_a,
            normal_b,
            normal_c,
            tangent,
            bitangent,
        }
    }
}

impl<M: Material> Intersect for Triangle<M> {
    fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'_>> {
        let ab = self.vertex_b - self.vertex_a;
        let ac = self.vertex_c - self.vertex_a;

        let p_vec = ray.direction.cross(ac);
        let det = ab.dot(p_vec);

        if det.abs() < f32::EPSILON * 2.0 {
            return None;
        }

        let inv_det = 1.0 / det;

        let t_vec = ray.origin - self.vertex_a;
        let u = t_vec.dot(p_vec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q_vec = t_vec.cross(ab);
        let v = ray.direction.dot(q_vec) * inv_det;
        if v < 0.0 || v + u > 1.0 {
            return None;
        }

        let t = ac.dot(q_vec) * inv_det;

        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);

        let d0 = self.vertex_a - point;
        let d1 = self.vertex_b - point;
        let d2 = self.vertex_c - point;

        let area = (self.vertex_a - self.vertex_b)
            .cross(self.vertex_a - self.vertex_c)
            .length();

        let a0 = d1.cross(d2).length() / area;
        let a1 = d2.cross(d0).length() / area;
        let a2 = d0.cross(d1).length() / area;

        let normal = self.normal_a * a0 + self.normal_b * a1 + self.normal_c * a2;

        let (normal, uv) = if let Some(uvs) = &self.uvs {
            let uv = uvs.uv_a * a0 + uvs.uv_b * a1 + uvs.uv_c * a2;

            let normal = if let Some(tan_normal) = self.material.normal(uv) {
                (self.tangent * tan_normal.x()
                    + self.bitangent * tan_normal.y()
                    + normal * tan_normal.z())
                .unit()
            } else {
                normal
            };

            (normal, Some(uv))
        } else {
            (normal, None)
        };

        let mut hit = Hit {
            point,
            normal,
            t,
            uv,
            front_face: false,
            material: &self.material,
        };

        hit.set_face_normal(ray, normal);

        Some(hit)
    }

    fn bounding_box(&self) -> Option<BoundingBox> {
        let min = self.vertex_a.min(self.vertex_b).min(self.vertex_c);
        let max = self.vertex_a.max(self.vertex_b).max(self.vertex_c);

        Some(BoundingBox::new(min, max))
    }
}
