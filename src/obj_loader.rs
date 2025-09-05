use crate::math::Vec3;
use crate::materials::Material;
use crate::primitives::{Triangle, Primitive};
use std::fs;
use std::io::{BufRead, BufReader};

pub struct ObjModel {
    pub triangles: Vec<Box<dyn Primitive>>,
}

impl ObjModel {
    pub fn load_from_file(path: &str, material: Material) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut vertices: Vec<Vec3> = Vec::new();
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    // Vertex
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse()?;
                        let y: f32 = parts[2].parse()?;
                        let z: f32 = parts[3].parse()?;
                        vertices.push(Vec3::new(x, y, z));
                    }
                },
                "f" => {
                    // Face
                    if parts.len() >= 4 {
                        // Parse face indices (OBJ indices are 1-based)
                        let mut face_vertices = Vec::new();
                        for i in 1..parts.len() {
                            let vertex_data = parts[i];
                            // Handle vertex/texture/normal format (v/vt/vn or just v)
                            let vertex_index: usize = vertex_data
                                .split('/')
                                .next()
                                .unwrap()
                                .parse::<usize>()?
                                .saturating_sub(1); // Convert to 0-based
                            
                            if vertex_index < vertices.len() {
                                face_vertices.push(vertices[vertex_index]);
                            }
                        }
                        
                        // Triangulate the face (simple fan triangulation)
                        if face_vertices.len() >= 3 {
                            for i in 1..face_vertices.len() - 1 {
                                let triangle = Triangle::new(
                                    face_vertices[0],
                                    face_vertices[i],
                                    face_vertices[i + 1],
                                    material.clone(),
                                );
                                triangles.push(Box::new(triangle));
                            }
                        }
                    }
                },
                _ => {} // Ignore other lines
            }
        }
        
        Ok(ObjModel { triangles })
    }
    
    pub fn create_cube_obj(center: Vec3, size: f32, material: Material) -> Self {
        let half = size / 2.0;
        
        // Define cube vertices
        let vertices = vec![
            center + Vec3::new(-half, -half, -half), // 0
            center + Vec3::new( half, -half, -half), // 1
            center + Vec3::new( half,  half, -half), // 2
            center + Vec3::new(-half,  half, -half), // 3
            center + Vec3::new(-half, -half,  half), // 4
            center + Vec3::new( half, -half,  half), // 5
            center + Vec3::new( half,  half,  half), // 6
            center + Vec3::new(-half,  half,  half), // 7
        ];
        
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        // Define cube faces (12 triangles)
        let faces = [
            // Front face
            [0, 1, 2], [0, 2, 3],
            // Back face
            [5, 4, 7], [5, 7, 6],
            // Left face
            [4, 0, 3], [4, 3, 7],
            // Right face
            [1, 5, 6], [1, 6, 2],
            // Top face
            [3, 2, 6], [3, 6, 7],
            // Bottom face
            [4, 5, 1], [4, 1, 0],
        ];
        
        for face in faces.iter() {
            let triangle = Triangle::new(
                vertices[face[0]],
                vertices[face[1]],
                vertices[face[2]],
                material.clone(),
            );
            triangles.push(Box::new(triangle));
        }
        
        ObjModel { triangles }
    }
    
    pub fn create_teapot_placeholder(center: Vec3, size: f32, material: Material) -> Self {
        // Create a simple approximation of a teapot using spheres and cylinders
        // represented as triangle meshes
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        // Create a simple teapot-like shape using icospheres
        let teapot_triangles = Self::create_icosphere(center, size * 0.7, 2, material.clone());
        triangles.extend(teapot_triangles);
        
        // Add a handle (simplified)
        let handle_center = center + Vec3::new(size * 0.8, 0.0, 0.0);
        let handle_triangles = Self::create_torus(handle_center, size * 0.3, size * 0.1, 8, 16, material.clone());
        triangles.extend(handle_triangles);
        
        // Add a spout (simplified)
        let spout_center = center + Vec3::new(-size * 0.8, 0.0, 0.0);
        let spout_triangles = Self::create_cylinder(spout_center, size * 0.1, size * 0.5, 8, material);
        triangles.extend(spout_triangles);
        
        ObjModel { triangles }
    }
    
    fn create_icosphere(center: Vec3, radius: f32, _subdivisions: u32, material: Material) -> Vec<Box<dyn Primitive>> {
        // Simple icosphere implementation
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        // Start with an icosahedron
        let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let vertices = vec![
            Vec3::new(-1.0, t, 0.0).normalize() * radius + center,
            Vec3::new(1.0, t, 0.0).normalize() * radius + center,
            Vec3::new(-1.0, -t, 0.0).normalize() * radius + center,
            Vec3::new(1.0, -t, 0.0).normalize() * radius + center,
            Vec3::new(0.0, -1.0, t).normalize() * radius + center,
            Vec3::new(0.0, 1.0, t).normalize() * radius + center,
            Vec3::new(0.0, -1.0, -t).normalize() * radius + center,
            Vec3::new(0.0, 1.0, -t).normalize() * radius + center,
            Vec3::new(t, 0.0, -1.0).normalize() * radius + center,
            Vec3::new(t, 0.0, 1.0).normalize() * radius + center,
            Vec3::new(-t, 0.0, -1.0).normalize() * radius + center,
            Vec3::new(-t, 0.0, 1.0).normalize() * radius + center,
        ];
        
        // Define icosahedron faces
        let faces = [
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ];
        
        for face in faces.iter() {
            let triangle = Triangle::new(
                vertices[face[0]],
                vertices[face[1]],
                vertices[face[2]],
                material.clone(),
            );
            triangles.push(Box::new(triangle));
        }
        
        triangles
    }
    
    fn create_torus(center: Vec3, major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32, material: Material) -> Vec<Box<dyn Primitive>> {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        for i in 0..major_segments {
            for j in 0..minor_segments {
                let u1 = (i as f32) / (major_segments as f32) * 2.0 * std::f32::consts::PI;
                let u2 = ((i + 1) as f32) / (major_segments as f32) * 2.0 * std::f32::consts::PI;
                let v1 = (j as f32) / (minor_segments as f32) * 2.0 * std::f32::consts::PI;
                let v2 = ((j + 1) as f32) / (minor_segments as f32) * 2.0 * std::f32::consts::PI;
                
                let p1 = torus_point(center, major_radius, minor_radius, u1, v1);
                let p2 = torus_point(center, major_radius, minor_radius, u2, v1);
                let p3 = torus_point(center, major_radius, minor_radius, u2, v2);
                let p4 = torus_point(center, major_radius, minor_radius, u1, v2);
                
                triangles.push(Box::new(Triangle::new(p1, p2, p3, material.clone())));
                triangles.push(Box::new(Triangle::new(p1, p3, p4, material.clone())));
            }
        }
        
        triangles
    }
    
    fn create_cylinder(center: Vec3, radius: f32, height: f32, segments: u32, material: Material) -> Vec<Box<dyn Primitive>> {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        let half_height = height / 2.0;
        
        for i in 0..segments {
            let angle1 = (i as f32) / (segments as f32) * 2.0 * std::f32::consts::PI;
            let angle2 = ((i + 1) as f32) / (segments as f32) * 2.0 * std::f32::consts::PI;
            
            let x1 = angle1.cos() * radius;
            let z1 = angle1.sin() * radius;
            let x2 = angle2.cos() * radius;
            let z2 = angle2.sin() * radius;
            
            // Side faces
            let p1 = center + Vec3::new(x1, -half_height, z1);
            let p2 = center + Vec3::new(x2, -half_height, z2);
            let p3 = center + Vec3::new(x2, half_height, z2);
            let p4 = center + Vec3::new(x1, half_height, z1);
            
            triangles.push(Box::new(Triangle::new(p1, p2, p3, material.clone())));
            triangles.push(Box::new(Triangle::new(p1, p3, p4, material.clone())));
            
            // Top and bottom caps
            let top_center = center + Vec3::new(0.0, half_height, 0.0);
            let bottom_center = center + Vec3::new(0.0, -half_height, 0.0);
            
            triangles.push(Box::new(Triangle::new(top_center, p4, p3, material.clone())));
            triangles.push(Box::new(Triangle::new(bottom_center, p1, p2, material.clone())));
        }
        
        triangles
    }
}

fn torus_point(center: Vec3, major_radius: f32, minor_radius: f32, u: f32, v: f32) -> Vec3 {
    let x = (major_radius + minor_radius * v.cos()) * u.cos();
    let y = minor_radius * v.sin();
    let z = (major_radius + minor_radius * v.cos()) * u.sin();
    center + Vec3::new(x, y, z)
}