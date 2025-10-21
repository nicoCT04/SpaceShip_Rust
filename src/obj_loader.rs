use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use glm::Vec3;

pub struct Model {
   pub vertices: Vec<Vec3>,
   pub faces: Vec<[usize; 3]>,
}

impl Model {
   pub fn load_obj<P: AsRef<Path>>(path: P) -> io::Result<Self> {
      let file = File::open(path)?;
      let reader = io::BufReader::new(file);
      
      let mut vertices = Vec::new();
      let mut faces = Vec::new();
      
      for line in reader.lines() {
         let line = line?;
         let parts: Vec<&str> = line.split_whitespace().collect();
         
         if parts.is_empty() {
               continue;
         }
         
         match parts[0] {
               "v" => {
                  if parts.len() >= 4 {
                     let x = parts[1].parse::<f32>().unwrap_or(0.0);
                     let y = parts[2].parse::<f32>().unwrap_or(0.0);
                     let z = parts[3].parse::<f32>().unwrap_or(0.0);
                     vertices.push(Vec3::new(x, y, z));
                  }
               },
               "f" => {
                  if parts.len() >= 4 {
                     // Los índices en archivos .obj empiezan en 1, así que restamos 1
                     let v1 = parts[1].split('/').next().unwrap_or("1").parse::<usize>().unwrap_or(1) - 1;
                     let v2 = parts[2].split('/').next().unwrap_or("1").parse::<usize>().unwrap_or(1) - 1;
                     let v3 = parts[3].split('/').next().unwrap_or("1").parse::<usize>().unwrap_or(1) - 1;
                     
                     faces.push([v1, v2, v3]);
                  }
               },
               _ => {}
         }
      }
      
      Ok(Model { vertices, faces })
   }
   
   // Calcular el centro del modelo
   pub fn calculate_center(&self) -> Vec3 {
      let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
      let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
      
      for vertex in &self.vertices {
         min.x = min.x.min(vertex.x);
         min.y = min.y.min(vertex.y);
         min.z = min.z.min(vertex.z);
         
         max.x = max.x.max(vertex.x);
         max.y = max.y.max(vertex.y);
         max.z = max.z.max(vertex.z);
      }
      
      Vec3::new(
         (min.x + max.x) * 0.5,
         (min.y + max.y) * 0.5,
         (min.z + max.z) * 0.5
      )
   }
   
   // Calcular el tamaño del modelo para escalado
   pub fn calculate_size(&self) -> Vec3 {
      let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
      let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
      
      for vertex in &self.vertices {
         min.x = min.x.min(vertex.x);
         min.y = min.y.min(vertex.y);
         min.z = min.z.min(vertex.z);
         
         max.x = max.x.max(vertex.x);
         max.y = max.y.max(vertex.y);
         max.z = max.z.max(vertex.z);
      }
      
      Vec3::new(
         max.x - min.x,
         max.y - min.y,
         max.z - min.z
      )
   }
}