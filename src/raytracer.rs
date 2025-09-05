use crate::math::*;
use crate::materials::*;
use crate::primitives::*;
use rand::Rng;

// Rendering feature toggles
const ENABLE_AA: bool = false; // Disable per-pixel jitter for higher FPS

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub focus_distance: f32,
    pub aperture: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, aspect: f32) -> Self {
        Camera {
            position,
            target,
            up: up.normalize(),
            fov,
            aspect,
            focus_distance: (target - position).length(),
            aperture: 0.0,
        }
    }
    
    pub fn with_depth_of_field(mut self, focus_distance: f32, aperture: f32) -> Self {
        self.focus_distance = focus_distance;
        self.aperture = aperture;
        self
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
            - u * half_width * self.focus_distance
            - v * half_height * self.focus_distance
            - w * self.focus_distance;
        
        let horizontal = u * 2.0 * half_width * self.focus_distance;
        let vertical = v * 2.0 * half_height * self.focus_distance;
        
        let s = x / width as f32;
        let t = (height as f32 - y) / height as f32;
        
        // Optional jitter for anti-aliasing
        let (s_jittered, t_jittered) = if ENABLE_AA {
            let mut rng = rand::thread_rng();
            let jitter_x = rng.gen::<f32>() - 0.5;
            let jitter_y = rng.gen::<f32>() - 0.5;
            (s + jitter_x / width as f32, t + jitter_y / height as f32)
        } else {
            (s, t)
        };
        
        let mut ray_origin = self.position;
        
        // Depth of field effect
        if self.aperture > 0.0 {
            let rd = random_in_unit_disk() * self.aperture;
            let offset = u * rd.x + v * rd.y;
            ray_origin = ray_origin + offset;
        }
        
        let direction = (lower_left_corner + horizontal * s_jittered + vertical * t_jittered - ray_origin).normalize();
        
        Ray::new(ray_origin, direction)
    }
}

// Precomputed per-frame camera parameters to avoid per-pixel recomputation
pub struct CameraFrame {
    pub origin: Vec3,
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub focus_distance: f32,
    pub aperture: f32,
    pub width: usize,
    pub height: usize,
}

impl Camera {
    pub fn build_frame(&self, width: usize, height: usize) -> CameraFrame {
        let aspect = width as f32 / height as f32;
        let fov_rad = self.fov.to_radians();
        let half_height = (fov_rad / 2.0).tan();
        let half_width = aspect * half_height;
        
        let w = (self.position - self.target).normalize();
        let u = self.up.cross(w).normalize();
        let v = w.cross(u);
        
        let lower_left_corner = self.position
            - u * half_width * self.focus_distance
            - v * half_height * self.focus_distance
            - w * self.focus_distance;
        
        let horizontal = u * 2.0 * half_width * self.focus_distance;
        let vertical = v * 2.0 * half_height * self.focus_distance;
        
        CameraFrame {
            origin: self.position,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            focus_distance: self.focus_distance,
            aperture: self.aperture,
            width,
            height,
        }
    }
}

impl CameraFrame {
    pub fn get_ray(&self, x: f32, y: f32) -> Ray {
        let s = x / self.width as f32;
        let t = (self.height as f32 - y) / self.height as f32;
        
        let (s_jittered, t_jittered) = if ENABLE_AA {
            let mut rng = rand::thread_rng();
            let jitter_x = rng.gen::<f32>() - 0.5;
            let jitter_y = rng.gen::<f32>() - 0.5;
            (s + jitter_x / self.width as f32, t + jitter_y / self.height as f32)
        } else {
            (s, t)
        };
        
        let mut ray_origin = self.origin;
        if self.aperture > 0.0 {
            let rd = random_in_unit_disk() * self.aperture;
            let offset = self.u * rd.x + self.v * rd.y;
            ray_origin = ray_origin + offset;
        }
        
        let direction = (self.lower_left_corner + self.horizontal * s_jittered + self.vertical * t_jittered - ray_origin).normalize();
        Ray::new(ray_origin, direction)
    }
}

fn random_in_unit_disk() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), 0.0) * 2.0 - Vec3::new(1.0, 1.0, 0.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub light_type: LightType,
}

#[derive(Clone)]
pub enum LightType {
    Point,
    Directional(Vec3), // Direction vector
    Spot { direction: Vec3, inner_cone: f32, outer_cone: f32 },
}

