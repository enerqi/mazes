
// Do I really want sdl2? It can render an image to the screen but what about
// to a file?

use std::path::Path;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};
use sdl2_image::{self, INIT_JPG, INIT_PNG, LoadSurface, LoadTexture};
use sdl2_ttf;
use sdl2_ttf::Font;

pub struct SdlSetup {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub timer_subsystem: sdl2::TimerSubsystem,
    pub ttf_context: sdl2_ttf::Sdl2TtfContext,
}

pub fn init() -> SdlSetup {

    let sdl_context: sdl2::Sdl = sdl2::init().unwrap();
    let video_subsystem: sdl2::VideoSubsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let ttf_context = sdl2_ttf::init().ok().expect("Failed to init true type fonts");
    sdl2_image::init(INIT_PNG | INIT_JPG).unwrap();

    SdlSetup {
        sdl_context: sdl_context,
        video_subsystem: video_subsystem,
        timer_subsystem: timer_subsystem,
        ttf_context: ttf_context,
    }
}
