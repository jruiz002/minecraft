#[derive(Debug, Clone, Copy)]
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
        if len > 0.0 {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        } else {
            Vec3::zero()
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
    
    pub fn rotate_y(self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a - self.z * sin_a,
            self.y,
            self.x * sin_a + self.z * cos_a,
        )
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

pub fn fresnel(cos_theta: f32, eta: f32) -> f32 {
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let sin_phi = sin_theta / eta;
    
    if sin_phi >= 1.0 {
        return 1.0; // Total internal reflection
    }
    
    let cos_phi = (1.0 - sin_phi * sin_phi).sqrt();
    
    let rs = ((eta * cos_theta - cos_phi) / (eta * cos_theta + cos_phi)).powi(2);
    let rp = ((cos_theta - eta * cos_phi) / (cos_theta + eta * cos_phi)).powi(2);
    
    (rs + rp) / 2.0
}