mod math;
mod raytracer;
mod materials;
mod primitives;
mod texture;
mod obj_loader;

use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::time::Instant;

use raytracer::*;
use math::*;
use materials::*;
use primitives::*;
use texture::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let mut window = Window::new(
        "Raytracing Diorama - By [Tu Nombre]",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("{}", e));

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // 60 FPS cap

    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        45.0,
        WIDTH as f32 / HEIGHT as f32,
    );

    let mut scene = create_scene();
    let mut frame_buffer = vec![0u32; WIDTH * HEIGHT];
    let mut time = 0.0f32;
    let mut fps_counter = 0;
    let mut fps_timer = Instant::now();
    let mut rotation_y = 0.0f32;

    println!("Controles:");
    println!("WASD: Mover cámara");
    println!("QE: Subir/Bajar cámara");
    println!("R: Rotar escena");
    println!("ESC: Salir");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start_time = Instant::now();
        
        // Input handling
        handle_input(&window, &mut camera, &mut rotation_y);
        
        // Update time for animations
        time += 0.016; // Assume 60 FPS for time step
        update_scene(&mut scene, time);
        
        // Render with multithreading
        render_parallel(&scene, &camera, &mut frame_buffer, time, rotation_y);
        
        window.update_with_buffer(&frame_buffer, WIDTH, HEIGHT).unwrap();
        
        // FPS calculation
        fps_counter += 1;
        if fps_timer.elapsed().as_secs() >= 1 {
            println!("FPS: {}", fps_counter);
            fps_counter = 0;
            fps_timer = Instant::now();
        }
        
        let frame_time = start_time.elapsed();
        if frame_time.as_millis() < 16 {
            std::thread::sleep(std::time::Duration::from_millis(16 - frame_time.as_millis() as u64));
        }
    }
}

fn handle_input(window: &Window, camera: &mut Camera, rotation_y: &mut f32) {
    let speed = 0.1;
    let rotation_speed = 0.02;
    
    if window.is_key_down(Key::W) {
        camera.position = camera.position + camera.get_forward() * speed;
    }
    if window.is_key_down(Key::S) {
        camera.position = camera.position - camera.get_forward() * speed;
    }
    if window.is_key_down(Key::A) {
        camera.position = camera.position - camera.get_right() * speed;
    }
    if window.is_key_down(Key::D) {
        camera.position = camera.position + camera.get_right() * speed;
    }
    if window.is_key_down(Key::Q) {
        camera.position.y -= speed;
    }
    if window.is_key_down(Key::E) {
        camera.position.y += speed;
    }
    if window.is_key_down(Key::R) {
        *rotation_y += rotation_speed;
    }
}

fn render_parallel(scene: &Scene, camera: &Camera, buffer: &mut [u32], time: f32, rotation_y: f32) {
    let chunks: Vec<_> = buffer.chunks_mut(WIDTH).collect();
    
    chunks.into_par_iter().enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            let ray = camera.get_ray(x as f32, y as f32, WIDTH, HEIGHT);
            let color = trace_ray(&ray, scene, 0, time, rotation_y);
            *pixel = color_to_u32(color);
        }
    });
}

