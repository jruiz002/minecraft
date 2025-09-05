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
use obj_loader::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

#[derive(Clone, Copy)]
struct RenderState {
    scale_factor: usize, // 1 = full res, 2 = half res, etc.
    shadow_mode: raytracer::ShadowMode,
    max_depth: i32,
    ultra_mode: bool,
    checker_phase: bool,
}

fn main() {
    let mut window_options = WindowOptions::default();
    window_options.scale = minifb::Scale::X2;
    window_options.resize = true;
    window_options.title = true;
    window_options.borderless = false;
    
    let mut window = Window::new(
        "Minecraft Raytracing Diorama - WASD: Move, Mouse: Look, R: Rotate, Scroll: Zoom, T: Toggle Day/Night",
        WIDTH,
        HEIGHT,
        window_options,
    ).unwrap_or_else(|e| panic!("{}", e));
    let _ = window.set_position(100, 100);
    
    window.limit_update_rate(Some(std::time::Duration::from_micros(16666))); // 60 FPS

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(15.0, 10.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        45.0,
        WIDTH as f32 / HEIGHT as f32,
    );

    let mut scene = create_minecraft_scene();
    // Build BVH once after scene creation for faster ray intersections
    build_scene_bvh(&mut scene);
    let mut frame_buffer = vec![0u32; WIDTH * HEIGHT];
    let mut prev_full_buffer = vec![0u32; WIDTH * HEIGHT];
    let mut lowres_buffer: Vec<u32> = Vec::new();
    let mut prev_lowres_buffer: Vec<u32> = Vec::new();
    let mut time = 0.0f32;
    let mut fps_counter = 0;
    let mut fps_timer = Instant::now();
    let mut rotation_y = 0.0f32;
    let mut input_state = InputState::default();
    let mut manual_time_control = false;

    println!("=== Minecraft Raytracer Controls ===");
    println!("WASD/Arrow Keys: Move camera");
    println!("QE/PageUp/PageDown: Move up/down");
    println!("R: Rotate scene");
    println!("T: Toggle manual day/night control (hold J/K to scrub, H to toggle)");
    println!("1-4: Resolution scale, Y/U/I: Shadows None/SunOnly/Full, F/G: Max depth +/-");
    println!("Z: Ultra mode (checkerboard + temporal reuse)");
    println!("Mouse: Look around (drag)");
    println!("Scroll: Zoom in/out");
    println!("ESC: Exit");
    println!("====================================");

    // Faster defaults for smoother movement (adjust at runtime with keys above)
    let mut render_state = RenderState { scale_factor: 3, shadow_mode: raytracer::ShadowMode::None, max_depth: 2, ultra_mode: true, checker_phase: false };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start_time = Instant::now();
        
        let delta_time = 1.0 / 60.0;
        input_state.move_speed = 5.0 * delta_time;
        handle_input(&window, &mut camera, &mut rotation_y, &mut input_state, &mut manual_time_control);
        
        if !manual_time_control {
            time += 0.016;
        } else {
            // Manual time scrubbing
            if window.is_key_down(Key::J) { time -= 0.05; }
            if window.is_key_down(Key::K) { time += 0.05; }
            if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) {
                // Jump half cycle to toggle day/night
                let half_cycle = std::f32::consts::PI / 0.05;
                time += half_cycle;
                println!("Toggled day/night");
            }
        }
        update_minecraft_scene(&mut scene, time);
        
        let render_start = Instant::now();
        // Keyboard toggles for performance/quality
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) { render_state.scale_factor = 1; }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) { render_state.scale_factor = 2; }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) { render_state.scale_factor = 3; }
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) { render_state.scale_factor = 4; }
        if window.is_key_pressed(Key::F, minifb::KeyRepeat::No) { render_state.max_depth = (render_state.max_depth + 1).clamp(1, 6); println!("Max depth: {}", render_state.max_depth); }
        if window.is_key_pressed(Key::G, minifb::KeyRepeat::No) { render_state.max_depth = (render_state.max_depth - 1).clamp(1, 6); println!("Max depth: {}", render_state.max_depth); }
        if window.is_key_pressed(Key::Y, minifb::KeyRepeat::No) { render_state.shadow_mode = raytracer::ShadowMode::None; println!("Shadows: None"); }
        if window.is_key_pressed(Key::U, minifb::KeyRepeat::No) { render_state.shadow_mode = raytracer::ShadowMode::SunOnly; println!("Shadows: SunOnly"); }
        if window.is_key_pressed(Key::I, minifb::KeyRepeat::No) { render_state.shadow_mode = raytracer::ShadowMode::Full; println!("Shadows: Full"); }
        if window.is_key_pressed(Key::Z, minifb::KeyRepeat::No) { render_state.ultra_mode = !render_state.ultra_mode; println!("Ultra mode: {}", if render_state.ultra_mode { "ON" } else { "OFF" }); }
        
        let opts = raytracer::RenderOptions { shadow_mode: render_state.shadow_mode, max_depth: render_state.max_depth, far_simplify_distance: 20.0 };
        
        if render_state.ultra_mode {
            render_checkerboard_scaled(
                &scene,
                &camera,
                &mut frame_buffer,
                &mut prev_full_buffer,
                &mut lowres_buffer,
                &mut prev_lowres_buffer,
                time,
                rotation_y,
                render_state.scale_factor,
                opts,
                render_state.checker_phase,
            );
            render_state.checker_phase = !render_state.checker_phase;
        } else {
            render_parallel_scaled(&scene, &camera, &mut frame_buffer, &mut lowres_buffer, time, rotation_y, render_state.scale_factor, opts);
            if render_state.scale_factor <= 1 {
                prev_full_buffer.copy_from_slice(&frame_buffer);
            } else {
                prev_lowres_buffer = lowres_buffer.clone();
            }
        }
        let render_time = render_start.elapsed();
        
        window.update_with_buffer(&frame_buffer, WIDTH, HEIGHT).unwrap();
        
        fps_counter += 1;
        if fps_timer.elapsed().as_secs_f32() >= 1.0 {
            let fps = fps_counter as f32 / fps_timer.elapsed().as_secs_f32();
            println!("FPS: {:.1}, Render: {:.2}ms", fps, render_time.as_secs_f32() * 1000.0);
            fps_counter = 0;
            fps_timer = Instant::now();
        }
        
        let frame_time = start_time.elapsed();
        if frame_time.as_millis() < 16 {
            std::thread::sleep(std::time::Duration::from_millis(16 - frame_time.as_millis() as u64));
        }
    }
}

