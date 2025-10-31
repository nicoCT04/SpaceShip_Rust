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
        angle_y: std::f32::consts::PI,       // Rotación inicial en Y (180 grados)
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
    let center = app_state.center;
    let scale_factor = app_state.scale_factor;
    let angle_x = app_state.angle_x;
    let angle_y = app_state.angle_y;
    let offset_x = app_state.offset_x;
    let offset_y = app_state.offset_y;
    
    // Limpiar el z-buffer con valores grandes
    for z in &mut app_state.framebuffer.z_buffer {
        *z = f32::MAX; // Usar MAX en lugar de INFINITY para evitar problemas numéricos
    }
    
    // Crear una lista de caras para ordenamiento
    let mut faces_to_render = Vec::new();
    
    // Colores definidos para cada parte de la nave
    let color_cuerpo = Color::new(250, 250, 250);      // Blanco (cuerpo principal)
    let color_propulsores = Color::new(100, 100, 100); // Gris (propulsores)
    let color_compartimentos = Color::new(255, 120, 50); // Naranja (compartimentos laterales)
    let color_cabina = Color::new(30, 30, 30);         // Negro (cabina)
    
    // Procesar todas las caras
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
        
        // Rotar los vértices para visualización
        let rv0 = rotate_y(&rotate_x(v0));
        let rv1 = rotate_y(&rotate_x(v1));
        let rv2 = rotate_y(&rotate_x(v2));
        
        // Calcular la normal de la cara para determinar visibilidad
        let edge1 = Vec3::new(rv1.x - rv0.x, rv1.y - rv0.y, rv1.z - rv0.z);
        let edge2 = Vec3::new(rv2.x - rv0.x, rv2.y - rv0.y, rv2.z - rv0.z);
        let normal = Vec3::new(
            edge1.y * edge2.z - edge1.z * edge2.y,
            edge1.z * edge2.x - edge1.x * edge2.z,
            edge1.x * edge2.y - edge1.y * edge2.x
        );
        
        // Solo procesar caras que miran hacia la cámara (backface culling)
        if normal.z <= 0.0 {
            continue;
        }
        
        // Calcular promedios para determinar la región de la nave
        let orig_avg_x = (v0.x + v1.x + v2.x) / 3.0;
        let orig_avg_y = (v0.y + v1.y + v2.y) / 3.0;
        let orig_avg_z = (v0.z + v1.z + v2.z) / 3.0;

        // Asignar colores basados en la posición original
        let final_color =
            // Cabina (parte superior central, Y alto y Z frontal)
            if orig_avg_y > 0.25 && abs(orig_avg_x) < 0.25 && orig_avg_z > 0.4 {
                color_cabina
            }
            // Propulsores (parte trasera, Z muy negativo)
            else if orig_avg_z < -0.8 {
                color_propulsores
            }
            // Compartimentos laterales / cápsulas (a los lados, X extremos)
            else if abs(orig_avg_x) > 0.7 {
                color_compartimentos
            }
            // Cuerpo principal (todo lo demás)
            else {
                color_cuerpo
            };
        
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
        
        // Calcular la profundidad promedio para ordenamiento
        let avg_z = (rv0.z + rv1.z + rv2.z) / 3.0;
        
        // Almacenar solo caras visibles para renderizado
        faces_to_render.push((avg_z, tv0, tv1, tv2, final_color));
    }
    
    // Ordenar las caras de atrás hacia adelante (mayor Z a menor Z)
    faces_to_render.sort_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Dibujar las caras en orden
    for (_, tv0, tv1, tv2, color) in faces_to_render {
        draw_triangle(&mut app_state.framebuffer, &tv0, &tv1, &tv2, color);
    }
    
    // Dibujar contornos después de todas las caras
    draw_wireframe(app_state);
}

// Función para dibujar los contornos
fn draw_wireframe(app_state: &mut AppState) {
    let model = &app_state.model;
    let center = app_state.center;
    let scale_factor = app_state.scale_factor;
    let angle_x = app_state.angle_x;
    let angle_y = app_state.angle_y;
    let offset_x = app_state.offset_x;
    let offset_y = app_state.offset_y;
    
    // Color para los contornos
    let line_color = Color::new(0, 0, 0);  // Negro
    
    for face in &model.faces {
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
        
        // Calcular la normal para determinar visibilidad
        let edge1 = Vec3::new(rv1.x - rv0.x, rv1.y - rv0.y, rv1.z - rv0.z);
        let edge2 = Vec3::new(rv2.x - rv0.x, rv2.y - rv0.y, rv2.z - rv0.z);
        let normal = Vec3::new(
            edge1.y * edge2.z - edge1.z * edge2.y,
            edge1.z * edge2.x - edge1.x * edge2.z,
            edge1.x * edge2.y - edge1.y * edge2.x
        );
        
        // Solo procesar caras que miran hacia la cámara
        if normal.z <= 0.0 {
            continue;
        }
        
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
        
        // Dibujar las líneas de contorno para caras visibles
        draw_line(&mut app_state.framebuffer, 
                tv0.x as i32, tv0.y as i32, 
                tv1.x as i32, tv1.y as i32, 
                line_color);
        
        draw_line(&mut app_state.framebuffer, 
                tv1.x as i32, tv1.y as i32, 
                tv2.x as i32, tv2.y as i32, 
                line_color);
        
        draw_line(&mut app_state.framebuffer, 
                tv2.x as i32, tv2.y as i32, 
                tv0.x as i32, tv0.y as i32, 
                line_color);
    }
}


// Función auxiliar para valor absoluto
fn abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

// Función para dibujar líneas (algoritmo de Bresenham)
fn draw_line(framebuffer: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let mut x = x0;
    let mut y = y0;
    
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    
    let mut err = dx + dy;
    
    loop {
        if x >= 0 && x < framebuffer.width as i32 && y >= 0 && y < framebuffer.height as i32 {
            let index = y as usize * framebuffer.width + x as usize;
            framebuffer.buffer[index] = color;
        }
        
        if x == x1 && y == y1 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 {
                break;
            }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            err += dx;
            y += sy;
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
            // Resetear rotación a la posición inicial en lugar de a cero
            app_state.angle_x = std::f32::consts::PI / 2.0;
            app_state.angle_y = std::f32::consts::PI;
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