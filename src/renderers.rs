use std::path::Path;

use petgraph::graph::IndexType;
use sdl2;
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point};
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;
use sdl2_image;

use sdl;
use squaregrid::{GridDirection, SquareGrid};

const INITIAL_W: u32 = 1920;
const INITIAL_H: u32 = 1080;
const LOGICAL_W: u32 = 1920;
const LOGICAL_H: u32 = 1080;
const BLACK: Color = Color::RGB(0, 0, 0);
const WHITE: Color = Color::RGB(0xff, 0xff, 0xff);
const RED: Color = Color::RGB(0xff, 0, 0);
const GREEN: Color = Color::RGB(0, 0xff, 0);
const BLUE: Color = Color::RGB(0, 0, 0xff);
const YELLOW: Color = Color::RGB(0xff, 0xff, 0);


pub struct RenderOptions<'path> {
    show_on_screen: bool,
    output_file: Option<&'path Path>,
    cell_side_pixels_length: u8,
}
impl<'path> RenderOptions<'path> {
    pub fn new(show_on_screen: bool, output_file: Option<&Path>, cell_side_pixels_length: u8) -> RenderOptions {
        RenderOptions {
            show_on_screen: show_on_screen,
            output_file: output_file,
            cell_side_pixels_length: cell_side_pixels_length,
        }
    }
}


pub fn render_square_grid<GridIndexType>(grid: &SquareGrid<GridIndexType>, options: &RenderOptions)
    where GridIndexType: IndexType
{
    let sdl_setup = sdl::init();

    // We are limited to one renderer per window it seems, at least with the current rust bindings.
    // We want a hardware accelerated window for displaying a maze that uses a texture for performance,
    // but we want a software surface with the maze drawn to it so that we can use sdl2_image save_surface
    // to write out a png.
    // - Renderer::from_surface looks like what we want
    // - Note renderer.into_surface()/into_window() is not what we want
    //   Only extracts the window/surface that already used by the Renderer, basically dropping the other data
    //   that the renderer uses.
    // After rendering to the surface, we can create texture from surface and use a new 2nd renderer to
    // display to a window
    let software_surface = Surface::new(LOGICAL_W, LOGICAL_H, PixelFormatEnum::RGB888).expect("Surface creation failed.");
    let mut software_renderer = Renderer::from_surface(software_surface).expect("Software renderer creation failed.");

    // Sets a device independent resolution for rendering.
    // SDL scales to the actual window size, which may change if we allow resizing and is also
    // unknown if we just drop into fullscreen.
    software_renderer.set_logical_size(LOGICAL_W, LOGICAL_H).unwrap();

    // 0 or 'nearest' == nearest pixel sampling
    // 1 or 'linear' == linear filtering (supported by OpenGL and Direct3D)
    // 2 or 'best' == anisotropic filtering (supported by Direct3D)
    // The hint strings don't seem to be abstracted in the rust source at the moment but we can see
    // the #defines at e.g. https://github.com/spurious/SDL-mirror/blob/master/include/SDL_hints.h
    hint::set("SDL_RENDER_SCALE_QUALITY", "1");

    draw_maze(&mut software_renderer, &grid, &options);

    // Getting the surface from the renderer drops the renderer.
    let maze_surface: Surface = software_renderer.into_surface().expect("Failed to get surface from software renderer");

    // WTF: the trait `sdl2_image::SaveSurface` is not implemented for the type `sdl2::surface::Surface<'_>`
    //(&maze_surface as &sdl2_image::SaveSurface).save(&Path::new("./maze_render.png")).expect("Failed to save surface as PNG");
    if let Some(file_path) = options.output_file {
        maze_surface.save_bmp(file_path).expect("Failed to save surface as BMP");
    }

    if options.show_on_screen {
        show_maze_on_screen(maze_surface, sdl_setup);
    }
}

