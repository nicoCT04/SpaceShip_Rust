use crate::framebuffer::Framebuffer;
use crate::color::Color;
use glm::Vec3;

pub fn draw_triangle(
   framebuffer: &mut Framebuffer,
   v0: &Vec3,
   v1: &Vec3,
   v2: &Vec3,
   color: Color,
) {
   // Convertir vértices a coordenadas de pantalla (coordenadas enteras)
   let x0 = v0.x as i32;
   let y0 = v0.y as i32;
   let z0 = v0.z;
   
   let x1 = v1.x as i32;
   let y1 = v1.y as i32;
   let z1 = v1.z;
   
   let x2 = v2.x as i32;
   let y2 = v2.y as i32;
   let z2 = v2.z;
   
   // Ordenar vértices por coordenada Y (de arriba a abajo)
   let mut vertices = [(x0, y0, z0), (x1, y1, z1), (x2, y2, z2)];
   if vertices[0].1 > vertices[1].1 {
      vertices.swap(0, 1);
   }
   if vertices[0].1 > vertices[2].1 {
      vertices.swap(0, 2);
   }
   if vertices[1].1 > vertices[2].1 {
      vertices.swap(1, 2);
   }
   
   // Desempaquetar vértices ordenados
   let (x0, y0, z0) = vertices[0];
   let (x1, y1, z1) = vertices[1];
   let (x2, y2, z2) = vertices[2];
   
   // Calcular pendientes inversas para interpolación
   let total_height = y2 - y0;
   
   if total_height == 0 {
      return;
   }
   
   // Primera mitad del triángulo (parte superior)
   let segment_height = y1 - y0;
   for y in y0..=y1 {
      if segment_height == 0 {
         continue;
      }
      
      let alpha = (y - y0) as f32 / total_height as f32;
      let beta = (y - y0) as f32 / segment_height as f32;
      
      let x_a = x0 as f32 + (x2 - x0) as f32 * alpha;
      let x_b = x0 as f32 + (x1 - x0) as f32 * beta;
      
      // Interpolación de z
      let z_a = z0 + (z2 - z0) * alpha;
      let z_b = z0 + (z1 - z0) * beta;
      
      let start_x = x_a.min(x_b) as i32;
      let end_x = x_a.max(x_b) as i32;
      
      for x in start_x..=end_x {
         // Interpolar z para este pixel
         let t = if end_x == start_x { 0.0 } else { (x - start_x) as f32 / (end_x - start_x) as f32 };
         let z = z_a * (1.0 - t) + z_b * t;
         
         // Verificar si este pixel está más cerca que lo que ya está en el z-buffer
         if x >= 0 && x < framebuffer.width as i32 && y >= 0 && y < framebuffer.height as i32 {
               let index = y as usize * framebuffer.width + x as usize;
               if z < framebuffer.z_buffer[index] {
                  framebuffer.buffer[index] = color;
                  framebuffer.z_buffer[index] = z;
               }
         }
      }
   }
   
   // Segunda mitad del triángulo (parte inferior)
   let segment_height = y2 - y1;
   for y in y1..=y2 {
      if segment_height == 0 {
         continue;
      }
      
      let alpha = (y - y0) as f32 / total_height as f32;
      let beta = (y - y1) as f32 / segment_height as f32;
      
      let x_a = x0 as f32 + (x2 - x0) as f32 * alpha;
      let x_b = x1 as f32 + (x2 - x1) as f32 * beta;
      
      // Interpolación de z
      let z_a = z0 + (z2 - z0) * alpha;
      let z_b = z1 + (z2 - z1) * beta;
      
      let start_x = x_a.min(x_b) as i32;
      let end_x = x_a.max(x_b) as i32;
      
      for x in start_x..=end_x {
         // Interpolar z para este pixel
         let t = if end_x == start_x { 0.0 } else { (x - start_x) as f32 / (end_x - start_x) as f32 };
         let z = z_a * (1.0 - t) + z_b * t;
         
         // Verificar si este pixel está más cerca que lo que ya está en el z-buffer
         if x >= 0 && x < framebuffer.width as i32 && y >= 0 && y < framebuffer.height as i32 {
               let index = y as usize * framebuffer.width + x as usize;
               if z < framebuffer.z_buffer[index] {
                  framebuffer.buffer[index] = color;
                  framebuffer.z_buffer[index] = z;
               }
         }
      }
   }
}