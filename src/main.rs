extern crate sdl2;
extern crate glm;

mod color;
mod framebuffer;
mod triangle;
mod obj_loader;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::Duration;

use color::Color;
use framebuffer::{Framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT};
use triangle::draw_triangle;
use obj_loader::Model;
use glm::Vec3;

// Estructura para mantener el estado de la aplicación
struct AppState {
    framebuffer: Framebuffer,
    current_color: Color,
    model: Model,
    center: Vec3,
    scale_factor: f32,
    angle_x: f32,
    angle_y: f32,
    offset_x: f32,
    offset_y: f32,
}

fn init() -> Result<(sdl2::Sdl, sdl2::render::Canvas<sdl2::video::Window>, AppState), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem.window("Software Renderer", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    
    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    
    // Cargar el modelo .obj
    let model = Model::load_obj(Path::new("models/NavePrototipo2.obj"))
        .map_err(|e| e.to_string())?;
    
    println!("Modelo cargado con éxito:");
    println!("  - Vértices: {}", model.vertices.len());
    println!("  - Caras: {}", model.faces.len());
    
    // Calcular el centro y la escala
    let center = model.calculate_center();
    let size = model.calculate_size();
    let scale_factor = (SCREEN_WIDTH.min(SCREEN_HEIGHT) as f32 * 0.8) / size.x.max(size.y).max(size.z);
    
    // Crear el estado de la aplicación
    let app_state = AppState {
        framebuffer: Framebuffer::new(),
        current_color: Color::new(255, 255, 0),
        model,
        center,
        scale_factor,
        angle_x: std::f32::consts::PI / 2.0, // Rotación inicial en X (90 grados)
        angle_y: std::f32::consts::PI,
        offset_x: 0.0,
        offset_y: 0.0,
    };
    
    Ok((sdl_context, canvas, app_state))
}

fn clear(app_state: &mut AppState) {
    app_state.framebuffer.clear(Color::new(0, 0, 0));
}

fn set_color(app_state: &mut AppState, color: Color) {
    app_state.current_color = color;
}

fn render_buffer(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, app_state: &Framebuffer) {
    app_state.render(canvas);
}