#[derive(Default)]
struct InputState {
    last_mouse_pos: Option<(f32, f32)>,
    move_speed: f32,
    rotation_speed: f32,
    zoom_speed: f32,
    mouse_sensitivity: f32,
}

impl InputState {
    fn new() -> Self {
        Self {
            last_mouse_pos: None,
            move_speed: 1.0,
            rotation_speed: 0.02,
            zoom_speed: 0.5,
            mouse_sensitivity: 0.005,
        }
    }
}

fn handle_input(
    window: &Window, 
    camera: &mut Camera, 
    rotation_y: &mut f32, 
    input_state: &mut InputState,
    manual_time: &mut bool
) {
    let move_speed = if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
        input_state.move_speed * 3.0
    } else {
        input_state.move_speed
    };
    
    // Camera movement
    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
        camera.position = camera.position + camera.get_forward() * move_speed;
    }
    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
        camera.position = camera.position - camera.get_forward() * move_speed;
    }
    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        camera.position = camera.position - camera.get_right() * move_speed;
    }
    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        camera.position = camera.position + camera.get_right() * move_speed;
    }
    
    // Vertical movement
    if window.is_key_down(Key::Q) || window.is_key_down(Key::PageDown) {
        camera.position.y -= move_speed;
    }
    if window.is_key_down(Key::E) || window.is_key_down(Key::PageUp) {
        camera.position.y += move_speed;
    }
    
    // Scene rotation
    if window.is_key_down(Key::R) {
        *rotation_y += input_state.rotation_speed * 5.0;
    }
    
    // Toggle manual time control
    if window.is_key_pressed(Key::T, minifb::KeyRepeat::No) {
        *manual_time = !*manual_time;
        println!("Manual time control: {}", if *manual_time { "ON" } else { "OFF" });
    }
    
    // Mouse controls
    if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
        if let Some((last_x, last_y)) = input_state.last_mouse_pos {
            let dx = x - last_x;
            let dy = y - last_y;
            
            // Mouse drag to look around
            if window.get_mouse_down(minifb::MouseButton::Left) {
                let yaw = dx * input_state.mouse_sensitivity;
                let pitch = dy * input_state.mouse_sensitivity;
                
                // Update camera rotation
                let forward = camera.get_forward();
                let _right = camera.get_right();
                
                // Yaw rotation (around Y axis)
                let new_forward = Vec3::new(
                    forward.x * yaw.cos() - forward.z * yaw.sin(),
                    forward.y,
                    forward.x * yaw.sin() + forward.z * yaw.cos()
                );
                
                // Pitch rotation (around right vector)
                let pitch_rotation = Vec3::new(0.0, -pitch, 0.0);
                let _adjusted_target = camera.target + pitch_rotation;
                
                camera.target = camera.position + new_forward * (camera.target - camera.position).length();
            }
            
            // Right click for scene rotation
            if window.get_mouse_down(minifb::MouseButton::Right) {
                *rotation_y += dx * input_state.rotation_speed;
            }
        }
        
        // Mouse wheel zoom
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            let zoom_factor = scroll_y as f32 * input_state.zoom_speed;
            camera.position = camera.position + camera.get_forward() * zoom_factor;
        }
        
        input_state.last_mouse_pos = Some((x, y));
    } else {
        input_state.last_mouse_pos = None;
    }
}

