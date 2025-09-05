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
    pub roughness: f32,
    pub metallic: f32,
    pub subsurface: f32,
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
            roughness: 0.5,
            metallic: 0.0,
            subsurface: 0.0,
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
            roughness: 1.0,
            metallic: 0.0,
            subsurface: 0.0,
        }
    }
    
    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Material {
            albedo,
            specular: 1.0 - roughness,
            transparency: 0.0,
            reflectivity: 0.9,
            refraction_index: 1.0,
            texture: None,
            emissive: Vec3::zero(),
            roughness,
            metallic: 1.0,
            subsurface: 0.0,
        }
    }
    
    pub fn dielectric(refraction_index: f32) -> Self {
        Material {
            albedo: Vec3::one(),
            specular: 0.0,
            transparency: 0.95,
            reflectivity: 0.05,
            refraction_index,
            texture: None,
            emissive: Vec3::zero(),
            roughness: 0.0,
            metallic: 0.0,
            subsurface: 0.0,
        }
    }
    
    pub fn emissive(color: Vec3, intensity: f32) -> Self {
        Material {
            albedo: color,
            specular: 0.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refraction_index: 1.0,
            texture: None,
            emissive: color * intensity,
            roughness: 1.0,
            metallic: 0.0,
            subsurface: 0.0,
        }
    }
    
    pub fn glass(tint: Vec3, refraction_index: f32) -> Self {
        Material {
            albedo: tint,
            specular: 0.9,
            transparency: 0.9,
            reflectivity: 0.1,
            refraction_index,
            texture: None,
            emissive: Vec3::zero(),
            roughness: 0.0,
            metallic: 0.0,
            subsurface: 0.0,
        }
    }
    
    pub fn water() -> Self {
        Material {
            albedo: Vec3::new(0.2, 0.6, 0.8),
            specular: 0.9,
            transparency: 0.8,
            reflectivity: 0.2,
            refraction_index: 1.33,
            texture: Some(Texture::animated_water()),
            emissive: Vec3::zero(),
            roughness: 0.1,
            metallic: 0.0,
            subsurface: 0.3,
        }
    }
    
    pub fn portal_material() -> Self {
        Material {
            albedo: Vec3::new(0.5, 0.1, 0.9),
            specular: 0.2,
            transparency: 0.7,
            reflectivity: 0.3,
            refraction_index: 1.2,
            texture: Some(Texture::nether_portal()),
            emissive: Vec3::new(0.3, 0.1, 0.6),
            roughness: 0.3,
            metallic: 0.0,
            subsurface: 0.8,
        }
    }
    
    // Builder pattern methods
    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.texture = Some(texture);
        self
    }
    
    pub fn with_emissive(mut self, emissive: Vec3) -> Self {
        self.emissive = emissive;
        self
    }
    
    pub fn with_properties(mut self, albedo: Vec3, specular: f32, transparency: f32, reflectivity: f32) -> Self {
        self.albedo = albedo;
        self.specular = specular;
        self.transparency = transparency;
        self.reflectivity = reflectivity;
        self
    }
    
    pub fn with_refraction(mut self, refraction_index: f32) -> Self {
        self.refraction_index = refraction_index;
        self
    }
    
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }
    
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }
    
    pub fn sample_texture(&self, uv: (f32, f32), time: f32) -> Vec3 {
        if let Some(ref texture) = self.texture {
            texture.sample(uv.0, uv.1, time)
        } else {
            self.albedo
        }
    }
    
    pub fn sample_texture_quality(&self, uv: (f32, f32), time: f32, quality: crate::texture::TextureQuality) -> Vec3 {
        if let Some(ref texture) = self.texture {
            texture.sample_quality(uv.0, uv.1, time, quality)
        } else {
            self.albedo
        }
    }
    
    // Physical properties for realistic rendering
    pub fn get_fresnel_reflectance(&self, cos_theta: f32) -> f32 {
        if self.metallic > 0.5 {
            // Metallic materials
            let f0 = self.albedo.length() / 3.0; // Simplified metallic fresnel
            f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
        } else {
            // Dielectric materials
            crate::math::schlick(cos_theta, self.refraction_index)
        }
    }
    
    pub fn is_emissive(&self) -> bool {
        self.emissive.length_squared() > 0.001
    }
    
    pub fn is_transparent(&self) -> bool {
        self.transparency > 0.001
    }
    
    pub fn is_reflective(&self) -> bool {
        self.reflectivity > 0.001
    }
}