fn render(app_state: &mut AppState) {
    let model = &app_state.model;
    let current_color = app_state.current_color;
    let center = app_state.center;
    let scale_factor = app_state.scale_factor;
    let angle_x = app_state.angle_x;
    let angle_y = app_state.angle_y;
    let offset_x = app_state.offset_x;
    let offset_y = app_state.offset_y;
    
    for (i, face) in model.faces.iter().enumerate() {
        // Obtener los tres vértices de la cara
        let v0 = &model.vertices[face[0]];
        let v1 = &model.vertices[face[1]];
        let v2 = &model.vertices[face[2]];
        
        // Aplicar rotación en X
        let rotate_x = |v: &Vec3| -> Vec3 {
            let y = v.y * angle_x.cos() - v.z * angle_x.sin();
            let z = v.y * angle_x.sin() + v.z * angle_x.cos();
            Vec3::new(v.x, y, z)
        };
        
        // Aplicar rotación en Y
        let rotate_y = |v: &Vec3| -> Vec3 {
            let x = v.x * angle_y.cos() + v.z * angle_y.sin();
            let z = -v.x * angle_y.sin() + v.z * angle_y.cos();
            Vec3::new(x, v.y, z)
        };
        
        // Rotar los vértices
        let rv0 = rotate_y(&rotate_x(v0));
        let rv1 = rotate_y(&rotate_x(v1));
        let rv2 = rotate_y(&rotate_x(v2));
        
        // Transformar a coordenadas de pantalla
        let tv0 = Vec3::new(
            (rv0.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5 + offset_x,
            (rv0.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5 + offset_y,
            rv0.z
        );
        
        let tv1 = Vec3::new(
            (rv1.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5 + offset_x,
            (rv1.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5 + offset_y,
            rv1.z
        );
        
        let tv2 = Vec3::new(
            (rv2.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5 + offset_x,
            (rv2.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5 + offset_y,
            rv2.z
        );
        
        // Calcular la normal de la cara
        let edge1 = Vec3::new(rv1.x - rv0.x, rv1.y - rv0.y, rv1.z - rv0.z);
        let edge2 = Vec3::new(rv2.x - rv0.x, rv2.y - rv0.y, rv2.z - rv0.z);
        
        // Producto cruz para obtener la normal
        let normal = Vec3::new(
            edge1.y * edge2.z - edge1.z * edge2.y,
            edge1.z * edge2.x - edge1.x * edge2.z,
            edge1.x * edge2.y - edge1.y * edge2.x
        );
        
        // Normalizar
        let length = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
        let normal = if length > 0.0001 {
            Vec3::new(normal.x / length, normal.y / length, normal.z / length)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        
        // Iluminación simple
        let light_dir = Vec3::new(0.0, 0.0, 1.0);
        let intensity = normal.x * light_dir.x + normal.y * light_dir.y + normal.z * light_dir.z;
        let intensity = intensity.max(0.2).min(1.0);
        
        // Color basado en la normal y el índice
        let base_r = ((normal.x.abs() * 0.5 + 0.5) * current_color.r as f32) as u8;
        let base_g = ((normal.y.abs() * 0.5 + 0.5) * current_color.g as f32) as u8;
        let base_b = ((normal.z.abs() * 0.5 + 0.5) * current_color.b as f32) as u8;
        
        let color_variant = i % 4;
        let face_color = match color_variant {
            0 => Color::new(180, 180, 180),  // Gris claro
            1 => Color::new(140, 140, 140),  // Gris medio
            2 => Color::new(120, 120, 130),  // Gris azulado
            _ => Color::new(160, 150, 140),  // Beige grisáceo
        };

        // Aplicar iluminación
        let shaded_color = Color::new(
            ((face_color.r as f32 * intensity) as u8),
            ((face_color.g as f32 * intensity) as u8),
            ((face_color.b as f32 * intensity) as u8)
        );
        
        // Solo dibujar si la normal apunta hacia la cámara
        if normal.z > 0.0 {
            draw_triangle(&mut app_state.framebuffer, &tv0, &tv1, &tv2, face_color);
        }
    }
}

fn handle_keys(app_state: &mut AppState, keycode: Keycode) {
    match keycode {
        Keycode::Left => app_state.angle_y -= 0.1,
        Keycode::Right => app_state.angle_y += 0.1,
        Keycode::Up => app_state.angle_x -= 0.1,
        Keycode::Down => app_state.angle_x += 0.1,
        Keycode::W => app_state.offset_y -= 10.0,
        Keycode::S => app_state.offset_y += 10.0,
        Keycode::A => app_state.offset_x -= 10.0,
        Keycode::D => app_state.offset_x += 10.0,
        Keycode::F => {
            // Girar 180 grados en Y (invertir dirección)
            app_state.angle_y += std::f32::consts::PI;
        },
        Keycode::R => {
            // Resetear rotación y posición
            app_state.angle_x = 0.0;
            app_state.angle_y = 0.0;
            app_state.offset_x = 0.0;
            app_state.offset_y = 0.0;
        },
        _ => {}
    }
}

fn main() -> Result<(), String> {
    let (sdl_context, mut canvas, mut app_state) = init()?;
    let mut event_pump = sdl_context.event_pump()?;
    
    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    handle_keys(&mut app_state, keycode);
                },
                _ => {}
            }
        }
        
        clear(&mut app_state);
        
        set_color(&mut app_state, Color::new(255, 255, 0));
        render(&mut app_state);
        
        render_buffer(&mut canvas, &app_state.framebuffer);
        
        // Limitar FPS
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    
    Ok(())
}