fn render_parallel(scene: &Scene, camera: &Camera, buffer: &mut [u32], time: f32, rotation_y: f32, opts: raytracer::RenderOptions) {
    let frame = camera.build_frame(WIDTH, HEIGHT);
    let chunks: Vec<_> = buffer.chunks_mut(WIDTH).collect();
    chunks.into_par_iter().enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            let ray = frame.get_ray(x as f32, y as f32);
            let color = trace_ray(&ray, scene, 0, time, rotation_y, &opts);
            *pixel = color_to_u32(color);
        }
    });
}

fn render_parallel_scaled(scene: &Scene, camera: &Camera, full_buffer: &mut [u32], lowres_buffer: &mut Vec<u32>, time: f32, rotation_y: f32, scale_factor: usize, opts: raytracer::RenderOptions) {
    if scale_factor <= 1 {
        render_parallel(scene, camera, full_buffer, time, rotation_y, opts);
        return;
    }
    let lw = WIDTH / scale_factor;
    let lh = HEIGHT / scale_factor;
    if lowres_buffer.len() != lw * lh { lowres_buffer.resize(lw * lh, 0); }
    let frame = camera.build_frame(lw, lh);
    let chunks: Vec<_> = lowres_buffer.chunks_mut(lw).collect();
    chunks.into_par_iter().enumerate().for_each(|(y, row)| {
        for (x, pixel) in row.iter_mut().enumerate() {
            let ray = frame.get_ray(x as f32, y as f32);
            let color = trace_ray(&ray, scene, 0, time, rotation_y, &opts);
            *pixel = color_to_u32(color);
        }
    });
    // Upscale nearest-neighbor
    for y in 0..HEIGHT {
        let mut src_y = y / scale_factor;
        if src_y >= lh { src_y = lh - 1; }
        let dest_row = &mut full_buffer[y * WIDTH..(y + 1) * WIDTH];
        for x in 0..WIDTH {
            let mut src_x = x / scale_factor;
            if src_x >= lw { src_x = lw - 1; }
            dest_row[x] = lowres_buffer[src_y * lw + src_x];
        }
    }
}

