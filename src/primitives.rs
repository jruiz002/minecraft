use crate::math::*;
use crate::materials::Material;
use crate::raytracer::HitInfo;

pub trait Primitive: Send + Sync {
    fn intersect(&self, ray: &Ray, time: f32) -> Option<HitInfo>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Primitive for Sphere {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        
        let t = if t1 > 0.001 { t1 } else if t2 > 0.001 { t2 } else { return None; };
        
        let point = ray.point_at(t);
        let normal = (point - self.center).normalize();
        
        // UV mapping for sphere
        let phi = normal.z.atan2(normal.x);
        let theta = normal.y.asin();
        let u = 1.0 - (phi + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
        let v = (theta + std::f32::consts::PI / 2.0) / std::f32::consts::PI;
        
        Some(HitInfo {
            t,
            point,
            normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
}

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Primitive for Plane {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        let denom = self.normal.dot(ray.direction);
        
        if denom.abs() < 0.0001 {
            return None;
        }
        
        let t = (self.point - ray.origin).dot(self.normal) / denom;
        
        if t <= 0.001 {
            return None;
        }
        
        let point = ray.point_at(t);
        
        // UV mapping for plane (simple tiling)
        let u = (point.x % 1.0 + 1.0) % 1.0;
        let v = (point.z % 1.0 + 1.0) % 1.0;
        
        Some(HitInfo {
            t,
            point,
            normal: self.normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
}

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
}

impl Primitive for Cube {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        let half_size = self.size / 2.0;
        let min = self.center - Vec3::new(half_size, half_size, half_size);
        let max = self.center + Vec3::new(half_size, half_size, half_size);
        
        let t_min_x = (min.x - ray.origin.x) / ray.direction.x;
        let t_max_x = (max.x - ray.origin.x) / ray.direction.x;
        let (t_min_x, t_max_x) = if t_min_x > t_max_x { (t_max_x, t_min_x) } else { (t_min_x, t_max_x) };
        
        let t_min_y = (min.y - ray.origin.y) / ray.direction.y;
        let t_max_y = (max.y - ray.origin.y) / ray.direction.y;
        let (t_min_y, t_max_y) = if t_min_y > t_max_y { (t_max_y, t_min_y) } else { (t_min_y, t_max_y) };
        
        let t_min_z = (min.z - ray.origin.z) / ray.direction.z;
        let t_max_z = (max.z - ray.origin.z) / ray.direction.z;
        let (t_min_z, t_max_z) = if t_min_z > t_max_z { (t_max_z, t_min_z) } else { (t_min_z, t_max_z) };
        
        let t_min = t_min_x.max(t_min_y).max(t_min_z);
        let t_max = t_max_x.min(t_max_y).min(t_max_z);
        
        if t_max < 0.0 || t_min > t_max {
            return None;
        }
        
        let t = if t_min > 0.001 { t_min } else if t_max > 0.001 { t_max } else { return None; };
        
        let point = ray.point_at(t);
        
        // Calculate normal based on which face was hit
        let normal;
        let eps = 0.0001;
        
        if (point.x - min.x).abs() < eps {
            normal = Vec3::new(-1.0, 0.0, 0.0);
        } else if (point.x - max.x).abs() < eps {
            normal = Vec3::new(1.0, 0.0, 0.0);
        } else if (point.y - min.y).abs() < eps {
            normal = Vec3::new(0.0, -1.0, 0.0);
        } else if (point.y - max.y).abs() < eps {
            normal = Vec3::new(0.0, 1.0, 0.0);
        } else if (point.z - min.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, -1.0);
        } else {
            normal = Vec3::new(0.0, 0.0, 1.0);
        }
        
        // UV mapping for cube
        let local_point = point - self.center;
        let (u, v) = if normal.x.abs() > 0.5 {
            // X face
            ((local_point.z + half_size) / self.size, (local_point.y + half_size) / self.size)
        } else if normal.y.abs() > 0.5 {
            // Y face
            ((local_point.x + half_size) / self.size, (local_point.z + half_size) / self.size)
        } else {
            // Z face
            ((local_point.x + half_size) / self.size, (local_point.y + half_size) / self.size)
        };
        
        Some(HitInfo {
            t,
            point,
            normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
}

pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: Material) -> Self {
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        Triangle {
            v0, v1, v2, normal, material
        }
    }
}

impl Primitive for Triangle {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);
        
        if a > -0.00001 && a < 0.00001 {
            return None;
        }
        
        let f = 1.0 / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * edge2.dot(q);
        
        if t <= 0.001 {
            return None;
        }
        
        let point = ray.point_at(t);
        
        Some(HitInfo {
            t,
            point,
            normal: self.normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
}