extern crate sdl2;
extern crate glm;

mod color;
mod framebuffer;
mod triangle;
mod obj_loader;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use color::Color;
use framebuffer::{Framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT};
use triangle::draw_triangle;
use obj_loader::Model;
use glm::Vec3;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem.window("Software Renderer", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut framebuffer = Framebuffer::new();
    
    // Cargar el modelo .obj
    let model = Model::load_obj(Path::new("models/NavePrototipo2.obj"))
        .map_err(|e| e.to_string())?;
    
    println!("Modelo cargado con éxito:");
    println!("  - Vértices: {}", model.vertices.len());
    println!("  - Caras: {}", model.faces.len());
    
    let center = model.calculate_center();
    let size = model.calculate_size();
    
    // Calcular el factor de escala para que el modelo ocupe aproximadamente el 80% de la pantalla
    let scale_factor = (SCREEN_WIDTH.min(SCREEN_HEIGHT) as f32 * 0.8) / size.x.max(size.y).max(size.z);
    
    // Función de renderizado
    let render = |framebuffer: &mut Framebuffer, base_color: Color| {
        for (i, face) in model.faces.iter().enumerate() {
            // Obtener los tres vértices de la cara
            let v0 = &model.vertices[face[0]];
            let v1 = &model.vertices[face[1]];
            let v2 = &model.vertices[face[2]];
            
            // Transformar los vértices (centrar y escalar)
            let tv0 = Vec3::new(
                (v0.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5,
                (v0.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5,
                v0.z
            );
            
            let tv1 = Vec3::new(
                (v1.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5,
                (v1.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5,
                v1.z
            );
            
            let tv2 = Vec3::new(
                (v2.x - center.x) * scale_factor + SCREEN_WIDTH as f32 * 0.5,
                (v2.y - center.y) * scale_factor + SCREEN_HEIGHT as f32 * 0.5,
                v2.z
            );
            
            // Calcular la normal de la cara para usarla en la asignación de color
            let edge1 = Vec3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
            let edge2 = Vec3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
            
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
                Vec3::new(0.0, 1.0, 0.0)  // Valor por defecto si la normal es cero
            };
            
            // Calcular color basado en la normal y el índice
            let r = ((normal.x.abs() * 0.8 + 0.2) * base_color.r as f32) as u8;
            let g = ((normal.y.abs() * 0.8 + 0.2) * base_color.g as f32) as u8;
            let b = ((normal.z.abs() * 0.8 + 0.2) * base_color.b as f32) as u8;
            
            // Variar colores para caras alternadas
            let face_color = if i % 3 == 0 {
                Color::new(r, g, b)
            } else if i % 3 == 1 {
                Color::new(g, b, r)
            } else {
                Color::new(b, r, g)
            };
            
            // Dibujar el triángulo
            draw_triangle(framebuffer, &tv0, &tv1, &tv2, face_color);
        }
    };

    // Variables para la rotación
    let mut angle = 0.0f32;
    
    // Ciclo principal
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        
        // Limpiar el framebuffer
        framebuffer.clear(Color::new(0, 0, 0));
        
        // Renderizar el modelo
        render(&mut framebuffer, Color::new(255, 255, 0));  // Color amarillo
        
        // Actualizar la pantalla
        framebuffer.render(&mut canvas);
        
        // Incrementar el ángulo para la siguiente frame (para animación)
        angle += 0.01;
        
        // Limitar FPS
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    
    Ok(())
}