fn draw_maze<GridIndexType>(r: &mut Renderer, grid: &SquareGrid<GridIndexType>, options: &RenderOptions)
    where GridIndexType: IndexType
{
    // clear the texture background to white
    r.set_draw_color(WHITE);
    r.clear();

    // Set the maze wall colour.
    r.set_draw_color(BLUE);

    let cell_size_pixels = options.cell_side_pixels_length as usize;
    let img_width = cell_size_pixels * grid.dimension();  // usize usize
    let img_height = cell_size_pixels * grid.dimension();
    let (max_width, max_height) = match r.logical_size() { (w, h) => (w as usize, h as usize) };

    let x_centering_offset = if img_width  < max_width { (max_width - img_width)/2 } else { 0 };
    let y_centering_offset = if img_height < max_height { (max_height - img_height)/2 } else { 0 };

    for cell in grid.iter() {
        let column = cell.x as usize;
        let row = cell.y as usize;

        let x1 = ((column * cell_size_pixels) + x_centering_offset) as i32;
        let y1 = ((row * cell_size_pixels) + y_centering_offset) as i32;
        let x2 = (((column + 1) * cell_size_pixels) + x_centering_offset) as i32;
        let y2 = (((row + 1) * cell_size_pixels) + y_centering_offset) as i32;

        // special cases north and west to handle first row and column.
        if grid.neighbour_at_direction(&cell, GridDirection::North).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x2, y1)).unwrap();
        }
        if grid.neighbour_at_direction(&cell, GridDirection::West).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x1, y2)).unwrap();
        }

        if !grid.is_neighbour_linked(&cell, GridDirection::East) {
            r.draw_line(Point::new(x2, y1), Point::new(x2, y2)).unwrap();
        }
        if !grid.is_neighbour_linked(&cell, GridDirection::South) {
            r.draw_line(Point::new(x1, y2), Point::new(x2, y2)).unwrap();
        }
    }
}

fn show_maze_on_screen(maze_surface: Surface, sdl_setup: sdl::SdlSetup) {
    let mut window_builder = sdl_setup.video_subsystem.window("Mazes", INITIAL_W, INITIAL_H);
    let window = window_builder.position_centered()
                               .resizable()
                               .allow_highdpi()
                               .build()
                               .unwrap();
    let mut renderer = window.renderer()
                             .present_vsync()
                             .accelerated()
                             .target_texture()
                             .build()
                             .unwrap();

    renderer.set_logical_size(LOGICAL_W, LOGICAL_H).unwrap();

    let screen_texture = renderer.create_texture_from_surface(maze_surface).unwrap();

    let mut events = sdl_setup.sdl_context.event_pump().unwrap();
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'running,
                Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Q), ..} => break 'running,
                _ => continue,
            }
        }

        renderer.set_draw_color(WHITE);
        renderer.clear();
        renderer.copy(&screen_texture, None, None);
        renderer.present();
    }
}

fn draw_maze_to_texture<GridIndexType>(r: &mut Renderer,
                                       t: Texture,
                                       grid: &SquareGrid<GridIndexType>, options: &RenderOptions)
                                       -> Texture
    where GridIndexType: IndexType
{
    // Setup to draw to the given texture. The texture is moved/owned by the `set` call.
    r.render_target()
     .expect("This platform doesn't support render targets")
     .set(t).unwrap(); // Returns the old render target if the function is successful, which we ignore.

    draw_maze(r, &grid, &options);

    // Reseting gives us back ownership of the updated texture and restores the default render target
    let updated_texture: Option<Texture> = r.render_target().unwrap().reset().unwrap();
    updated_texture.unwrap()
}


