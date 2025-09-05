use crate::math::*;
use crate::materials::Material;
use crate::raytracer::HitInfo;

pub trait Primitive: Send + Sync {
    fn intersect(&self, ray: &Ray, time: f32) -> Option<HitInfo>;
    fn get_bounds(&self) -> (Vec3, Vec3);
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Self { center, radius, material }
    }
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
        
        // Spherical UV mapping
        let theta = (-normal.y).acos();
        let phi = (-normal.z).atan2(normal.x) + std::f32::consts::PI;
        let u = phi / (2.0 * std::f32::consts::PI);
        let v = theta / std::f32::consts::PI;
        
        Some(HitInfo {
            t,
            point,
            normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        (self.center - r, self.center + r)
    }
}

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub size: Option<(f32, f32)>, // Optional size limits
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material: Material) -> Self {
        Self {
            point,
            normal: normal.normalize(),
            material,
            size: None,
        }
    }
    
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some((width, height));
        self
    }
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
        
        // Check size limits if specified
        if let Some((width, height)) = self.size {
            let local_point = point - self.point;
            let u_axis = if self.normal.x.abs() > 0.9 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };
            let u_axis = u_axis.cross(self.normal).normalize();
            let v_axis = self.normal.cross(u_axis).normalize();
            
            let u_coord = local_point.dot(u_axis);
            let v_coord = local_point.dot(v_axis);
            
            if u_coord.abs() > width * 0.5 || v_coord.abs() > height * 0.5 {
                return None;
            }
        }
        
        // UV mapping for plane
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
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        // Return very large bounds for infinite plane
        if self.size.is_none() {
            (Vec3::new(-1000.0, -1000.0, -1000.0), Vec3::new(1000.0, 1000.0, 1000.0))
        } else {
            let (w, h) = self.size.unwrap();
            let half_w = Vec3::new(w * 0.5, 0.0, 0.0);
            let half_h = Vec3::new(0.0, h * 0.5, 0.0);
            (self.point - half_w - half_h, self.point + half_w + half_h)
        }
    }
}

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
    pub rotation: Vec3, // Euler angles
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Self {
            center,
            size,
            material,
            rotation: Vec3::zero(),
        }
    }
    
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }
}

impl Primitive for Cube {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        // Transform ray to cube's local space if rotated
        let (local_ray, inv_rotation) = if self.rotation != Vec3::zero() {
            let inv_rot = -self.rotation;
            let local_origin = (ray.origin - self.center).rotate_y(inv_rot.y).rotate_x(inv_rot.x).rotate_z(inv_rot.z);
            let local_dir = ray.direction.rotate_y(inv_rot.y).rotate_x(inv_rot.x).rotate_z(inv_rot.z);
            (Ray::new(local_origin + self.center, local_dir), Some(inv_rot))
        } else {
            (Ray::new(ray.origin, ray.direction), None)
        };
        
        let half_size = self.size / 2.0;
        let min = self.center - Vec3::new(half_size, half_size, half_size);
        let max = self.center + Vec3::new(half_size, half_size, half_size);
        
        // Ray-AABB intersection
        let t_min_x = (min.x - local_ray.origin.x) / local_ray.direction.x;
        let t_max_x = (max.x - local_ray.origin.x) / local_ray.direction.x;
        let (t_min_x, t_max_x) = if t_min_x > t_max_x { (t_max_x, t_min_x) } else { (t_min_x, t_max_x) };
        
        let t_min_y = (min.y - local_ray.origin.y) / local_ray.direction.y;
        let t_max_y = (max.y - local_ray.origin.y) / local_ray.direction.y;
        let (t_min_y, t_max_y) = if t_min_y > t_max_y { (t_max_y, t_min_y) } else { (t_min_y, t_max_y) };
        
        let t_min_z = (min.z - local_ray.origin.z) / local_ray.direction.z;
        let t_max_z = (max.z - local_ray.origin.z) / local_ray.direction.z;
        let (t_min_z, t_max_z) = if t_min_z > t_max_z { (t_max_z, t_min_z) } else { (t_min_z, t_max_z) };
        
        let t_min = t_min_x.max(t_min_y).max(t_min_z);
        let t_max = t_max_x.min(t_max_y).min(t_max_z);
        
        if t_max < 0.0 || t_min > t_max {
            return None;
        }
        
        let t = if t_min > 0.001 { t_min } else if t_max > 0.001 { t_max } else { return None; };
        
        let local_point = local_ray.point_at(t);
        
        // Calculate normal based on which face was hit
        let mut normal;
        let eps = 0.0001;
        
