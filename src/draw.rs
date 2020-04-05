use sdl2::video::WindowBuilder;
use sdl2::render::{WindowCanvas, TextureQuery};
use sdl2::ttf::{Sdl2TtfContext};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::{Sdl, EventPump};
use std::path::Path;
use std::sync::{Arc, RwLock};
use crate::payload::PayloadStorage;

pub struct Context {
  canvas: WindowCanvas,
  sdl: Sdl,
  ttf_context: Box<Sdl2TtfContext>
}


static SCREEN_WIDTH : u32 = 800;
static SCREEN_HEIGHT : u32 = 480;

static FONT_SIZE : u16 = 35;

static FONT_NAME : &str = "resources/VCR_OSD_MONO.ttf";

impl Context {
  pub fn new() -> Result<Self, String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window_builder = WindowBuilder::new(
      &video_subsystem,
      "COVID-19 stats",
      SCREEN_WIDTH,
      SCREEN_HEIGHT
    );

    let window = window_builder.fullscreen()
      .position_centered()
      .opengl()
      .build()
      .map_err(|e| e.to_string())?;

    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let ttf_context = Box::new(sdl2::ttf::init().map_err(|e| e.to_string())?);
    Ok(
      Context {
        canvas,
        sdl: sdl_context,
        ttf_context
      }
    )
  }

  pub fn draw(
    &mut self,
    payload_storage: Arc<RwLock<PayloadStorage>>
  ) -> Result<(), String> {

    let readable_storage = payload_storage.read().expect("Can not read storage");
    let strings_vec = readable_storage.peek();

    let font_path: &Path = Path::new(FONT_NAME);
    let font = self.ttf_context.load_font(font_path, FONT_SIZE)?;

    self.canvas.set_draw_color(Color::RGBA(50, 50, 50, 255));
    self.canvas.clear();

    for (idx, message) in strings_vec.iter().enumerate() {
      let surface = font
        .render(message)
        .solid(Color::RGBA(200, 200, 200, 255))
        .map_err(|e| e.to_string())?;
      let texture_creator = self.canvas
        .texture_creator();
      let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

      let TextureQuery { width, height, .. } = texture.query();

      let target = Rect::new(20, (30 + (30 * idx) + (idx * FONT_SIZE as usize)) as i32 , width, height);

      self.canvas.copy(&texture, None, Some(target))?;
    }

    self.canvas.present();
    Ok(())
  }

  pub fn event_pump (&self) -> Result<EventPump, String> {
    self.sdl.event_pump()
  }
}