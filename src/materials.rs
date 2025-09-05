use crate::math::Vec3;
use crate::texture::Texture;

#[derive(Clone)]
pub struct Material {
    pub albedo: Vec3,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,
    pub texture: Option<Texture>,
    pub emissive: Vec3,
}

impl Material {
    pub fn new() -> Self {
        Material {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            specular: 0.1,
            transparency: 0.0,
            reflectivity: 0.1,
            refraction_index: 1.0,
            texture: None,
            emissive: Vec3::zero(),
        }
    }
    
    pub fn lambertian(albedo: Vec3) -> Self {
        Material {
            albedo,
            specular: 0.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refraction_index: 1.0,
            texture: None,
            emissive: Vec3::zero(),
        }
    }
    
    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Material {
            albedo,
            specular: 1.0 - roughness,
            transparency: 0.0,
            reflectivity: 0.8,
            refraction_index: 1.0,
            texture: None,
            emissive: Vec3::zero(),
        }
    }
    
    pub fn dielectric(refraction_index: f32) -> Self {
        Material {
            albedo: Vec3::one(),
            specular: 0.0,
            transparency: 0.9,
            reflectivity: 0.1,
            refraction_index,
            texture: None,
            emissive: Vec3::zero(),
        }
    }
    
    pub fn emissive(color: Vec3, intensity: f32) -> Self {
        Material {
            albedo: Vec3::zero(),
            specular: 0.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refraction_index: 1.0,
            texture: None,
            emissive: color * intensity,
        }
    }
    
    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.texture = Some(texture);
        self
    }
}