fn render_checkerboard_scaled(
    scene: &Scene,
    camera: &Camera,
    full_buffer: &mut [u32],
    prev_full_buffer: &mut [u32],
    lowres_buffer: &mut Vec<u32>,
    prev_lowres_buffer: &mut Vec<u32>,
    time: f32,
    rotation_y: f32,
    scale_factor: usize,
    opts: raytracer::RenderOptions,
    phase: bool,
) {
    if scale_factor <= 1 {
        // Full-res checkerboard: render every other pixel, reuse previous frame for the rest
        let frame = camera.build_frame(WIDTH, HEIGHT);
        full_buffer.par_chunks_mut(WIDTH).enumerate().for_each(|(y, row)| {
            for x in 0..WIDTH {
                let pattern = ((x + y) & 1) == 0;
                if pattern == phase {
                    let ray = frame.get_ray(x as f32, y as f32);
                    let color = trace_ray(&ray, scene, 0, time, rotation_y, &opts);
                    row[x] = color_to_u32(color);
                } else {
                    row[x] = prev_full_buffer[y * WIDTH + x];
                }
            }
        });
        prev_full_buffer.copy_from_slice(&full_buffer);
        return;
    }
    // Low-res checkerboard
    let lw = WIDTH / scale_factor;
    let lh = HEIGHT / scale_factor;
    if lowres_buffer.len() != lw * lh { lowres_buffer.resize(lw * lh, 0); }
    if prev_lowres_buffer.len() != lw * lh { prev_lowres_buffer.resize(lw * lh, 0); }
    let frame = camera.build_frame(lw, lh);
    lowres_buffer.par_chunks_mut(lw).enumerate().for_each(|(y, row)| {
        for x in 0..lw {
            let pattern = ((x + y) & 1) == 0;
            if pattern == phase {
                let ray = frame.get_ray(x as f32, y as f32);
                let color = trace_ray(&ray, scene, 0, time, rotation_y, &opts);
                row[x] = color_to_u32(color);
            } else {
                row[x] = prev_lowres_buffer[y * lw + x];
            }
        }
    });
    // Upscale to full buffer
    for y in 0..HEIGHT {
        let mut src_y = y / scale_factor;
        if src_y >= lh { src_y = lh - 1; }
        let dest_row = &mut full_buffer[y * WIDTH..(y + 1) * WIDTH];
        for x in 0..WIDTH {
            let mut src_x = x / scale_factor;
            if src_x >= lw { src_x = lw - 1; }
            dest_row[x] = lowres_buffer[src_y * lw + src_x];
        }
    }
    *prev_lowres_buffer = lowres_buffer.clone();
}

// (Removed duplicate alternate version)

fn create_minecraft_scene() -> Scene {
    let mut scene = Scene::new();
    
    // Create Minecraft block materials
    let materials = create_minecraft_materials();
    
    // Create terrain base
    create_terrain(&mut scene, &materials);
    
    // Create structures
    create_house(&mut scene, &materials);
    create_tower(&mut scene, &materials);
    create_water_features(&mut scene, &materials);
    create_nether_portal(&mut scene, &materials);
    create_campfire(&mut scene, &materials);
    
    // Load 3D models
    load_3d_models(&mut scene, &materials);
    
    // Add lights
    setup_lighting(&mut scene);
    
    // Create skybox
    scene.skybox = Some(create_minecraft_skybox());
    
    scene
}

fn create_minecraft_materials() -> MinecraftMaterials {
    MinecraftMaterials::new()
}

struct MinecraftMaterials {
    pub grass: Material,
    pub stone: Material,
    pub wood: Material,
    pub water: Material,
    pub glass: Material,
    pub diamond: Material,
    pub obsidian: Material,
    pub glowstone: Material,
    pub portal: Material,
    pub campfire: Material,
}