        if (local_point.x - min.x).abs() < eps {
            normal = Vec3::new(-1.0, 0.0, 0.0);
        } else if (local_point.x - max.x).abs() < eps {
            normal = Vec3::new(1.0, 0.0, 0.0);
        } else if (local_point.y - min.y).abs() < eps {
            normal = Vec3::new(0.0, -1.0, 0.0);
        } else if (local_point.y - max.y).abs() < eps {
            normal = Vec3::new(0.0, 1.0, 0.0);
        } else if (local_point.z - min.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, -1.0);
        } else {
            normal = Vec3::new(0.0, 0.0, 1.0);
        }
        
        // Transform normal back to world space if rotated
        if let Some(_) = inv_rotation {
            normal = normal.rotate_z(self.rotation.z).rotate_x(self.rotation.x).rotate_y(self.rotation.y);
        }
        
        // Transform point back to world space
        let world_point = if inv_rotation.is_some() {
            let local_offset = local_point - self.center;
            let world_offset = local_offset.rotate_z(self.rotation.z).rotate_x(self.rotation.x).rotate_y(self.rotation.y);
            self.center + world_offset
        } else {
            local_point
        };
        
        // UV mapping for cube faces
        let local_point_centered = local_point - self.center;
        let (u, v) = if normal.x.abs() > 0.5 {
            // X face
            ((local_point_centered.z + half_size) / self.size, (local_point_centered.y + half_size) / self.size)
        } else if normal.y.abs() > 0.5 {
            // Y face
            ((local_point_centered.x + half_size) / self.size, (local_point_centered.z + half_size) / self.size)
        } else {
            // Z face
            ((local_point_centered.x + half_size) / self.size, (local_point_centered.y + half_size) / self.size)
        };
        
        Some(HitInfo {
            t,
            point: world_point,
            normal,
            material: self.material.clone(),
            uv: (u, v),
        })
    }
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        let half_size = self.size / 2.0;
        let extent = Vec3::new(half_size, half_size, half_size);
        
        if self.rotation == Vec3::zero() {
            (self.center - extent, self.center + extent)
        } else {
            // For rotated cubes, compute conservative bounds
            let expanded = extent * 1.73; // sqrt(3) for worst case rotation
            (self.center - expanded, self.center + expanded)
        }
    }
}

pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub uv0: (f32, f32),
    pub uv1: (f32, f32),
    pub uv2: (f32, f32),
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: Material) -> Self {
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        Triangle {
            v0, v1, v2, normal, material,
            uv0: (0.0, 0.0),
            uv1: (1.0, 0.0),
            uv2: (0.5, 1.0),
        }
    }
    
    pub fn new_with_uvs(
        v0: Vec3, v1: Vec3, v2: Vec3,
        uv0: (f32, f32), uv1: (f32, f32), uv2: (f32, f32),
        material: Material
    ) -> Self {
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        Triangle {
            v0, v1, v2, normal, material,
            uv0, uv1, uv2,
        }
    }
    
    pub fn new_with_normal(v0: Vec3, v1: Vec3, v2: Vec3, normal: Vec3, material: Material) -> Self {
        Triangle {
            v0, v1, v2, 
            normal: normal.normalize(), 
            material,
            uv0: (0.0, 0.0),
            uv1: (1.0, 0.0),
            uv2: (0.5, 1.0),
        }
    }
}

impl Primitive for Triangle {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        // Möller-Trumbore intersection algorithm
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);
        
        if a > -0.00001 && a < 0.00001 {
            return None; // Ray is parallel to triangle
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
        
        // Interpolate UV coordinates
        let w = 1.0 - u - v;
        let interpolated_u = w * self.uv0.0 + u * self.uv1.0 + v * self.uv2.0;
        let interpolated_v = w * self.uv0.1 + u * self.uv1.1 + v * self.uv2.1;
        
        Some(HitInfo {
            t,
            point,
            normal: self.normal,
            material: self.material.clone(),
            uv: (interpolated_u, interpolated_v),
        })
    }
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        let min_x = self.v0.x.min(self.v1.x).min(self.v2.x);
        let min_y = self.v0.y.min(self.v1.y).min(self.v2.y);
        let min_z = self.v0.z.min(self.v1.z).min(self.v2.z);
        
        let max_x = self.v0.x.max(self.v1.x).max(self.v2.x);
        let max_y = self.v0.y.max(self.v1.y).max(self.v2.y);
        let max_z = self.v0.z.max(self.v1.z).max(self.v2.z);
        
        (Vec3::new(min_x, min_y, min_z), Vec3::new(max_x, max_y, max_z))
    }
}

// Cylinder primitive for more variety
pub struct Cylinder {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub material: Material,
}

impl Cylinder {
    pub fn new(center: Vec3, radius: f32, height: f32, material: Material) -> Self {
        Self { center, radius, height, material }
    }
}

impl Primitive for Cylinder {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        let half_height = self.height * 0.5;
        let oc = ray.origin - self.center;
        