// Predefined Minecraft-style materials
impl Material {
    pub fn minecraft_grass() -> Self {
        Material::lambertian(Vec3::new(0.4, 0.8, 0.2))
            .with_texture(Texture::minecraft_grass())
            .with_roughness(0.8)
    }
    
    pub fn minecraft_stone() -> Self {
        Material::lambertian(Vec3::new(0.5, 0.5, 0.5))
            .with_texture(Texture::minecraft_stone())
            .with_properties(Vec3::new(0.5, 0.5, 0.5), 0.2, 0.0, 0.1)
            .with_roughness(0.9)
    }
    
    pub fn minecraft_wood() -> Self {
        Material::lambertian(Vec3::new(0.6, 0.4, 0.2))
            .with_texture(Texture::minecraft_wood())
            .with_properties(Vec3::new(0.6, 0.4, 0.2), 0.1, 0.0, 0.05)
            .with_roughness(0.7)
    }
    
    pub fn minecraft_water() -> Self {
        Material::water()
    }
    
    pub fn minecraft_glass() -> Self {
        Material::glass(Vec3::new(0.9, 0.95, 1.0), 1.52)
            .with_texture(Texture::solid_color(Vec3::new(0.9, 0.95, 1.0)))
            .with_roughness(0.0)
    }
    
    pub fn minecraft_diamond() -> Self {
        Material::dielectric(2.42)
            .with_texture(Texture::minecraft_diamond())
            .with_properties(Vec3::new(0.7, 0.9, 1.0), 0.95, 0.3, 0.9)
            .with_roughness(0.0)
    }
    
    pub fn minecraft_obsidian() -> Self {
        Material::lambertian(Vec3::new(0.1, 0.05, 0.2))
            .with_texture(Texture::minecraft_obsidian())
            .with_properties(Vec3::new(0.1, 0.05, 0.2), 0.4, 0.0, 0.7)
            .with_roughness(0.2)
    }
    
    pub fn minecraft_glowstone() -> Self {
        Material::emissive(Vec3::new(1.0, 0.8, 0.4), 2.5)
            .with_texture(Texture::minecraft_glowstone())
            .with_roughness(0.6)
    }
    
    pub fn minecraft_campfire() -> Self {
        Material::emissive(Vec3::new(1.0, 0.4, 0.1), 4.0)
            .with_texture(Texture::animated_fire())
            .with_roughness(1.0)
    }
    
    pub fn minecraft_portal() -> Self {
        Material::portal_material()
    }
    
    pub fn minecraft_iron() -> Self {
        Material::metal(Vec3::new(0.56, 0.57, 0.58), 0.3)
            .with_texture(Texture::minecraft_iron())
    }
    
    pub fn minecraft_gold() -> Self {
        Material::metal(Vec3::new(1.0, 0.86, 0.57), 0.2)
            .with_texture(Texture::minecraft_gold())
    }
}

// Material blending for complex surfaces
pub fn blend_materials(mat1: &Material, mat2: &Material, factor: f32) -> Material {
    let t = factor.clamp(0.0, 1.0);
    
    Material {
        albedo: mat1.albedo.lerp(mat2.albedo, t),
        specular: mat1.specular * (1.0 - t) + mat2.specular * t,
        transparency: mat1.transparency * (1.0 - t) + mat2.transparency * t,
        reflectivity: mat1.reflectivity * (1.0 - t) + mat2.reflectivity * t,
        refraction_index: mat1.refraction_index * (1.0 - t) + mat2.refraction_index * t,
        texture: if t > 0.5 { mat2.texture.clone() } else { mat1.texture.clone() },
        emissive: mat1.emissive.lerp(mat2.emissive, t),
        roughness: mat1.roughness * (1.0 - t) + mat2.roughness * t,
        metallic: mat1.metallic * (1.0 - t) + mat2.metallic * t,
        subsurface: mat1.subsurface * (1.0 - t) + mat2.subsurface * t,
    }
}