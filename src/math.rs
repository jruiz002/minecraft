#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    
    pub fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }
    
    pub fn one() -> Self {
        Vec3::new(1.0, 1.0, 1.0)
    }
    
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 1e-8 {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        } else {
            Vec3::new(0.0, 1.0, 0.0) // Default up vector
        }
    }
    
    pub fn reflect(self, normal: Vec3) -> Vec3 {
        self - normal * 2.0 * self.dot(normal)
    }
    
    pub fn refract(self, normal: Vec3, eta: f32) -> Option<Vec3> {
        let cos_i = -self.dot(normal);
        let sin_t2 = eta * eta * (1.0 - cos_i * cos_i);
        
        if sin_t2 > 1.0 {
            None // Total internal reflection
        } else {
            let cos_t = (1.0 - sin_t2).sqrt();
            Some(self * eta + normal * (eta * cos_i - cos_t))
        }
    }
    
    pub fn lerp(self, other: Vec3, t: f32) -> Vec3 {
        self * (1.0 - t) + other * t
    }
    
    pub fn rotate_x(self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x,
            self.y * cos_a - self.z * sin_a,
            self.y * sin_a + self.z * cos_a,
        )
    }
    
    pub fn rotate_y(self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a - self.z * sin_a,
            self.y,
            self.x * sin_a + self.z * cos_a,
        )
    }
    
    pub fn rotate_z(self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
            self.z,
        )
    }
    
    pub fn clamp(self, min: f32, max: f32) -> Vec3 {
        Vec3::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
        )
    }
    
    pub fn abs(self) -> Vec3 {
        Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
    
    pub fn min(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }
    
    pub fn max(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }
    
    pub fn component_div(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
    
    pub fn component_mul(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
    
    pub fn distance(self, other: Vec3) -> f32 {
        (self - other).length()
    }
    
    // For noise functions
    pub fn floor(self) -> Vec3 {
        Vec3::new(self.x.floor(), self.y.floor(), self.z.floor())
    }
    
    pub fn fract(self) -> Vec3 {
        Vec3::new(self.x.fract(), self.y.fract(), self.z.fract())
    }
    
    // Smoothstep interpolation
    pub fn smoothstep(edge0: f32, edge1: f32, t: f32) -> f32 {
        let t = ((t - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl std::ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray {
            origin,
            direction: direction.normalize(),
        }
    }
    
    pub fn point_at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// Fresnel reflectance calculation
pub fn fresnel(cos_theta: f32, eta: f32) -> f32 {
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let sin_phi = sin_theta / eta;
    
    if sin_phi >= 1.0 {
        return 1.0; // Total internal reflection
    }
    
    let cos_phi = (1.0 - sin_phi * sin_phi).sqrt();
    
    let rs = ((eta * cos_theta - cos_phi) / (eta * cos_theta + cos_phi)).powi(2);
    let rp = ((cos_theta - eta * cos_phi) / (cos_theta + eta * cos_phi)).powi(2);
    
    (rs + rp) / 2.0
}

// Schlick's approximation for Fresnel reflectance
pub fn schlick(cos_theta: f32, eta: f32) -> f32 {
    let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}

// Noise functions for procedural textures
pub fn hash(p: Vec3) -> f32 {
    let p3 = (p * 0.1031).fract();
    let p3_x = p3.x + p3.y * 19.19 + p3.z * 213.0;
    (p3_x.sin() * 43758.5453).fract()
}

pub fn noise(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p.fract();
    
    // Smooth interpolation
    let u = f * f * (Vec3::new(3.0, 3.0, 3.0) - f * 2.0);
    
    // Sample corners of cube
    let a = hash(i + Vec3::new(0.0, 0.0, 0.0));
    let b = hash(i + Vec3::new(1.0, 0.0, 0.0));
    let c = hash(i + Vec3::new(0.0, 1.0, 0.0));
    let d = hash(i + Vec3::new(1.0, 1.0, 0.0));
    let e = hash(i + Vec3::new(0.0, 0.0, 1.0));
    let f_val = hash(i + Vec3::new(1.0, 0.0, 1.0));
    let g = hash(i + Vec3::new(0.0, 1.0, 1.0));
    let h = hash(i + Vec3::new(1.0, 1.0, 1.0));
    
    // Trilinear interpolation
    let mix_ab = a * (1.0 - u.x) + b * u.x;
    let mix_cd = c * (1.0 - u.x) + d * u.x;
    let mix_ef = e * (1.0 - u.x) + f_val * u.x;
    let mix_gh = g * (1.0 - u.x) + h * u.x;
    
    let mix_abcd = mix_ab * (1.0 - u.y) + mix_cd * u.y;
    let mix_efgh = mix_ef * (1.0 - u.y) + mix_gh * u.y;
    
    mix_abcd * (1.0 - u.z) + mix_efgh * u.z
}

// Fractal Brownian Motion for complex noise
pub fn fbm(p: Vec3, octaves: i32, persistence: f32, lacunarity: f32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;
    
    for _ in 0..octaves {
        value += noise(p * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    
    value / max_value
}

// Utility functions for color manipulation
pub fn gamma_correct(color: Vec3, gamma: f32) -> Vec3 {
    Vec3::new(
        color.x.powf(1.0 / gamma),
        color.y.powf(1.0 / gamma),
        color.z.powf(1.0 / gamma),
    )
}

pub fn tone_map_reinhard(color: Vec3) -> Vec3 {
    color.component_div(&(color + Vec3::one()))
}

pub fn tone_map_aces(color: Vec3) -> Vec3 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    
    let numerator = color.component_mul(&(color * a + Vec3::new(b, b, b)));
    let denominator = color.component_mul(&(color * c + Vec3::new(d, d, d))) + Vec3::new(e, e, e);
    numerator.component_div(&denominator)
}