use crate::color::Color;

pub const SCREEN_WIDTH: usize = 800;
pub const SCREEN_HEIGHT: usize = 600;

pub struct Framebuffer {
   pub buffer: Vec<Color>,
   pub z_buffer: Vec<f32>,
   pub width: usize,
   pub height: usize,
}

impl Framebuffer {
   pub fn new() -> Self {
      let buffer = vec![Color::new(0, 0, 0); SCREEN_WIDTH * SCREEN_HEIGHT];
      let z_buffer = vec![f32::INFINITY; SCREEN_WIDTH * SCREEN_HEIGHT];
      Framebuffer {
         buffer,
         z_buffer,
         width: SCREEN_WIDTH,
         height: SCREEN_HEIGHT,
      }
   }

   pub fn clear(&mut self, color: Color) {
      for pixel in &mut self.buffer {
         *pixel = color;
      }

      for z in &mut self.z_buffer {
      *z = f32::MAX;
      }
   }

   pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
      if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
         let index = y as usize * self.width + x as usize;
         self.buffer[index] = color;
      }
   }

   pub fn render(&self, renderer: &mut sdl2::render::Canvas<sdl2::video::Window>) {
      let creator = renderer.texture_creator();
      let mut texture = creator
         .create_texture_streaming(
               sdl2::pixels::PixelFormatEnum::RGB24,
               self.width as u32,
               self.height as u32,
         )
         .unwrap();

      // Actualizar la textura con nuestro framebuffer
      texture
         .with_lock(None, |buffer: &mut [u8], pitch: usize| {
               for y in 0..self.height {
                  for x in 0..self.width {
                     let offset = y * pitch + x * 3;
                     let index = y * self.width + x;
                     buffer[offset] = self.buffer[index].r;
                     buffer[offset + 1] = self.buffer[index].g;
                     buffer[offset + 2] = self.buffer[index].b;
                  }
               }
         })
         .unwrap();

      renderer.copy(&texture, None, None).unwrap();
      renderer.present();
   }
}