impl MinecraftMaterials {
    fn new() -> Self {
        Self {
            grass: Material::new()
                .with_texture(Texture::minecraft_grass())
                .with_properties(Vec3::new(0.4, 0.8, 0.2), 0.1, 0.0, 0.05),
            
            stone: Material::new()
                .with_texture(Texture::minecraft_stone())
                .with_properties(Vec3::new(0.6, 0.6, 0.6), 0.2, 0.0, 0.1),
            
            wood: Material::new()
                .with_texture(Texture::minecraft_wood())
                .with_properties(Vec3::new(0.8, 0.5, 0.3), 0.1, 0.0, 0.05),
            
            water: Material::new()
                .with_texture(Texture::animated_water())
                .with_properties(Vec3::new(0.2, 0.4, 0.8), 0.9, 0.8, 0.3)
                .with_refraction(1.33),
            
            glass: Material::new()
                .with_texture(Texture::solid_color(Vec3::new(0.9, 0.9, 1.0)))
                .with_properties(Vec3::new(0.9, 0.9, 1.0), 0.9, 0.9, 0.1)
                .with_refraction(1.5),
            
            diamond: Material::new()
                .with_texture(Texture::minecraft_diamond())
                .with_properties(Vec3::new(0.7, 0.9, 1.0), 0.95, 0.2, 0.8)
                .with_refraction(2.4),
            
            obsidian: Material::new()
                .with_texture(Texture::minecraft_obsidian())
                .with_properties(Vec3::new(0.1, 0.05, 0.2), 0.3, 0.0, 0.6),
            
            glowstone: Material::emissive(Vec3::new(1.0, 0.8, 0.4), 2.0)
                .with_texture(Texture::minecraft_glowstone()),
            
            portal: Material::new()
                .with_texture(Texture::nether_portal())
                .with_properties(Vec3::new(0.5, 0.1, 0.8), 0.1, 0.9, 0.3)
                .with_emissive(Vec3::new(0.3, 0.1, 0.5)),
            
            campfire: Material::emissive(Vec3::new(1.0, 0.4, 0.1), 3.0)
                .with_texture(Texture::animated_fire()),
        }
    }
}

fn create_terrain(scene: &mut Scene, materials: &MinecraftMaterials) {
    // Create layered terrain
    for x in -15..15 {
        for z in -15..15 {
            // Grass layer
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, -1.0, z as f32),
                1.0,
                materials.grass.clone(),
            )));
            
            // Stone layers below
            for y in -4..-1 {
                if (x + z + y) % 3 != 0 { // Varied stone placement
                    scene.objects.push(Box::new(Cube::new(
                        Vec3::new(x as f32, y as f32, z as f32),
                        1.0,
                        materials.stone.clone(),
                    )));
                }
            }
        }
    }
}

fn create_house(scene: &mut Scene, materials: &MinecraftMaterials) {
    // Wooden house foundation
    for x in -2..3 {
        for z in -2..3 {
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, 0.0, z as f32),
                1.0,
                materials.wood.clone(),
            )));
        }
    }
    
    // Walls
    for y in 1..4 {
        // Front and back walls
        for x in -2..3 {
            if y == 2 && x == 0 { continue; } // Door
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, y as f32, -2.0),
                1.0,
                materials.wood.clone(),
            )));
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, y as f32, 2.0),
                1.0,
                materials.wood.clone(),
            )));
        }
        
        // Side walls
        for z in -1..2 {
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(-2.0, y as f32, z as f32),
                1.0,
                materials.wood.clone(),
            )));
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(2.0, y as f32, z as f32),
                1.0,
                materials.wood.clone(),
            )));
        }
    }
    
    // Glass windows
    scene.objects.push(Box::new(Cube::new(
        Vec3::new(-2.0, 2.0, 0.0),
        1.0,
        materials.glass.clone(),
    )));
    scene.objects.push(Box::new(Cube::new(
        Vec3::new(2.0, 2.0, 0.0),
        1.0,
        materials.glass.clone(),
    )));
    
    // Roof
    for x in -3..4 {
        for z in -3..4 {
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, 4.0, z as f32),
                1.0,
                materials.stone.clone(),
            )));
        }
    }
}