//// Research Notes
//
// For a non-text based view of a maze we need a GUI window if
// we want to see anything live.
// To just write out an image we only need an sdl surface
// to draw on and write out as a PNG?
// Renderer seems to require a window...though the window
// could be hidden I guess?
// - sdl_context -> video_subsystem -> window_builder
// - window_builder options:
//   - *hidden*, fullscreen, opengl enabled, borderless, resizable
//
// sdl2_image provides:
// trait SaveSurface for Surface (save as PNG only)
// trait LoadSurface for Surface (load any texture)
// trait LoadTexture for Renderer
//
// sdl2 provides a Surface accessor on the Renderer type, which
// may also work even if the Renderer is using a Texture?
// I'm guessing that *all* renderers for a window have a Surface, but
// textures as memory locations on the GPU are optional.
// SDL_Surface is just a collection of pixels while SDL_Texture is an
// efficient, driver-specific representation of pixel data
// SDL_Texture on the other hand, is used in a hardware rendering, textures
// are stored in VRAM and you don't have access to it directly as with SDL_Surface.
// The rendering operations are accelerated by GPU, using, internally, either
// OpenGL or DirectX (available only on Windows) API, which in turn are using
// your video hardware, hence hardware rendering name.
// http://stackoverflow.com/questions/21007329/what-is-a-sdl-renderer/21007477#21007477
//
// Actually the section "If your game just wants to get fully-rendered frames to the screen"
// looks like what we want: https://wiki.libsdl.org/MigrationGuide
//
// should we use rendererbuilder target_texture? "Set the renderer to support rendering to a texture."
// does that mean we can render to a texture then do whatever.... renderer.create_texture_target
// renderer. render_target(&mut self) -> Option<RenderTarget>
// When the render target has been set/created the draw_line/fill_rect etc calls are directed to that
// render target.
//
// # drawing every pixel yourself in software and then doing one big blit (software doom etc.)
// - single sdlTexture to represent the screen
// - texture access streaming (frequent content change of the texture)
// - create surface or [u8] as RGBA buffer block of pixels (can convert if need to from other formats)
// - sdl_updatetexture (texture.update) at the end which uploads the pixels to the gpu.
// - [SDL_RenderSetLogicalSize()] so we get scaling as required.
// - render clear
// - rendercopy sdlTexture (put the texture on the gpu in to the backbuffer/framebuffer memory)
// - render present
// Examples:
// - https://github.com/AngryLawyer/rust-sdl2/blob/master/examples/renderer-texture.rs shows streaming texture
//   but does not draw any lines, just messes with a mutable [u8] buffer.
//
// # blitting multiple "sprites" to the screen (treating surfaces as sprites, not pixel buffers)
//   textures tend to be static once uploaded
// - create texture(s) one per sprite etc. +
// - texture access static
// (or createTextureFromSurface as these two steps)
// (it's annoying that the rust api makes it hard to draw a line to a surface)
//
// # Blit surfaces and modify individual pixels in the framebuffer.
//   Round trips--reading data back from textures--can be painfully expensive; generally you want to be pushing data in one direction always.
//   You are probably best off, in this case, keeping everything in software until the final push to the screen, so we'll combine the two previous techniques.
//  - create 'screen' texture
//  - create screen surface
//  - compose the final framebuffer into the sdl_surface.
//  - update texture from screen surface
//  - render clear && render copy 'screen' texture && render present
// N.B.
// SDL_TEXTUREACCESS_STATIC changes rarely, not lockable
// SDL_TEXTUREACCESS_STREAMING changes frequently, lockable
// SDL_TEXTUREACCESS_TARGET can be used as a render target
// Static textures are designed for sprites and backgrounds and other images that don't change much.
// You update them with pixels using SDL_UpdateTexture() (which is pretty slow compared to streaming).
// ```
// let static_maze_texture = renderer.create_texture_static(PixelFormatEnum::RGB888, LOGICAL_W, LOGICAL_H).unwrap();
// Surface::new(...)
// data = surface.without_lock() -> Option<&[u8]> to get the pixel data
// static_maze_texture.update(..., data)
// ```
// Streaming textures are designed for things that update frequently, every few seconds, or every frame.
// You can also update them with SDL_UpdateTexture(), but for optimal performance you lock them, write the pixels,
// and then unlock them.
// Presumably locking allows you to write to them on the GPU directly as you 'stream' the data into them, whereas
// updateTexture just blats the whole thing.
//
// If the renderer is in fullscreen desktop mode we wouldn't know or care what the size
// is but we could then SDL_RenderSetLogicalSize(sdlRenderer, 640, 480); for example.
// The app works with a given logical size but sdl scales it on the GPU, even handling scaling
// with different aspect ratios and letterboxing the difference.