impl Light {
    pub fn point(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            light_type: LightType::Point,
        }
    }
    
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position: Vec3::zero(), // Not used for directional
            color,
            intensity,
            light_type: LightType::Directional(direction.normalize()),
        }
    }
    
    pub fn spot(position: Vec3, direction: Vec3, color: Vec3, intensity: f32, inner_cone: f32, outer_cone: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            light_type: LightType::Spot {
                direction: direction.normalize(),
                inner_cone,
                outer_cone,
            },
        }
    }
    
    pub fn get_light_direction(&self, point: Vec3) -> Vec3 {
        match &self.light_type {
            LightType::Point => (self.position - point).normalize(),
            LightType::Directional(dir) => -*dir,
            LightType::Spot { .. } => (self.position - point).normalize(),
        }
    }
    
    pub fn get_attenuation(&self, point: Vec3) -> f32 {
        match &self.light_type {
            LightType::Point | LightType::Spot { .. } => {
                let distance = (self.position - point).length();
                1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance)
            },
            LightType::Directional(_) => 1.0,
        }
    }
    
    pub fn get_spot_factor(&self, point: Vec3) -> f32 {
        match &self.light_type {
            LightType::Spot { direction, inner_cone, outer_cone } => {
                let light_dir = (point - self.position).normalize();
                let cos_angle = direction.dot(light_dir);
                let cos_inner = inner_cone.cos();
                let cos_outer = outer_cone.cos();
                
                if cos_angle > cos_inner {
                    1.0
                } else if cos_angle > cos_outer {
                    (cos_angle - cos_outer) / (cos_inner - cos_outer)
                } else {
                    0.0
                }
            },
            _ => 1.0,
        }
    }
}

pub struct Scene {
    pub objects: Vec<Box<dyn Primitive>>,
    pub lights: Vec<Light>,
    pub skybox: Option<Skybox>,
    pub ambient_light: Vec3,
    pub fog: Option<Fog>,
    pub bvh: Option<BVHNode>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            skybox: None,
            ambient_light: Vec3::new(0.1, 0.1, 0.15),
            fog: None,
            bvh: None,
        }
    }
    
    pub fn with_ambient_light(mut self, ambient: Vec3) -> Self {
        self.ambient_light = ambient;
        self
    }
    
    pub fn with_fog(mut self, fog: Fog) -> Self {
        self.fog = Some(fog);
        self
    }
}

// ========================= BVH Acceleration =========================
#[derive(Clone)]
pub struct BVHNode {
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub left: Option<Box<BVHNode>>,
    pub right: Option<Box<BVHNode>>,
    pub indices: Vec<usize>, // leaf indices into Scene.objects
}

fn union_bounds(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> (Vec3, Vec3) {
    (a_min.min(b_min), a_max.max(b_max))
}

fn ray_aabb_intersect(ray: &Ray, bmin: Vec3, bmax: Vec3) -> bool {
    let inv_dir = Vec3::new(
        if ray.direction.x.abs() < 1e-8 { 1e32 } else { 1.0 / ray.direction.x },
        if ray.direction.y.abs() < 1e-8 { 1e32 } else { 1.0 / ray.direction.y },
        if ray.direction.z.abs() < 1e-8 { 1e32 } else { 1.0 / ray.direction.z },
    );
    let mut tmin = (bmin.x - ray.origin.x) * inv_dir.x;
    let mut tmax = (bmax.x - ray.origin.x) * inv_dir.x;
    if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }
    let mut tymin = (bmin.y - ray.origin.y) * inv_dir.y;
    let mut tymax = (bmax.y - ray.origin.y) * inv_dir.y;
    if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }
    if (tmin > tymax) || (tymin > tmax) { return false; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }
    let mut tzmin = (bmin.z - ray.origin.z) * inv_dir.z;
    let mut tzmax = (bmax.z - ray.origin.z) * inv_dir.z;
    if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }
    if (tmin > tzmax) || (tzmin > tmax) { return false; }
    true
}

struct ObjectInfo {
    index: usize,
    bmin: Vec3,
    bmax: Vec3,
    centroid: Vec3,
}

pub fn build_scene_bvh(scene: &mut Scene) {
    if scene.objects.is_empty() { return; }
    // Gather bounds
    let mut infos: Vec<ObjectInfo> = Vec::with_capacity(scene.objects.len());
    for (i, obj) in scene.objects.iter().enumerate() {
        let (bmin, bmax) = obj.get_bounds();
        let centroid = (bmin + bmax) * 0.5;
        infos.push(ObjectInfo { index: i, bmin, bmax, centroid });
    }
    let node = build_bvh_recursive(&mut infos[..]);
    scene.bvh = Some(node);
}

