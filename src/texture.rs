use crate::math::{Vec3, noise, fbm};

#[derive(Clone)]
pub enum TextureType {
    SolidColor(Vec3),
    Checker(Vec3, Vec3, f32),
    AnimatedWater,
    AnimatedFire,
    NetherPortal,
    Noise(NoiseConfig),
    MinecraftGrass,
    MinecraftStone,
    MinecraftWood,
    MinecraftDiamond,
    MinecraftGlowstone,
    MinecraftObsidian,
    MinecraftIron,
    MinecraftGold,
    Procedural(fn(f32, f32, f32) -> Vec3),
}

#[derive(Clone)]
pub struct NoiseConfig {
    pub octaves: i32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub scale: f32,
    pub color1: Vec3,
    pub color2: Vec3,
}

#[derive(Clone)]
pub struct Texture {
    pub texture_type: TextureType,
    pub scale: f32,
    pub offset: (f32, f32),
    pub rotation: f32,
}

impl Texture {
    pub fn solid_color(color: Vec3) -> Self {
        Texture {
            texture_type: TextureType::SolidColor(color),
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn checker(color1: Vec3, color2: Vec3, size: f32) -> Self {
        Texture {
            texture_type: TextureType::Checker(color1, color2, size),
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn animated_water() -> Self {
        Texture {
            texture_type: TextureType::AnimatedWater,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn animated_fire() -> Self {
        Texture {
            texture_type: TextureType::AnimatedFire,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn nether_portal() -> Self {
        Texture {
            texture_type: TextureType::NetherPortal,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn noise(octaves: i32, persistence: f32, lacunarity: f32, scale: f32, color1: Vec3, color2: Vec3) -> Self {
        Texture {
            texture_type: TextureType::Noise(NoiseConfig {
                octaves,
                persistence,
                lacunarity,
                scale,
                color1,
                color2,
            }),
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn procedural(func: fn(f32, f32, f32) -> Vec3) -> Self {
        Texture {
            texture_type: TextureType::Procedural(func),
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    // Minecraft-specific textures
    pub fn minecraft_grass() -> Self {
        Texture {
            texture_type: TextureType::MinecraftGrass,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_stone() -> Self {
        Texture {
            texture_type: TextureType::MinecraftStone,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_wood() -> Self {
        Texture {
            texture_type: TextureType::MinecraftWood,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_diamond() -> Self {
        Texture {
            texture_type: TextureType::MinecraftDiamond,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_glowstone() -> Self {
        Texture {
            texture_type: TextureType::MinecraftGlowstone,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_obsidian() -> Self {
        Texture {
            texture_type: TextureType::MinecraftObsidian,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_iron() -> Self {
        Texture {
            texture_type: TextureType::MinecraftIron,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn minecraft_gold() -> Self {
        Texture {
            texture_type: TextureType::MinecraftGold,
            scale: 1.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
        }
    }
    
    pub fn from_file(_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // For now, return a default texture since we're focusing on procedural textures
        Ok(Self::solid_color(Vec3::new(0.8, 0.8, 0.8)))
    }
    
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    
    pub fn with_offset(mut self, offset: (f32, f32)) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
    
    pub fn sample(&self, u: f32, v: f32, time: f32) -> Vec3 {
        // Apply transformations
        let mut u_transformed = (u + self.offset.0) * self.scale;
        let mut v_transformed = (v + self.offset.1) * self.scale;
        
        // Apply rotation if needed
        if self.rotation != 0.0 {
            let cos_r = self.rotation.cos();
            let sin_r = self.rotation.sin();
            let u_rot = u_transformed * cos_r - v_transformed * sin_r;
            let v_rot = u_transformed * sin_r + v_transformed * cos_r;
            u_transformed = u_rot;
            v_transformed = v_rot;
        }
        
        match &self.texture_type {
            TextureType::SolidColor(color) => *color,
            
            TextureType::Checker(color1, color2, size) => {
                let checker_size = *size;
                let iu = (u_transformed * checker_size).floor() as i32;
                let iv = (v_transformed * checker_size).floor() as i32;
                if (iu + iv) % 2 == 0 {
                    *color1
                } else {
                    *color2
                }
            },
            
            TextureType::AnimatedWater => {
                let wave1 = (u_transformed * 8.0 + time * 1.5).sin();
                let wave2 = (v_transformed * 6.0 + time * 2.0).sin();
                let wave3 = ((u_transformed + v_transformed) * 4.0 + time * 2.5).sin();
                let wave4 = ((u_transformed - v_transformed) * 10.0 + time * 1.8).cos();
                
                let intensity = (wave1 + wave2 + wave3 + wave4) * 0.125 + 0.5;
                let foam_factor = ((wave1 * wave2 + wave3 * wave4) * 0.3 + 0.7).clamp(0.0, 1.0);
                
                let deep_water = Vec3::new(0.05, 0.2, 0.6);
                let shallow_water = Vec3::new(0.2, 0.5, 0.8);
                let foam_color = Vec3::new(0.7, 0.8, 0.9);
                
                let water_color = deep_water.lerp(shallow_water, intensity);
                water_color.lerp(foam_color, foam_factor * 0.3)
            },
            
            TextureType::AnimatedFire => {
                let flame1 = (u_transformed * 3.0 + time * 8.0).sin() * 0.5 + 0.5;
                let flame2 = (v_transformed * 4.0 - time * 12.0).sin() * 0.5 + 0.5;
                let flame3 = ((u_transformed + v_transformed) * 6.0 + time * 10.0).cos() * 0.5 + 0.5;
                
                let intensity = (flame1 * flame2 * flame3).powf(0.5);
                let heat = (intensity * 2.0 - v_transformed).clamp(0.0, 1.0);
                
                let ember = Vec3::new(0.1, 0.05, 0.0);
                let orange = Vec3::new(1.0, 0.3, 0.05);
                let yellow = Vec3::new(1.0, 0.8, 0.1);
                let white = Vec3::new(1.0, 0.95, 0.8);
                
                if heat < 0.3 {
                    ember.lerp(orange, heat / 0.3)
                } else if heat < 0.7 {
                    orange.lerp(yellow, (heat - 0.3) / 0.4)
                } else {
                    yellow.lerp(white, (heat - 0.7) / 0.3)
                }
            },
            
            TextureType::NetherPortal => {
                let portal1 = (u_transformed * 2.0 + time * 3.0).sin() * 0.5 + 0.5;
                let portal2 = (v_transformed * 3.0 - time * 4.0).cos() * 0.5 + 0.5;
                let portal3 = ((u_transformed + v_transformed) * 1.5 + time * 5.0).sin() * 0.5 + 0.5;
                
                let energy = (portal1 * portal2 + portal3) * 0.5;
                let swirl = ((u_transformed - 0.5).atan2(v_transformed - 0.5) + time * 2.0).sin() * 0.5 + 0.5;
                
                let purple = Vec3::new(0.4, 0.1, 0.8);
                let magenta = Vec3::new(0.8, 0.2, 0.6);
                let white = Vec3::new(0.9, 0.8, 1.0);
                
                let base = purple.lerp(magenta, energy);
                base.lerp(white, swirl * 0.4)
            },
            
            TextureType::MinecraftGrass => {
                let noise_val = noise(Vec3::new(u_transformed * 16.0, v_transformed * 16.0, time * 0.1));
                let grass_base = Vec3::new(0.3, 0.6, 0.2);
                let grass_bright = Vec3::new(0.5, 0.8, 0.3);
                let dirt = Vec3::new(0.4, 0.25, 0.1);
                
                // Add some dirt patches
                let dirt_noise = noise(Vec3::new(u_transformed * 8.0, v_transformed * 8.0, 0.0));
                if dirt_noise > 0.7 {
                    dirt
                } else {
                    grass_base.lerp(grass_bright, noise_val)
                }
            },
            
            TextureType::MinecraftStone => {
                let noise1 = noise(Vec3::new(u_transformed * 8.0, v_transformed * 8.0, 0.0));
                let noise2 = noise(Vec3::new(u_transformed * 16.0, v_transformed * 16.0, 0.0));
                let combined = (noise1 + noise2 * 0.5) / 1.5;
                
                let stone_dark = Vec3::new(0.4, 0.4, 0.4);
                let stone_light = Vec3::new(0.7, 0.7, 0.7);
                
                stone_dark.lerp(stone_light, combined)
            },
            
            TextureType::MinecraftWood => {
                let ring_noise = ((v_transformed * 12.0).sin() * 0.3 + 0.7).clamp(0.0, 1.0);
                let grain_noise = noise(Vec3::new(u_transformed * 32.0, v_transformed * 4.0, 0.0));
                
                let wood_dark = Vec3::new(0.4, 0.2, 0.1);
                let wood_light = Vec3::new(0.7, 0.4, 0.2);
                
                let base = wood_dark.lerp(wood_light, ring_noise);
                base.lerp(wood_dark, grain_noise * 0.3)
            },
            
            TextureType::MinecraftDiamond => {
                let sparkle1 = ((u_transformed * 20.0 + time).sin() * (v_transformed * 20.0 + time).cos()).abs();
                let sparkle2 = ((u_transformed * 15.0 - time * 0.7).cos() * (v_transformed * 15.0 - time * 0.7).sin()).abs();
                let sparkles = (sparkle1 + sparkle2) * 0.5;
                
                let diamond_base = Vec3::new(0.6, 0.8, 1.0);
                let diamond_bright = Vec3::new(0.9, 0.95, 1.0);
                
                diamond_base.lerp(diamond_bright, sparkles.powf(2.0))
            },
            
            TextureType::MinecraftGlowstone => {
                let glow1 = (u_transformed * 8.0 + time * 2.0).sin() * 0.5 + 0.5;
                let glow2 = (v_transformed * 8.0 + time * 1.5).cos() * 0.5 + 0.5;
                let pulse = (time * 4.0).sin() * 0.1 + 0.9;
                
                let intensity = (glow1 * glow2 * pulse).clamp(0.0, 1.0);
                
                let glow_dim = Vec3::new(0.8, 0.6, 0.2);
                let glow_bright = Vec3::new(1.0, 0.9, 0.5);
                
                glow_dim.lerp(glow_bright, intensity)
            },
            
            TextureType::MinecraftObsidian => {
                let noise_val = noise(Vec3::new(u_transformed * 12.0, v_transformed * 12.0, 0.0));
                let reflection = ((u_transformed + v_transformed) * 16.0).sin() * 0.5 + 0.5;
                
                let obsidian_base = Vec3::new(0.05, 0.02, 0.1);
                let obsidian_highlight = Vec3::new(0.2, 0.1, 0.3);
                
                let base = obsidian_base.lerp(obsidian_highlight, noise_val);
                base.lerp(obsidian_highlight, reflection * 0.3)
            },
            
            TextureType::MinecraftIron => {
                let noise_val = fbm(Vec3::new(u_transformed * 10.0, v_transformed * 10.0, 0.0), 3, 0.5, 2.0);
                let scratches = ((u_transformed * 40.0).sin() * 0.1 + 0.9).clamp(0.0, 1.0);
                
                let iron_dark = Vec3::new(0.4, 0.4, 0.45);
                let iron_light = Vec3::new(0.7, 0.7, 0.75);
                
                let base = iron_dark.lerp(iron_light, noise_val);
                base * scratches
            },
            
            TextureType::MinecraftGold => {
                let noise_val = noise(Vec3::new(u_transformed * 8.0, v_transformed * 8.0, 0.0));
                let shine = ((u_transformed + v_transformed) * 12.0 + time).sin() * 0.5 + 0.5;
                
                let gold_base = Vec3::new(0.8, 0.6, 0.2);
                let gold_bright = Vec3::new(1.0, 0.9, 0.5);
                
                let base = gold_base.lerp(gold_bright, noise_val);
                base.lerp(gold_bright, shine * 0.4)
            },
            
            TextureType::Noise(config) => {
                let noise_val = fbm(
                    Vec3::new(u_transformed * config.scale, v_transformed * config.scale, time * 0.1),
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                );
                config.color1.lerp(config.color2, noise_val)
            },
            
            TextureType::Procedural(func) => func(u_transformed, v_transformed, time),
        }
    }
}

// Procedural texture functions
pub fn noise_texture(u: f32, v: f32, time: f32) -> Vec3 {
    let noise_val = noise(Vec3::new(u * 10.0, v * 10.0, time * 0.5));
    Vec3::new(noise_val, noise_val, noise_val)
}

pub fn wood_texture(u: f32, v: f32, _time: f32) -> Vec3 {
    let rings = (v * 20.0).sin() * 0.5 + 0.5;
    let grain = noise(Vec3::new(u * 100.0, v * 20.0, 0.0));
    
    let base = Vec3::new(0.6, 0.3, 0.1);
    let dark = Vec3::new(0.3, 0.15, 0.05);
    let ring_color = base.lerp(dark, rings);
    
    ring_color.lerp(dark, grain * 0.2)
}

pub fn marble_texture(u: f32, v: f32, time: f32) -> Vec3 {
    let x = u * 8.0 + (v * 6.0 + time * 0.2).sin() * 2.0;
    let marble = (x.sin() * 0.5 + 0.5).powf(2.0);
    
    let white = Vec3::new(0.95, 0.95, 0.9);
    let gray = Vec3::new(0.3, 0.3, 0.35);
    let dark = Vec3::new(0.1, 0.1, 0.15);
    
    if marble > 0.7 {
        white.lerp(gray, (marble - 0.7) / 0.3)
    } else {
        gray.lerp(dark, marble / 0.7)
    }
}

pub fn metal_texture(u: f32, v: f32, _time: f32) -> Vec3 {
    let brushed = ((u * 50.0).sin() * 0.1 + 0.9).clamp(0.0, 1.0);
    let noise_val = noise(Vec3::new(u * 20.0, v * 20.0, 0.0)) * 0.2;
    
    let metal_base = Vec3::new(0.6, 0.6, 0.7);
    metal_base * brushed + Vec3::new(noise_val, noise_val, noise_val)
}

pub fn fabric_texture(u: f32, v: f32, _time: f32) -> Vec3 {
    let weave_u = ((u * 32.0).sin() + 1.0) * 0.5;
    let weave_v = ((v * 32.0).sin() + 1.0) * 0.5;
    let weave = (weave_u * weave_v).clamp(0.0, 1.0);
    
    let fabric_base = Vec3::new(0.4, 0.2, 0.6);
    let fabric_bright = Vec3::new(0.6, 0.4, 0.8);
    
    fabric_base.lerp(fabric_bright, weave)
}

pub fn lava_texture(u: f32, v: f32, time: f32) -> Vec3 {
    let flow1 = (u * 4.0 + time * 2.0).sin();
    let flow2 = (v * 3.0 - time * 1.5).cos();
    let bubble = ((u + v) * 8.0 + time * 6.0).sin();
    
    let heat = ((flow1 + flow2 + bubble) * 0.33 + 1.0) * 0.5;
    
    let cool = Vec3::new(0.3, 0.05, 0.0);
    let warm = Vec3::new(1.0, 0.3, 0.0);
    let hot = Vec3::new(1.0, 0.8, 0.2);
    
    if heat < 0.5 {
        cool.lerp(warm, heat * 2.0)
    } else {
        warm.lerp(hot, (heat - 0.5) * 2.0)
    }
}