fn create_tower(scene: &mut Scene, materials: &MinecraftMaterials) {
    let tower_x = 8.0;
    let tower_z = 8.0;
    
    // Stone tower
    for y in 0..12 {
        for angle in 0..8 {
            let angle_rad = (angle as f32) * std::f32::consts::PI / 4.0;
            let x = tower_x + angle_rad.cos() * 2.0;
            let z = tower_z + angle_rad.sin() * 2.0;
            
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x, y as f32, z),
                1.0,
                materials.stone.clone(),
            )));
        }
    }
    
    // Glowstone at the top
    scene.objects.push(Box::new(Cube::new(
        Vec3::new(tower_x, 12.0, tower_z),
        1.0,
        materials.glowstone.clone(),
    )));
    
    // Diamond block as decoration
    scene.objects.push(Box::new(Cube::new(
        Vec3::new(tower_x, 6.0, tower_z),
        1.0,
        materials.diamond.clone(),
    )));
}

fn create_water_features(scene: &mut Scene, materials: &MinecraftMaterials) {
    // Water pond
    for x in 5..9 {
        for z in -3..1 {
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, -0.8, z as f32),
                1.0,
                materials.water.clone(),
            )));
        }
    }
    
    // Stone around water
    for x in 4..10 {
        for z in -4..2 {
            if x >= 5 && x < 9 && z >= -3 && z < 1 { continue; }
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(x as f32, -1.0, z as f32),
                1.0,
                materials.stone.clone(),
            )));
        }
    }
}

fn create_nether_portal(scene: &mut Scene, materials: &MinecraftMaterials) {
    let portal_x = -8.0;
    let portal_z = 0.0;
    
    // Obsidian frame
    for y in 0..5 {
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(portal_x - 1.0, y as f32, portal_z),
            1.0,
            materials.obsidian.clone(),
        )));
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(portal_x + 2.0, y as f32, portal_z),
            1.0,
            materials.obsidian.clone(),
        )));
    }
    
    for x in 0..2 {
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(portal_x + x as f32, -1.0, portal_z),
            1.0,
            materials.obsidian.clone(),
        )));
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(portal_x + x as f32, 4.0, portal_z),
            1.0,
            materials.obsidian.clone(),
        )));
    }
    
    // Portal effect inside
    for y in 0..4 {
        for x in 0..2 {
            scene.objects.push(Box::new(Cube::new(
                Vec3::new(portal_x + x as f32, y as f32, portal_z),
                1.0,
                materials.portal.clone(),
            )));
        }
    }
}

fn create_campfire(scene: &mut Scene, materials: &MinecraftMaterials) {
    // Campfire in the center
    scene.objects.push(Box::new(Cube::new(
        Vec3::new(0.0, 0.2, 6.0),
        0.8,
        materials.campfire.clone(),
    )));
    
    // Stone circle around campfire
    for angle in 0..8 {
        let angle_rad = (angle as f32) * std::f32::consts::PI / 4.0;
        let x = angle_rad.cos() * 2.0;
        let z = 6.0 + angle_rad.sin() * 2.0;
        
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(x, 0.0, z),
            1.0,
            materials.stone.clone(),
        )));
    }
}

fn load_3d_models(scene: &mut Scene, materials: &MinecraftMaterials) {
    // Ensure there is at least one OBJ in assets; if missing, write a tiny model
    ensure_obj_asset();
    // Try to load a model (fallback to procedural if file doesn't exist)
    if let Ok(model) = ObjModel::load_from_file("assets/tree.obj", materials.wood.clone()) {
        for triangle in model.triangles {
            scene.objects.push(triangle);
        }
    } else {
        // Create a procedural tree
        create_procedural_tree(scene, materials);
    }
}

fn ensure_obj_asset() {
    use std::fs;
    use std::io::Write;
    let _ = fs::create_dir_all("assets");
    let path = "assets/tree.obj";
    if fs::metadata(path).is_ok() { return; }
    let obj = "# simple low-poly tree\n\
v 0 0 0\n\
v 1 0 0\n\
v 1 0 1\n\
v 0 0 1\n\
v 0.5 1.5 0.5\n\
v 0.5 2.3 0.5\n\
v -0.3 1.8 0.5\n\
v 1.3 1.8 0.5\n\
v 0.5 1.8 -0.3\n\
v 0.5 1.8 1.3\n\
f 1 2 3\n\
f 1 3 4\n\
f 1 2 5\n\
f 2 3 5\n\
f 3 4 5\n\
f 4 1 5\n\
f 6 7 8\n\
f 6 8 9\n\
f 6 9 10\n\
f 6 10 7\n";
    if let Ok(mut f) = fs::File::create(path) {
        let _ = f.write_all(obj.as_bytes());
    }
}