fn build_bvh_recursive(objects: &mut [ObjectInfo]) -> BVHNode {
    // Compute bounds of all
    let mut bounds_min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut bounds_max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for info in objects.iter() {
        let (bmin, bmax) = union_bounds(bounds_min, bounds_max, info.bmin, info.bmax);
        bounds_min = bmin; bounds_max = bmax;
    }
    if objects.len() <= 8 {
        return BVHNode {
            bounds_min,
            bounds_max,
            left: None,
            right: None,
            indices: objects.iter().map(|o| o.index).collect(),
        };
    }
    // Choose split axis by largest extent
    let extent = bounds_max - bounds_min;
    let axis = if extent.x > extent.y && extent.x > extent.z { 0 } else if extent.y > extent.z { 1 } else { 2 };
    objects.sort_by(|a, b| {
        let ca = match axis { 0 => a.centroid.x, 1 => a.centroid.y, _ => a.centroid.z };
        let cb = match axis { 0 => b.centroid.x, 1 => b.centroid.y, _ => b.centroid.z };
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = objects.len() / 2;
    let (left_slice, right_slice) = objects.split_at_mut(mid);
    let left = build_bvh_recursive(left_slice);
    let right = build_bvh_recursive(right_slice);
    BVHNode {
        bounds_min,
        bounds_max,
        left: Some(Box::new(left)),
        right: Some(Box::new(right)),
        indices: Vec::new(),
    }
}

fn intersect_bvh(ray: &Ray, node: &BVHNode, scene: &Scene, time: f32) -> Option<HitInfo> {
    if !ray_aabb_intersect(ray, node.bounds_min, node.bounds_max) {
        return None;
    }
    let mut best_hit: Option<HitInfo> = None;
    let mut best_t = f32::INFINITY;
    if node.left.is_none() && node.right.is_none() {
        for &idx in &node.indices {
            if let Some(hit) = scene.objects[idx].intersect(ray, time) {
                if hit.t > 0.001 && hit.t < best_t {
                    best_t = hit.t;
                    best_hit = Some(hit);
                }
            }
        }
        return best_hit;
    }
    if let Some(left) = &node.left {
        if let Some(hit) = intersect_bvh(ray, left, scene, time) {
            best_t = hit.t;
            best_hit = Some(hit);
        }
    }
    if let Some(right) = &node.right {
        if let Some(hit) = intersect_bvh(ray, right, scene, time) {
            if hit.t < best_t {
                best_hit = Some(hit);
            }
        }
    }
    best_hit
}

pub struct Fog {
    pub color: Vec3,
    pub density: f32,
    pub start: f32,
    pub end: f32,
}

impl Fog {
    pub fn linear(color: Vec3, start: f32, end: f32) -> Self {
        Self {
            color,
            density: 0.0,
            start,
            end,
        }
    }
    
    pub fn exponential(color: Vec3, density: f32) -> Self {
        Self {
            color,
            density,
            start: 0.0,
            end: f32::INFINITY,
        }
    }
    
    pub fn apply(&self, original_color: Vec3, distance: f32) -> Vec3 {
        if self.density > 0.0 {
            // Exponential fog
            let fog_factor = (-self.density * distance).exp();
            original_color.lerp(self.color, 1.0 - fog_factor)
        } else {
            // Linear fog
            let fog_factor = ((distance - self.start) / (self.end - self.start)).clamp(0.0, 1.0);
            original_color.lerp(self.color, fog_factor)
        }
    }
}

pub struct Skybox {
    pub top_color: Vec3,
    pub horizon_color: Vec3,
    pub night_top_color: Vec3,
    pub night_horizon_color: Vec3,
    pub sun_color: Vec3,
    pub sun_direction: Vec3,
    pub sun_size: f32,
    pub time_of_day: f32, // 0.0 = night, 1.0 = day
}

impl Skybox {
    pub fn gradient(day_top: Vec3, day_horizon: Vec3) -> Self {
        Skybox {
            top_color: day_top,
            horizon_color: day_horizon,
            night_top_color: Vec3::new(0.05, 0.05, 0.15),
            night_horizon_color: Vec3::new(0.1, 0.1, 0.2),
            sun_color: Vec3::new(1.0, 0.9, 0.7),
            sun_direction: Vec3::new(0.3, 0.6, 0.7).normalize(),
            sun_size: 0.04,
            time_of_day: 1.0,
        }
    }
    
    pub fn textured(day_top: Vec3, day_horizon: Vec3, night_top: Vec3, night_horizon: Vec3) -> Self {
        Skybox {
            top_color: day_top,
            horizon_color: day_horizon,
            night_top_color: night_top,
            night_horizon_color: night_horizon,
            sun_color: Vec3::new(1.0, 0.9, 0.6),
            sun_direction: Vec3::new(0.3, 0.6, 0.7).normalize(),
            sun_size: 0.04,
            time_of_day: 1.0,
        }
    }
    
    pub fn update_time_of_day(&mut self, time_factor: f32) {
        self.time_of_day = time_factor;
        // Update sun direction based on time
        let sun_angle = time_factor * std::f32::consts::PI - std::f32::consts::PI * 0.5;
        self.sun_direction = Vec3::new(0.3, sun_angle.sin(), sun_angle.cos()).normalize();
    }
    
    pub fn color_at(&self, direction: Vec3) -> Vec3 {
        let dir = direction.normalize();
        
        // Sky gradient based on height
        let t = (dir.y * 0.5 + 0.5).clamp(0.0, 1.0);
        
        // Interpolate between day and night
        let day_color = self.horizon_color.lerp(self.top_color, t);
        let night_color = self.night_horizon_color.lerp(self.night_top_color, t);
        let mut sky_color = night_color.lerp(day_color, self.time_of_day);
        
        // Add sun disk
        let sun_dot = dir.dot(self.sun_direction).max(0.0);
        if sun_dot > (1.0 - self.sun_size) && self.time_of_day > 0.3 {
            let sun_intensity = ((sun_dot - (1.0 - self.sun_size)) / self.sun_size).clamp(0.0, 1.0);
            sky_color = sky_color.lerp(self.sun_color, sun_intensity * self.time_of_day);
            sky_color
        } else {
            // Add sun glow
            let glow_size = self.sun_size * 3.0;
            if sun_dot > (1.0 - glow_size) && self.time_of_day > 0.2 {
                let glow_intensity = ((sun_dot - (1.0 - glow_size)) / glow_size).clamp(0.0, 1.0);
                let glow_color = self.sun_color * 0.3;
                sky_color = sky_color.lerp(sky_color + glow_color, glow_intensity * self.time_of_day * 0.5);
                sky_color
            } else {
                // Night stars
                if self.time_of_day < 0.4 {
                    // hash-based star field
                    let h = (dir.x * 157.0 + dir.y * 311.0 + dir.z * 653.0).sin().abs();
                    let star = if h > 0.995 { (h - 0.995) * 200.0 } else { 0.0 };
                    let star_color = Vec3::new(1.0, 1.0, 1.0) * star;
                    sky_color + star_color * (0.6 - self.time_of_day)
                } else {
                    sky_color
                }
            }
        }
    }
}

pub struct HitInfo {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub uv: (f32, f32),
}

// Enhanced raytracing parameters
const MAX_DEPTH: i32 = 6;
const SAMPLES_PER_PIXEL: i32 = 1;
const RUSSIAN_ROULETTE_DEPTH: i32 = 3;
const MIN_CONTRIBUTION: f32 = 0.001;

pub fn trace_ray(ray: &Ray, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    // Russian roulette termination
    if depth >= RUSSIAN_ROULETTE_DEPTH {
        let termination_prob = 0.9_f32.powf((depth - RUSSIAN_ROULETTE_DEPTH) as f32);
        if rand::random::<f32>() > termination_prob || depth >= MAX_DEPTH {
            return Vec3::zero();
        }
    }
    
    // Transform ray for scene rotation
    let rotated_origin = ray.origin.rotate_y(-rotation_y);
    let rotated_direction = ray.direction.rotate_y(-rotation_y);
    let rotated_ray = Ray::new(rotated_origin, rotated_direction);
    
    // Find closest intersection
    if let Some(hit) = intersect_scene(&rotated_ray, scene, time) {
        let color = shade_hit(ray, &hit, scene, depth, time, rotation_y);
        
        // Apply fog if present
        if let Some(fog) = &scene.fog {
            fog.apply(color, hit.t)
        } else {
            color
        }
    } else {
        // Background/skybox
        if let Some(skybox) = &scene.skybox {
            skybox.color_at(ray.direction)
        } else {
            // Default gradient sky
            let t = 0.5 * (ray.direction.normalize().y + 1.0);
            Vec3::new(0.5, 0.7, 1.0).lerp(Vec3::new(1.0, 1.0, 1.0), t)
        }
    }
}

fn intersect_scene(ray: &Ray, scene: &Scene, time: f32) -> Option<HitInfo> {
    if let Some(bvh) = &scene.bvh {
        intersect_bvh(ray, bvh, scene, time)
    } else {
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
}

fn shade_hit(ray: &Ray, hit: &HitInfo, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    let mut color = Vec3::zero();
    
    // Sample material texture
    let albedo = hit.material.sample_texture(hit.uv, time);
    
    // Emissive materials
    if hit.material.is_emissive() {
        return hit.material.emissive * albedo;
    }
    
    // Ambient lighting
    color = color + scene.ambient_light * albedo;
    
    // Direct lighting from all light sources
    for light in &scene.lights {
        color = color + calculate_direct_lighting(ray, hit, light, albedo, scene, time);
    }
    
    // Reflection
    if hit.material.is_reflective() && depth < MAX_DEPTH {
        let reflect_contribution = calculate_reflection(ray, hit, scene, depth, time, rotation_y);
        color = color + reflect_contribution * hit.material.reflectivity;
    }
    
    // Refraction/Transmission
    if hit.material.is_transparent() && depth < MAX_DEPTH {
        let refract_contribution = calculate_refraction(ray, hit, scene, depth, time, rotation_y);
        color = color.lerp(refract_contribution, hit.material.transparency);
    }
    
    color
}

fn calculate_direct_lighting(ray: &Ray, hit: &HitInfo, light: &Light, albedo: Vec3, scene: &Scene, time: f32) -> Vec3 {
    let light_dir = light.get_light_direction(hit.point);
    let light_distance = match light.light_type {
        LightType::Directional(_) => f32::INFINITY,
        _ => (light.position - hit.point).length(),
    };
    
    // Shadow test
    let shadow_ray = Ray::new(hit.point + hit.normal * 0.001, light_dir);
    let in_shadow = intersect_scene(&shadow_ray, scene, time)
        .map(|shadow_hit| shadow_hit.t < light_distance)
        .unwrap_or(false);
    
    if in_shadow {
        return Vec3::zero();
    }
    
    // Light attenuation and spot factor
    let attenuation = light.get_attenuation(hit.point);
    let spot_factor = light.get_spot_factor(hit.point);
    let light_intensity = light.intensity * attenuation * spot_factor;
    
    if light_intensity <= 0.0 {
        return Vec3::zero();
    }
    
    // Lambertian diffuse
    let n_dot_l = hit.normal.dot(light_dir).max(0.0);
    let diffuse = albedo * light.color * light_intensity * n_dot_l / std::f32::consts::PI;
    
    // Blinn-Phong specular
    let view_dir = (-ray.direction).normalize();
    let half_dir = (view_dir + light_dir).normalize();
    let n_dot_h = hit.normal.dot(half_dir).max(0.0);
    
    let shininess = (1.0 - hit.material.roughness) * 256.0 + 1.0;
    let specular_strength = hit.material.specular;
    let specular = light.color * light_intensity * n_dot_h.powf(shininess) * specular_strength;
    
    diffuse + specular
}

fn calculate_reflection(ray: &Ray, hit: &HitInfo, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    let reflect_dir = ray.direction.reflect(hit.normal);
    let reflect_ray = Ray::new(hit.point + hit.normal * 0.001, reflect_dir);
    trace_ray(&reflect_ray, scene, depth + 1, time, rotation_y)
}

fn calculate_refraction(ray: &Ray, hit: &HitInfo, scene: &Scene, depth: i32, time: f32, rotation_y: f32) -> Vec3 {
    let entering = ray.direction.dot(hit.normal) < 0.0;
    let eta = if entering {
        1.0 / hit.material.refraction_index
    } else {
        hit.material.refraction_index
    };
    
    let normal = if entering { hit.normal } else { -hit.normal };
    
    if let Some(refract_dir) = (-ray.direction).refract(normal, eta) {
        let refract_ray = Ray::new(hit.point - normal * 0.001, refract_dir);
        trace_ray(&refract_ray, scene, depth + 1, time, rotation_y)
    } else {
        // Total internal reflection
        calculate_reflection(ray, hit, scene, depth, time, rotation_y)
    }
}

fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = Vec3::new(
            rng.gen::<f32>() * 2.0 - 1.0,
            rng.gen::<f32>() * 2.0 - 1.0,
            rng.gen::<f32>() * 2.0 - 1.0,
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

fn random_unit_vector() -> Vec3 {
    random_in_unit_sphere().normalize()
}