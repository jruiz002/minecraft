use crate::math::Vec3;

#[derive(Clone)]
pub enum TextureType {
    SolidColor(Vec3),
    Checker(Vec3, Vec3),
    AnimatedWater,
    Procedural(fn(f32, f32, f32) -> Vec3),
}

#[derive(Clone)]
pub struct Texture {
    pub texture_type: TextureType,
}

impl Texture {
    pub fn solid_color(color: Vec3) -> Self {
        Texture {
            texture_type: TextureType::SolidColor(color),
        }
    }
    
    pub fn checker(color1: Vec3, color2: Vec3) -> Self {
        Texture {
            texture_type: TextureType::Checker(color1, color2),
        }
    }
    
    pub fn animated_water() -> Self {
        Texture {
            texture_type: TextureType::AnimatedWater,
        }
    }
    
    pub fn procedural(func: fn(f32, f32, f32) -> Vec3) -> Self {
        Texture {
            texture_type: TextureType::Procedural(func),
        }
    }
    
    pub fn from_file(_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // For now, return a solid color since we're not loading actual files
        // In a real implementation, you would load the image file here
        Ok(Self::solid_color(Vec3::new(0.8, 0.8, 0.8)))
    }
    
    pub fn sample(&self, u: f32, v: f32, time: f32) -> Vec3 {
        match &self.texture_type {
            TextureType::SolidColor(color) => *color,
            TextureType::Checker(color1, color2) => {
                let checker_size = 8.0;
                let iu = (u * checker_size).floor() as i32;
                let iv = (v * checker_size).floor() as i32;
                if (iu + iv) % 2 == 0 {
                    *color1
                } else {
                    *color2
                }
            },
            TextureType::AnimatedWater => {
                let wave1 = (u * 10.0 + time * 2.0).sin();
                let wave2 = (v * 8.0 + time * 1.5).sin();
                let wave3 = ((u + v) * 6.0 + time * 3.0).sin();
                
                let intensity = (wave1 + wave2 + wave3) * 0.1 + 0.5;
                let base_color = Vec3::new(0.1, 0.3, 0.8);
                let foam_color = Vec3::new(0.8, 0.9, 1.0);
                
                base_color.lerp(foam_color, intensity.clamp(0.0, 1.0))
            },
            TextureType::Procedural(func) => func(u, v, time),
        }
    }
}

// Procedural texture functions
pub fn noise_texture(u: f32, v: f32, _time: f32) -> Vec3 {
    // Simple noise implementation
    let x = (u * 100.0).sin() * (v * 100.0).cos();
    let noise = (x * 43758.5453).fract();
    Vec3::new(noise, noise, noise)
}

pub fn wood_texture(u: f32, _v: f32, _time: f32) -> Vec3 {
    let rings = (u * 20.0).sin() * 0.5 + 0.5;
    let base = Vec3::new(0.6, 0.3, 0.1);
    let dark = Vec3::new(0.3, 0.15, 0.05);
    base.lerp(dark, rings)
}

pub fn marble_texture(u: f32, v: f32, time: f32) -> Vec3 {
    let x = u * 10.0 + (v * 8.0 + time * 0.5).sin() * 2.0;
    let marble = (x.sin() * 0.5 + 0.5).powf(3.0);
    let white = Vec3::new(0.9, 0.9, 0.9);
    let gray = Vec3::new(0.3, 0.3, 0.3);
    white.lerp(gray, marble)
}