        // Check intersection with infinite cylinder (ignoring Y)
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
        let b = 2.0 * (oc.x * ray.direction.x + oc.z * ray.direction.z);
        let c = oc.x * oc.x + oc.z * oc.z - self.radius * self.radius;
        
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        
        // Check which intersection is valid (within height bounds)
        for t in [t1, t2] {
            if t > 0.001 {
                let point = ray.point_at(t);
                let y_local = point.y - self.center.y;
                
                if y_local.abs() <= half_height {
                    // Hit the curved surface
                    let local_point = point - self.center;
                    let normal = Vec3::new(local_point.x, 0.0, local_point.z).normalize();
                    
                    // Cylindrical UV mapping
                    let theta = local_point.z.atan2(local_point.x);
                    let u = (theta + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                    let v = (y_local + half_height) / self.height;
                    
                    return Some(HitInfo {
                        t,
                        point,
                        normal,
                        material: self.material.clone(),
                        uv: (u, v),
                    });
                }
            }
        }
        
        // Check intersection with top and bottom caps
        let t_top = (self.center.y + half_height - ray.origin.y) / ray.direction.y;
        let t_bottom = (self.center.y - half_height - ray.origin.y) / ray.direction.y;
        
        for (t, normal) in [(t_top, Vec3::new(0.0, 1.0, 0.0)), (t_bottom, Vec3::new(0.0, -1.0, 0.0))] {
            if t > 0.001 {
                let point = ray.point_at(t);
                let local_point = point - self.center;
                let radius_sq = local_point.x * local_point.x + local_point.z * local_point.z;
                
                if radius_sq <= self.radius * self.radius {
                    // Hit a cap
                    let u = (local_point.x / self.radius + 1.0) * 0.5;
                    let v = (local_point.z / self.radius + 1.0) * 0.5;
                    
                    return Some(HitInfo {
                        t,
                        point,
                        normal,
                        material: self.material.clone(),
                        uv: (u, v),
                    });
                }
            }
        }
        
        None
    }
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        let half_height = self.height * 0.5;
        (
            self.center - Vec3::new(self.radius, half_height, self.radius),
            self.center + Vec3::new(self.radius, half_height, self.radius)
        )
    }
}

// Torus primitive for advanced geometry
pub struct Torus {
    pub center: Vec3,
    pub major_radius: f32,
    pub minor_radius: f32,
    pub material: Material,
}

impl Torus {
    pub fn new(center: Vec3, major_radius: f32, minor_radius: f32, material: Material) -> Self {
        Self { center, major_radius, minor_radius, material }
    }
}

impl Primitive for Torus {
    fn intersect(&self, ray: &Ray, _time: f32) -> Option<HitInfo> {
        // Transform ray to torus local space
        let oc = ray.origin - self.center;
        
        // Torus intersection is complex - using approximation for performance
        // This is a simplified version that works reasonably well
        let R = self.major_radius;
        let r = self.minor_radius;
        
        // Coefficients for quartic equation (simplified)
        let ox = oc.x;
        let oy = oc.y;
        let oz = oc.z;
        let dx = ray.direction.x;
        let dy = ray.direction.y;
        let dz = ray.direction.z;
        
        let _sum_d_sqr = dx*dx + dy*dy + dz*dz;
        let _e = ox*ox + oy*oy + oz*oz - R*R - r*r;
        let _f = ox*dx + oy*dy + oz*dz;
        let _four_a_sqr = 4.0 * R*R;
        
        // Simplified quartic solving - use iterative method
        for i in 0..100 {
            let t = 0.1 + i as f32 * 0.1;
            let point = ray.point_at(t);
            let local = point - self.center;
            
            let xy_dist = (local.x*local.x + local.z*local.z).sqrt();
            let torus_center_dist = (xy_dist - R).abs();
            let torus_surface_dist = (torus_center_dist*torus_center_dist + local.y*local.y).sqrt();
            
            if torus_surface_dist <= r && t > 0.001 {
                // Approximate normal
                let torus_center = Vec3::new(
                    local.x / xy_dist * R,
                    0.0,
                    local.z / xy_dist * R
                );
                let normal = (local - torus_center).normalize();
                
                // Torus UV mapping (simplified)
                let u = (local.z.atan2(local.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                let v = (local.y.atan2(xy_dist - R) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                
                return Some(HitInfo {
                    t,
                    point,
                    normal,
                    material: self.material.clone(),
                    uv: (u, v),
                });
            }
        }
        
        None
    }
    
    fn get_bounds(&self) -> (Vec3, Vec3) {
        let outer_radius = self.major_radius + self.minor_radius;
        (
            self.center - Vec3::new(outer_radius, self.minor_radius, outer_radius),
            self.center + Vec3::new(outer_radius, self.minor_radius, outer_radius)
        )
    }
}