use crate::math::Vec3;
use crate::materials::Material;
use crate::primitives::{Triangle, Primitive};
use std::fs;
use std::io::{BufRead, BufReader};

pub struct ObjModel {
    pub triangles: Vec<Box<dyn Primitive>>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl ObjModel {
    pub fn load_from_file(path: &str, material: Material) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut vertices: Vec<Vec3> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut uvs: Vec<(f32, f32)> = Vec::new();
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        let mut bounds_min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut bounds_max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() || parts[0].starts_with('#') {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    // Vertex position
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse()?;
                        let y: f32 = parts[2].parse()?;
                        let z: f32 = parts[3].parse()?;
                        let vertex = Vec3::new(x, y, z);
                        
                        // Update bounds
                        bounds_min = bounds_min.min(vertex);
                        bounds_max = bounds_max.max(vertex);
                        
                        vertices.push(vertex);
                    }
                },
                "vn" => {
                    // Vertex normal
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse()?;
                        let y: f32 = parts[2].parse()?;
                        let z: f32 = parts[3].parse()?;
                        normals.push(Vec3::new(x, y, z).normalize());
                    }
                },
                "vt" => {
                    // Texture coordinate
                    if parts.len() >= 3 {
                        let u: f32 = parts[1].parse()?;
                        let v: f32 = parts[2].parse()?;
                        uvs.push((u, v));
                    }
                },
                "f" => {
                    // Face
                    if parts.len() >= 4 {
                        let mut face_data = Vec::new();
                        
                        for i in 1..parts.len() {
                            let vertex_data = parts[i];
                            let indices: Vec<&str> = vertex_data.split('/').collect();
                            
                            // Parse vertex index (required)
                            let vertex_index: usize = indices[0].parse::<usize>()?.saturating_sub(1);
                            
                            // Parse texture coordinate index (optional)
                            let uv_index = if indices.len() > 1 && !indices[1].is_empty() {
                                Some(indices[1].parse::<usize>()?.saturating_sub(1))
                            } else {
                                None
                            };
                            
                            // Parse normal index (optional)
                            let normal_index = if indices.len() > 2 && !indices[2].is_empty() {
                                Some(indices[2].parse::<usize>()?.saturating_sub(1))
                            } else {
                                None
                            };
                            
                            if vertex_index < vertices.len() {
                                let vertex = vertices[vertex_index];
                                let uv = uv_index.and_then(|i| uvs.get(i)).copied().unwrap_or((0.0, 0.0));
                                let normal = normal_index.and_then(|i| normals.get(i)).copied();
                                
                                face_data.push((vertex, uv, normal));
                            }
                        }
                        
                        // Triangulate the face (fan triangulation for n-gons)
                        if face_data.len() >= 3 {
                            for i in 1..face_data.len() - 1 {
                                let (v0, uv0, n0) = face_data[0];
                                let (v1, uv1, n1) = face_data[i];
                                let (v2, uv2, n2) = face_data[i + 1];
                                
                                // Calculate face normal if not provided
                                let _face_normal = if let (Some(n0), Some(n1), Some(n2)) = (n0, n1, n2) {
                                    // Use average of vertex normals for smooth shading
                                    ((n0 + n1 + n2) / 3.0).normalize()
                                } else {
                                    // Calculate geometric normal
                                    (v1 - v0).cross(v2 - v0).normalize()
                                };
                                
                                let triangle = Triangle::new_with_uvs(v0, v1, v2, uv0, uv1, uv2, material.clone());
                                triangles.push(Box::new(triangle));
                            }
                        }
                    }
                },
                _ => {} // Ignore other OBJ commands
            }
        }
        
        Ok(ObjModel {
            triangles,
            bounds_min,
            bounds_max,
        })
    }
    
    pub fn create_minecraft_tree(center: Vec3, size: f32, _material: Material) -> Self {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        let trunk_material = Material::minecraft_wood();
        let leaves_material = Material::minecraft_grass(); // Using grass for leaves
        
        let trunk_height = size * 0.6;
        let trunk_radius = size * 0.1;
        let crown_radius = size * 0.4;
        let crown_height = size * 0.5;
        
        // Create trunk (cylinder approximation with triangles)
        let trunk_segments = 8;
        for i in 0..trunk_segments {
            let angle1 = (i as f32) / (trunk_segments as f32) * 2.0 * std::f32::consts::PI;
            let angle2 = ((i + 1) as f32) / (trunk_segments as f32) * 2.0 * std::f32::consts::PI;
            
            let x1 = center.x + angle1.cos() * trunk_radius;
            let z1 = center.z + angle1.sin() * trunk_radius;
            let x2 = center.x + angle2.cos() * trunk_radius;
            let z2 = center.z + angle2.sin() * trunk_radius;
            
            // Trunk side faces
            let v1 = Vec3::new(x1, center.y, z1);
            let v2 = Vec3::new(x2, center.y, z2);
            let v3 = Vec3::new(x2, center.y + trunk_height, z2);
            let v4 = Vec3::new(x1, center.y + trunk_height, z1);
            
            triangles.push(Box::new(Triangle::new(v1, v2, v3, trunk_material.clone())));
            triangles.push(Box::new(Triangle::new(v1, v3, v4, trunk_material.clone())));
        }
        
        // Create leaves (simplified icosphere)
        let crown_center = center + Vec3::new(0.0, trunk_height + crown_height * 0.5, 0.0);
        let leaves_triangles = Self::create_icosphere(crown_center, crown_radius, 2, leaves_material);
        triangles.extend(leaves_triangles);
        
        // Calculate bounds
        let bounds_min = center - Vec3::new(crown_radius, 0.0, crown_radius);
        let bounds_max = center + Vec3::new(crown_radius, trunk_height + crown_height, crown_radius);
        
        ObjModel {
            triangles,
            bounds_min,
            bounds_max,
        }
    }
    
    pub fn create_minecraft_house(center: Vec3, size: f32) -> Self {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        let wall_material = Material::minecraft_wood();
        let roof_material = Material::minecraft_stone();
        let window_material = Material::minecraft_glass();
        
        let half_size = size * 0.5;
        let height = size * 0.8;
        let roof_height = size * 0.4;
        
        // House walls (4 walls as triangles)
        let corners = [
            Vec3::new(-half_size, 0.0, -half_size),
            Vec3::new(half_size, 0.0, -half_size),
            Vec3::new(half_size, 0.0, half_size),
            Vec3::new(-half_size, 0.0, half_size),
        ];
        
        for i in 0..4 {
            let next = (i + 1) % 4;
            let bottom1 = center + corners[i];
            let bottom2 = center + corners[next];
            let top1 = bottom1 + Vec3::new(0.0, height, 0.0);
            let top2 = bottom2 + Vec3::new(0.0, height, 0.0);
            
            // Wall triangles
            triangles.push(Box::new(Triangle::new(bottom1, bottom2, top2, wall_material.clone())));
            triangles.push(Box::new(Triangle::new(bottom1, top2, top1, wall_material.clone())));
            
            // Add windows to front and back walls
            if i == 0 || i == 2 {
                let window_size = size * 0.2;
                let window_center = (bottom1 + bottom2) * 0.5 + Vec3::new(0.0, height * 0.6, 0.0);
                let window_half = Vec3::new(window_size * 0.5, window_size * 0.5, 0.0);
                
                // Simple window (just a quad)
                let w1 = window_center - window_half;
                let w2 = window_center + Vec3::new(window_half.x, -window_half.y, 0.0);
                let w3 = window_center + window_half;
                let w4 = window_center + Vec3::new(-window_half.x, window_half.y, 0.0);
                
                triangles.push(Box::new(Triangle::new(w1, w2, w3, window_material.clone())));
                triangles.push(Box::new(Triangle::new(w1, w3, w4, window_material.clone())));
            }
        }
        
        // Roof (simple triangular roof)
        let roof_peak = center + Vec3::new(0.0, height + roof_height, 0.0);
        let roof_base_1 = center + Vec3::new(-half_size, height, -half_size);
        let roof_base_2 = center + Vec3::new(half_size, height, -half_size);
        let roof_base_3 = center + Vec3::new(half_size, height, half_size);
        let roof_base_4 = center + Vec3::new(-half_size, height, half_size);
        
        // Roof triangles
        triangles.push(Box::new(Triangle::new(roof_peak, roof_base_1, roof_base_2, roof_material.clone())));
        triangles.push(Box::new(Triangle::new(roof_peak, roof_base_2, roof_base_3, roof_material.clone())));
        triangles.push(Box::new(Triangle::new(roof_peak, roof_base_3, roof_base_4, roof_material.clone())));
        triangles.push(Box::new(Triangle::new(roof_peak, roof_base_4, roof_base_1, roof_material.clone())));
        
        // Floor
        triangles.push(Box::new(Triangle::new(roof_base_1, roof_base_3, roof_base_2, wall_material.clone())));
        triangles.push(Box::new(Triangle::new(roof_base_1, roof_base_4, roof_base_3, wall_material)));
        
        let bounds_min = center - Vec3::new(half_size, 0.0, half_size);
        let bounds_max = center + Vec3::new(half_size, height + roof_height, half_size);
        
        ObjModel {
            triangles,
            bounds_min,
            bounds_max,
        }
    }
    
    pub fn create_minecraft_windmill(center: Vec3, size: f32) -> Self {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        let stone_material = Material::minecraft_stone();
        let wood_material = Material::minecraft_wood();
        
        // Tower (cylinder)
        let tower_height = size * 1.5;
        let tower_radius = size * 0.3;
        let tower_segments = 12;
        
        for i in 0..tower_segments {
            let angle1 = (i as f32) / (tower_segments as f32) * 2.0 * std::f32::consts::PI;
            let angle2 = ((i + 1) as f32) / (tower_segments as f32) * 2.0 * std::f32::consts::PI;
            
            let x1 = center.x + angle1.cos() * tower_radius;
            let z1 = center.z + angle1.sin() * tower_radius;
            let x2 = center.x + angle2.cos() * tower_radius;
            let z2 = center.z + angle2.sin() * tower_radius;
            
            let bottom1 = Vec3::new(x1, center.y, z1);
            let bottom2 = Vec3::new(x2, center.y, z2);
            let top1 = Vec3::new(x1, center.y + tower_height, z1);
            let top2 = Vec3::new(x2, center.y + tower_height, z2);
            
            triangles.push(Box::new(Triangle::new(bottom1, bottom2, top2, stone_material.clone())));
            triangles.push(Box::new(Triangle::new(bottom1, top2, top1, stone_material.clone())));
        }
        
        // Windmill blades
        let blade_center = center + Vec3::new(0.0, tower_height * 0.8, tower_radius);
        let blade_length = size * 0.8;
        let blade_width = size * 0.1;
        
        for i in 0..4 {
            let angle = (i as f32) * std::f32::consts::PI * 0.5;
            let blade_end = blade_center + Vec3::new(
                angle.cos() * blade_length,
                angle.sin() * blade_length,
                0.0,
            );
            
            let blade_side1 = blade_center + Vec3::new(
                -angle.sin() * blade_width,
                angle.cos() * blade_width,
                0.0,
            );
            let blade_side2 = blade_center + Vec3::new(
                angle.sin() * blade_width,
                -angle.cos() * blade_width,
                0.0,
            );
            
            triangles.push(Box::new(Triangle::new(blade_center, blade_end, blade_side1, wood_material.clone())));
            triangles.push(Box::new(Triangle::new(blade_center, blade_side2, blade_end, wood_material.clone())));
        }
        
        let bounds_min = center - Vec3::new(blade_length, 0.0, blade_length);
        let bounds_max = center + Vec3::new(blade_length, tower_height, blade_length);
        
        ObjModel {
            triangles,
            bounds_min,
            bounds_max,
        }
    }
    
    fn create_icosphere(center: Vec3, radius: f32, _subdivisions: u32, material: Material) -> Vec<Box<dyn Primitive>> {
        let mut triangles: Vec<Box<dyn Primitive>> = Vec::new();
        
        // Start with icosahedron vertices
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
        
        // Icosahedron faces
        let faces = [
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ];
        
        for face in faces.iter() {
            let v0 = vertices[face[0]];
            let v1 = vertices[face[1]];
            let v2 = vertices[face[2]];
            
            triangles.push(Box::new(Triangle::new(v0, v1, v2, material.clone())));
        }
        
        triangles
    }
}