use crate::math::*;
use crate::materials::*;
use crate::primitives::*;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, aspect: f32) -> Self {
        Camera {
            position,
            target,
            up: up.normalize(),
            fov,
            aspect,
        }
    }
    
    pub fn get_forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }
    
    pub fn get_right(&self) -> Vec3 {
        self.get_forward().cross(self.up).normalize()
    }
    
    pub fn get_up(&self) -> Vec3 {
        self.get_right().cross(self.get_forward()).normalize()
    }
    
    pub fn get_ray(&self, x: f32, y: f32, width: usize, height: usize) -> Ray {
        let aspect = width as f32 / height as f32;
        let fov_rad = self.fov.to_radians();
        let half_height = (fov_rad / 2.0).tan();
        let half_width = aspect * half_height;
        
        let w = (self.position - self.target).normalize();
        let u = self.up.cross(w).normalize();
        let v = w.cross(u);
        
        let lower_left_corner = self.position 
            - u * half_width
            - v * half_height
            - w;
        
        let horizontal = u * 2.0 * half_width;
        let vertical = v * 2.0 * half_height;
        
        let s = x / width as f32;
        let t = (height as f32 - y) / height as f32;
        
        let direction = (lower_left_corner + horizontal * s + vertical * t - self.position).normalize();
        
        Ray::new(self.position, direction)
    }
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

pub struct Scene {
    pub objects: Vec<Box<dyn Primitive>>,
    pub lights: Vec<Light>,
    pub skybox: Option<Skybox>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            skybox: None,
        }
    }
}

pub struct Skybox {
    pub top_color: Vec3,
    pub horizon_color: Vec3,
}

impl Skybox {
    pub fn gradient(top_color: Vec3, horizon_color: Vec3) -> Self {
        Skybox {
            top_color,
            horizon_color,
        }
    }
    
    pub fn color_at(&self, direction: Vec3) -> Vec3 {
        let t = (direction.normalize().y + 1.0) * 0.5;
        self.horizon_color.lerp(self.top_color, t)
    }
}

pub struct HitInfo {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub uv: (f32, f32),
}

const MAX_DEPTH: i32 = 8;

pub fn trace_ray(ray: &Ray, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    if depth >= MAX_DEPTH {
        return Vec3::zero();
    }
    
    // Apply rotation to ray
    let rotated_origin = ray.origin.rotate_y(-rotation_y);
    let rotated_direction = ray.direction.rotate_y(-rotation_y);
    let rotated_ray = Ray::new(rotated_origin, rotated_direction);
    
    if let Some(hit) = intersect_scene(&rotated_ray, scene, time) {
        shade_hit(ray, &hit, scene, depth, time, rotation_y)
    } else {
        // Skybox
        if let Some(skybox) = &scene.skybox {
            skybox.color_at(ray.direction)
        } else {
            Vec3::new(0.2, 0.3, 0.5) // Default sky color
        }
    }
}

fn intersect_scene(ray: &Ray, scene: &Scene, time: f32) -> Option<HitInfo> {
    let mut closest_hit: Option<HitInfo> = None;
    let mut closest_t = f32::INFINITY;
    
    for object in &scene.objects {
        if let Some(hit) = object.intersect(ray, time) {
            if hit.t > 0.001 && hit.t < closest_t {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }
    }
    
    closest_hit
}

fn shade_hit(ray: &Ray, hit: &HitInfo, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    let mut color = Vec3::zero();
    
    // Emissive contribution
    color = color + hit.material.emissive;
    
    // Direct lighting
    for light in &scene.lights {
        let light_dir = (light.position - hit.point).normalize();
        let light_distance = (light.position - hit.point).length();
        
        // Shadow ray
        let shadow_ray = Ray::new(hit.point + hit.normal * 0.001, light_dir);
        let in_shadow = intersect_scene(&shadow_ray, scene, time)
            .map(|shadow_hit| shadow_hit.t < light_distance)
            .unwrap_or(false);
        
        if !in_shadow {
            // Diffuse lighting
            let n_dot_l = hit.normal.dot(light_dir).max(0.0);
            let attenuation = 1.0 / (1.0 + 0.1 * light_distance + 0.01 * light_distance * light_distance);
            
            let mut material_color = hit.material.albedo;
            if let Some(texture) = &hit.material.texture {
                material_color = material_color * texture.sample(hit.uv.0, hit.uv.1, time);
            }
            
            color = color + material_color * light.color * light.intensity * n_dot_l * attenuation;
            
            // Specular lighting
            let view_dir = (-ray.direction).normalize();
            let reflect_dir = (-light_dir).reflect(hit.normal);
            let spec = view_dir.dot(reflect_dir).max(0.0).powf(32.0);
            color = color + light.color * hit.material.specular * spec * attenuation;
        }
    }
    
    // Reflection
    if hit.material.reflectivity > 0.0 && depth < MAX_DEPTH {
        let reflect_dir = ray.direction.reflect(hit.normal);
        let reflect_ray = Ray::new(hit.point + hit.normal * 0.001, reflect_dir);
        let reflect_color = trace_ray(&reflect_ray, scene, depth + 1, time, rotation_y);
        color = color + reflect_color * hit.material.reflectivity;
    }
    
    // Refraction/Transparency
    if hit.material.transparency > 0.0 && depth < MAX_DEPTH {
        let eta = if ray.direction.dot(hit.normal) < 0.0 {
            1.0 / hit.material.refraction_index
        } else {
            hit.material.refraction_index
        };
        
        let normal = if ray.direction.dot(hit.normal) < 0.0 {
            hit.normal
        } else {
            -hit.normal
        };
        
        if let Some(refract_dir) = ray.direction.refract(normal, eta) {
            let refract_ray = Ray::new(hit.point - normal * 0.001, refract_dir);
            let refract_color = trace_ray(&refract_ray, scene, depth + 1, time, rotation_y);
            color = color.lerp(refract_color, hit.material.transparency);
        }
    }
    
    color
}