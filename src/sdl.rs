
// Do I really want sdl2? It can render an image to the screen but what about
// to a file?

use std::path::Path;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};
use sdl2_image::{self, LoadTexture, LoadSurface, INIT_PNG, INIT_JPG};
use sdl2_ttf;
use sdl2_ttf::Font;

pub struct BasicWindow {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub timer_subsystem: sdl2::TimerSubsystem,
    pub window: sdl2::video::Window,
    pub ttf_context: sdl2_ttf::Sdl2TtfContext,
}

pub fn init(window_title: &str, width: u32, height: u32) -> BasicWindow {

    let sdl_context: sdl2::Sdl = sdl2::init().unwrap();

    let video_subsystem: sdl2::VideoSubsystem = sdl_context.video().unwrap();

    let timer_subsystem = sdl_context.timer().unwrap();

    let mut window_builder: sdl2::video::WindowBuilder = video_subsystem.window(window_title,
                                                                                width,
                                                                                height);

    let window: sdl2::video::Window = window_builder.position_centered().opengl().build().unwrap();

    sdl2_image::init(INIT_PNG | INIT_JPG).unwrap();
    let ttf_context = sdl2_ttf::init().ok().expect("Failed to init true type fonts");

    BasicWindow {
        sdl_context: sdl_context,
        video_subsystem: video_subsystem,
        timer_subsystem: timer_subsystem,
        window: window,
        ttf_context: ttf_context,
    }
}