fn create_procedural_tree(scene: &mut Scene, materials: &MinecraftMaterials) {
    let tree_x = -5.0;
    let tree_z = -8.0;
    
    // Tree trunk
    for y in 0..6 {
        scene.objects.push(Box::new(Cube::new(
            Vec3::new(tree_x, y as f32, tree_z),
            1.0,
            materials.wood.clone(),
        )));
    }
    
    // Tree leaves
    for x in -2i32..3i32 {
        for z in -2i32..3i32 {
            for y in 5..8 {
                if (x.abs() + z.abs()) <= 2 {
                    scene.objects.push(Box::new(Cube::new(
                        Vec3::new(tree_x + x as f32, y as f32, tree_z + z as f32),
                        1.0,
                        materials.grass.clone(), // Using grass texture for leaves
                    )));
                }
            }
        }
    }
}

fn setup_lighting(scene: &mut Scene) {
    // Sun (main directional light)
    scene.lights.push(Light {
        position: Vec3::new(50.0, 50.0, 30.0),
        color: Vec3::new(1.0, 1.0, 0.9),
        intensity: 1.5,
        light_type: LightType::Directional(Vec3::new(-1.0, -1.0, -0.5).normalize()),
    });
    
    // Campfire light
    scene.lights.push(Light {
        position: Vec3::new(0.0, 1.5, 6.0),
        color: Vec3::new(1.0, 0.5, 0.2),
        intensity: 3.0,
        light_type: LightType::Point,
    });
    
    // Glowstone light
    scene.lights.push(Light {
        position: Vec3::new(8.0, 13.0, 8.0),
        color: Vec3::new(1.0, 0.8, 0.4),
        intensity: 2.5,
        light_type: LightType::Point,
    });
    
    // Portal light
    scene.lights.push(Light {
        position: Vec3::new(-7.0, 2.0, 0.0),
        color: Vec3::new(0.5, 0.1, 0.8),
        intensity: 2.0,
        light_type: LightType::Point,
    });
}

fn create_minecraft_skybox() -> Skybox {
    Skybox::textured(
        Vec3::new(0.5, 0.8, 1.0), // Day sky blue
        Vec3::new(1.0, 0.6, 0.3), // Sunset orange
        Vec3::new(0.1, 0.1, 0.3), // Night dark blue
        Vec3::new(0.3, 0.2, 0.4)  // Night horizon
    )
}

fn update_minecraft_scene(scene: &mut Scene, time: f32) {
    // Day/night cycle
    let day_progress = (time * 0.05).sin() * 0.5 + 0.5; // Slower cycle
    
    // Update sun
    if let Some(main_light) = scene.lights.get_mut(0) {
        let sun_angle = time * 0.05;
        // Update directional light to follow the sun path
        let sun_dir = Vec3::new(0.3, sun_angle.sin(), sun_angle.cos()).normalize();
        main_light.light_type = LightType::Directional(sun_dir);
        
        if day_progress > 0.3 {
            // Day
            main_light.color = Vec3::new(1.0, 1.0, 0.9);
            main_light.intensity = day_progress * 2.0;
        } else {
            // Night/Dawn/Dusk
            main_light.color = Vec3::new(1.0, 0.6, 0.3);
            main_light.intensity = (day_progress + 0.2) * 0.8;
        }
    }
    
    // Update skybox
    if let Some(skybox) = &mut scene.skybox {
        skybox.update_time_of_day(day_progress);
    }
}

fn color_to_u32(color: Vec3) -> u32 {
    // Gamma correction for better visual quality
    let gamma = 1.0 / 2.2;
    let r = (color.x.powf(gamma).clamp(0.0, 1.0) * 255.0) as u32;
    let g = (color.y.powf(gamma).clamp(0.0, 1.0) * 255.0) as u32;
    let b = (color.z.powf(gamma).clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}