fn create_scene() -> Scene {
    let mut scene = Scene::new();
    
    // Load textures
    let grass_texture = Texture::from_file("assets/grass.jpg").unwrap_or(Texture::solid_color(Vec3::new(0.2, 0.8, 0.2)));
    let stone_texture = Texture::from_file("assets/stone.png").unwrap_or(Texture::solid_color(Vec3::new(0.5, 0.5, 0.5)));
    let wood_texture = Texture::from_file("assets/wood.jpg").unwrap_or(Texture::solid_color(Vec3::new(0.6, 0.3, 0.1)));
    let water_texture = Texture::animated_water();
    let glass_texture = Texture::solid_color(Vec3::new(0.9, 0.9, 1.0));
    
    // Materials with different properties
    let grass_material = Material {
        albedo: Vec3::new(0.7, 0.7, 0.7),
        specular: 0.1,
        transparency: 0.0,
        reflectivity: 0.1,
        refraction_index: 1.0,
        texture: Some(grass_texture),
        emissive: Vec3::zero(),
    };
    
    let stone_material = Material {
        albedo: Vec3::new(0.6, 0.6, 0.6),
        specular: 0.3,
        transparency: 0.0,
        reflectivity: 0.2,
        refraction_index: 1.0,
        texture: Some(stone_texture),
        emissive: Vec3::zero(),
    };
    
    let wood_material = Material {
        albedo: Vec3::new(0.8, 0.8, 0.8),
        specular: 0.2,
        transparency: 0.0,
        reflectivity: 0.05,
        refraction_index: 1.0,
        texture: Some(wood_texture),
        emissive: Vec3::zero(),
    };
    
    let water_material = Material {
        albedo: Vec3::new(0.3, 0.3, 0.8),
        specular: 0.8,
        transparency: 0.7,
        reflectivity: 0.3,
        refraction_index: 1.33,
        texture: Some(water_texture),
        emissive: Vec3::zero(),
    };
    
    let glass_material = Material {
        albedo: Vec3::new(0.9, 0.9, 1.0),
        specular: 0.9,
        transparency: 0.9,
        reflectivity: 0.1,
        refraction_index: 1.5,
        texture: Some(glass_texture),
        emissive: Vec3::zero(),
    };
    
    let torch_material = Material {
        albedo: Vec3::new(1.0, 0.8, 0.4),
        specular: 0.1,
        transparency: 0.0,
        reflectivity: 0.0,
        refraction_index: 1.0,
        texture: None,
        emissive: Vec3::new(1.0, 0.6, 0.2),
    };
    
    // Ground plane
    for x in -5..5 {
        for z in -5..5 {
            scene.objects.push(Box::new(Cube {
                center: Vec3::new(x as f32, -1.0, z as f32),
                size: 1.0,
                material: grass_material.clone(),
            }));
        }
    }
    
    // Stone castle walls
    for i in 0..8 {
        scene.objects.push(Box::new(Cube {
            center: Vec3::new(-4.0, i as f32, 0.0),
            size: 1.0,
            material: stone_material.clone(),
        }));
        scene.objects.push(Box::new(Cube {
            center: Vec3::new(4.0, i as f32, 0.0),
            size: 1.0,
            material: stone_material.clone(),
        }));
    }
    
    // Wooden bridge
    for i in -3..4 {
        scene.objects.push(Box::new(Cube {
            center: Vec3::new(i as f32, 0.0, 2.0),
            size: 1.0,
            material: wood_material.clone(),
        }));
    }
    
    // Water pond
    scene.objects.push(Box::new(Cube {
        center: Vec3::new(0.0, -0.5, -2.0),
        size: 3.0,
        material: water_material,
    }));
    
    // Glass tower
    for i in 0..6 {
        scene.objects.push(Box::new(Cube {
            center: Vec3::new(2.0, i as f32, -4.0),
            size: 1.0,
            material: glass_material.clone(),
        }));
    }
    
    // Torches (emissive materials)
    scene.objects.push(Box::new(Cube {
        center: Vec3::new(-2.0, 1.0, 1.0),
        size: 0.2,
        material: torch_material.clone(),
    }));
    scene.objects.push(Box::new(Cube {
        center: Vec3::new(2.0, 1.0, 1.0),
        size: 0.2,
        material: torch_material,
    }));
    
    // Lights
    scene.lights.push(Light {
        position: Vec3::new(10.0, 10.0, 10.0),
        color: Vec3::new(1.0, 1.0, 0.8),
        intensity: 1.0,
    });
    
    scene.lights.push(Light {
        position: Vec3::new(-2.0, 1.5, 1.0),
        color: Vec3::new(1.0, 0.6, 0.2),
        intensity: 2.0,
    });
    
    scene.lights.push(Light {
        position: Vec3::new(2.0, 1.5, 1.0),
        color: Vec3::new(1.0, 0.6, 0.2),
        intensity: 2.0,
    });
    
    // Skybox
    scene.skybox = Some(Skybox::gradient(
        Vec3::new(0.5, 0.8, 1.0), // Top color (day)
        Vec3::new(1.0, 0.6, 0.3)  // Horizon color
    ));
    
    scene
}

fn update_scene(scene: &mut Scene, time: f32) {
    // Day/night cycle
    let day_progress = (time * 0.1).sin() * 0.5 + 0.5;
    
    // Update sun position and intensity
    if let Some(ref mut main_light) = scene.lights.get_mut(0) {
        let sun_angle = time * 0.1;
        main_light.position = Vec3::new(
            sun_angle.cos() * 15.0,
            sun_angle.sin() * 10.0 + 5.0,
            10.0
        );
        
        // Adjust color based on time of day
        if day_progress > 0.5 {
            // Day time
            main_light.color = Vec3::new(1.0, 1.0, 0.9);
            main_light.intensity = day_progress * 2.0;
        } else {
            // Night time / sunset
            main_light.color = Vec3::new(1.0, 0.4, 0.1);
            main_light.intensity = (1.0 - day_progress) * 0.5;
        }
    }
    
    // Update skybox colors based on time of day
    if let Some(ref mut skybox) = scene.skybox {
        if day_progress > 0.5 {
            skybox.top_color = Vec3::new(0.5, 0.8, 1.0);
            skybox.horizon_color = Vec3::new(0.8, 0.9, 1.0);
        } else {
            skybox.top_color = Vec3::new(0.1, 0.1, 0.3);
            skybox.horizon_color = Vec3::new(0.3, 0.2, 0.4);
        }
    }
}

fn color_to_u32(color: Vec3) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}