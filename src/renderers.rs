use std::path::Path;

use petgraph::graph::IndexType;
use sdl2;
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;
use sdl2_ttf;

use sdl;
use sdl::SdlSetup;
use pathing;
use squaregrid::{GridCoordinate, GridDirection, SquareGrid};

const WINDOW_W: u32 = 1920;
const WINDOW_H: u32 = 1080;
const BLACK: Color = Color::RGB(0, 0, 0);
const WHITE: Color = Color::RGB(0xff, 0xff, 0xff);
const RED: Color = Color::RGB(0xff, 0, 0);
const GREEN: Color = Color::RGB(0, 0xff, 0);
const BLUE: Color = Color::RGB(0, 0, 0xff);
const YELLOW: Color = Color::RGB(0xff, 0xff, 0);
const HOT_PINK: Color = Color::RGB(255, 105, 180);


#[derive(Debug)]
pub struct RenderOptions<'path, 'dist> {
    show_on_screen: bool,
    colour_distances: bool,
    mark_start_end: bool,
    show_path: bool,
    distances: Option<&'dist pathing::DijkstraDistances<u32>>,
    output_file: Option<&'path Path>,
    path: Option<Vec<GridCoordinate>>,
    cell_side_pixels_length: u8,
}

#[derive(Debug)]
pub struct RenderOptionsBuilder<'path, 'dist> {
    options: RenderOptions<'path, 'dist>
}
impl<'path, 'dist> Default for RenderOptionsBuilder<'path, 'dist> {
    fn default() -> Self {
        RenderOptionsBuilder::new()
    }
}
impl<'path, 'dist> RenderOptionsBuilder<'path, 'dist> {
    pub fn new() -> RenderOptionsBuilder<'path, 'dist> {
        RenderOptionsBuilder {
            options: RenderOptions {
                show_on_screen: false,
                colour_distances: false,
                mark_start_end: false,
                show_path: false,
                distances: None,
                output_file: None,
                path: None,
                cell_side_pixels_length: 10,
            }
        }
    }
    pub fn show_on_screen(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.show_on_screen = on; self
    }
    pub fn colour_distances(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.colour_distances = on; self
    }
    pub fn mark_start_end(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.mark_start_end = on; self
    }
    pub fn show_path(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.show_path = on; self
    }
    pub fn distances(mut self, distances: Option<&'dist pathing::DijkstraDistances<u32>>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.distances = distances; self
    }
    pub fn output_file(mut self, output_file: Option<&'path Path>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.output_file = output_file; self
    }
    pub fn path(mut self, path: Option<Vec<GridCoordinate>>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.path = path; self
    }
    pub fn cell_side_pixels_length(mut self, cell_side_pixels_length: u8) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.cell_side_pixels_length = cell_side_pixels_length; self
    }
    pub fn build(self) -> RenderOptions<'path, 'dist> {
        self.options
    }
}


pub fn render_square_grid<GridIndexType>(grid: &SquareGrid<GridIndexType>, options: &RenderOptions)
    where GridIndexType: IndexType
{
    let sdl_setup = sdl::init();

    // Logically eg. 20x20 grid === 200 x 200 pixels + 32 on the sides (232x232).
    // scaled to whatever the window size is, which maybe a different aspect ratio.
    let (logical_w, logical_h) = logical_maze_rendering_dimensions(&grid, &options);

    // The image size can happily be the logical width and height. It will not be lossy as all pixels are drawn
    // and there will not be any visualisation issues
    let (image_w, image_h) = (logical_w, logical_h);

    // The visualisation window size can be whatever size we want. If it uses auto scaling by setting a logical size
    // we can easily have aspect ratio issues unless the logical size is the same aspect ratio as the image
    // If we convert a large surface to a texture it will probably have that weird aspect ratio (probably not a power of 2)
    // Specifically: when we copy the texture to the framebuffer, we do not do any rectangle clipping, the texture is
    // stretched over the output window.


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
    println!("image w/h ({:?}, {:?})", image_w, image_h);
    let software_surface = Surface::new(image_w, image_h, PixelFormatEnum::RGB888)
                               .expect("Surface creation failed.");
    let mut software_renderer = Renderer::from_surface(software_surface)
                                    .expect("Software renderer creation failed.");

    // Sets a device independent resolution for rendering.
    // SDL scales to the actual window size, which may change if we allow resizing and is also
    // unknown if we just drop into fullscreen.
    // software_renderer.set_logical_size(logical_w, logical_h).unwrap();

    // 0 or 'nearest' == nearest pixel sampling
    // 1 or 'linear' == linear filtering (supported by OpenGL and Direct3D)
    // 2 or 'best' == anisotropic filtering (supported by Direct3D)
    // The hint strings don't seem to be abstracted in the rust source at the moment but we can see
    // the #defines at e.g. https://github.com/spurious/SDL-mirror/blob/master/include/SDL_hints.h
    hint::set("SDL_RENDER_SCALE_QUALITY", "1");

    draw_maze(&mut software_renderer, &grid, &options, &sdl_setup);

    // Getting the surface from the renderer drops the renderer.
    let maze_surface: Surface =
        software_renderer.into_surface().expect("Failed to get surface from software renderer");

    // WTF: the trait `sdl2_image::SaveSurface` is not implemented for the type `sdl2::surface::Surface<'_>`
    // (&maze_surface as &sdl2_image::SaveSurface).save(&Path::new("./maze_render.png")).expect("Failed to save surface as PNG");
    if let Some(file_path) = options.output_file {
        maze_surface.save_bmp(file_path).expect("Failed to save surface as BMP");
    }

    if options.show_on_screen {
        show_maze_on_screen(maze_surface, sdl_setup);
    }
}

fn draw_maze<GridIndexType>(r: &mut Renderer,
                            grid: &SquareGrid<GridIndexType>,
                            options: &RenderOptions, sdl_setup: &SdlSetup)
    where GridIndexType: IndexType
{
    // clear the texture background to white
    r.set_draw_color(WHITE);
    r.clear();

    let distance_colour = GREEN;
    let wall_colour = BLUE;
    r.set_draw_color(wall_colour);

    let cell_size_pixels = options.cell_side_pixels_length as usize;
    let img_width = cell_size_pixels * grid.dimension() as usize;
    let img_height = cell_size_pixels * grid.dimension() as usize;
    let (max_width, max_height) = match r.logical_size() {
        (w, h) => (w as usize, h as usize),
    };

    let x_centering_offset = if img_width < max_width {
        (max_width - img_width) / 2
    } else {
        0
    };
    let y_centering_offset = if img_height < max_height {
        (max_height - img_height) / 2
    } else {
        0
    };

    // Font creation
    let font_path: &Path = Path::new("resources/Roboto-Regular.ttf");
    let font_px_size = ((cell_size_pixels as f32) * 0.8) as u16;
    let mut font = sdl_setup.ttf_context.load_font(&font_path, font_px_size)
                                    .expect("Failed to load font");
    font.set_style(sdl2_ttf::STYLE_BOLD);

    // Start and end symbol letters rendered to different surfaces
    let s_surface = font.render("S").blended(BLACK).unwrap();
    let e_white_surface = font.render("E").blended(WHITE).unwrap();
    let e_black_surface = font.render("E").blended(BLACK).unwrap();

    let calc_cell_screen_coordinates = |cell: GridCoordinate| -> (i32, i32, i32, i32) {
        let column = cell.x as usize;
        let row = cell.y as usize;
        let x1 = ((column * cell_size_pixels) + x_centering_offset) as i32;
        let y1 = ((row * cell_size_pixels) + y_centering_offset) as i32;
        let x2 = (((column + 1) * cell_size_pixels) + x_centering_offset) as i32;
        let y2 = (((row + 1) * cell_size_pixels) + y_centering_offset) as i32;
        (x1, y1, x2, y2)
    };

    let max_cell_distance = if let Some(dist) = options.distances { dist.max() } else { 0 };
    let max_cell_distance_f: f32 = max_cell_distance as f32;

    for cell in grid.iter() {

        let (x1, y1, x2, y2) = calc_cell_screen_coordinates(cell);

        // special cases north and west to handle first row and column.
        if grid.neighbour_at_direction(cell, GridDirection::North).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x2, y1)).unwrap();
        }
        if grid.neighbour_at_direction(cell, GridDirection::West).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x1, y2)).unwrap();
        }

        let must_draw_east_wall = !grid.is_neighbour_linked(cell, GridDirection::East);
        let must_draw_south_wall = !grid.is_neighbour_linked(cell, GridDirection::South);

        if must_draw_east_wall {
            r.draw_line(Point::new(x2, y1), Point::new(x2, y2)).unwrap();
        }
        if must_draw_south_wall {
            r.draw_line(Point::new(x1, y2), Point::new(x2, y2)).unwrap();
        }

        let distance_to_cell = if let Some(dist) = options.distances {
                dist.distance_from_start_to(cell)
                    .expect("Coordinate invalid for distances_from_start data.")
            } else { 0 };
        let distance_to_cell_f = distance_to_cell as f32;

        if options.colour_distances || options.mark_start_end {

            // Pixels on which to draw a particular cell
            // offset inside the wall line
            let cell_x1 = x1 + 1;
            let cell_y1 = y1 + 1;
            // extend to cover where not drawing the wall if required
            let cell_x2 = if must_draw_east_wall { x2 } else { x2 + 1 };
            let cell_y2 = if must_draw_south_wall { y2 } else { y2 + 1 };

            let w = (cell_x2 - cell_x1) as u32;
            let h = (cell_y2 - cell_y1) as u32;

            if options.colour_distances {
                let intensity = (max_cell_distance_f - distance_to_cell_f) / max_cell_distance_f;
                let cell_colour = colour_mul(distance_colour, intensity);

                r.set_draw_color(cell_colour);
                let cell_bg_rect = Rect::new(cell_x1, cell_y1, w, h);
                r.fill_rect(cell_bg_rect).unwrap();
                r.set_draw_color(wall_colour);
            }

            if options.mark_start_end {

                if distance_to_cell == 0 {
                    // At the start
                    s_surface.blit(None, r.surface_mut().unwrap(),
                                   Some(Rect::new(cell_x1+1, cell_y1-1, w-1, h-1)))
                             .expect("S blit to maze surface failed");
                } else if distance_to_cell == max_cell_distance {
                    // At the end
                    let end_surface = if options.colour_distances { &e_white_surface } else { &e_black_surface };
                    end_surface.blit(None, r.surface_mut().unwrap(),
                                     Some(Rect::new(cell_x1+1, cell_y1-1, w-1, h-1)))
                               .expect("E blit to maze surface failed");
                }
            }
        }
    }

    if let Some(ref path) = options.path {

        let calc_cell_centre_screen_coordinate = |cell| {
            let (x1, y1, x2, y2) = calc_cell_screen_coordinates(cell);
            let half_w = (x2 - x1)/2;
            let half_h = (y2 - y1)/2;
            let mid_x = x1 + half_w;
            let mid_y= y1 + half_h;
            (mid_x, mid_y)
        };

        r.set_draw_color(HOT_PINK);

        let (skip_amount, take_amount) = if options.mark_start_end { (1, path.len() - 2) } else { (0, path.len()) };
        let mut last_cell_draw_pos = calc_cell_centre_screen_coordinate(path[skip_amount]);

        for cell in path.iter().skip(skip_amount).take(take_amount) {
            let path_line_point_1 = last_cell_draw_pos;
            let path_line_point_2 = calc_cell_centre_screen_coordinate(*cell);

            r.draw_line(Point::from(path_line_point_1),
                        Point::from(path_line_point_2)).unwrap();

            last_cell_draw_pos = path_line_point_2;
        }
    }
}

fn show_maze_on_screen(maze_surface: Surface, sdl_setup: SdlSetup) {

    let primary_display_mode = sdl_setup.video_subsystem.current_display_mode(0).unwrap();
    let (w, h) = (primary_display_mode.w as u32, primary_display_mode.h as u32);

    let mut window_builder = sdl_setup.video_subsystem.window("Mazes", w, h);
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

    let screen_texture = renderer.create_texture_from_surface(maze_surface).unwrap();

    let mut events = sdl_setup.sdl_context.event_pump().unwrap();
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} | Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Q), ..} => {
                    break 'running
                }
                _ => continue,
            }
        }

        renderer.set_draw_color(WHITE);
        renderer.clear();
        renderer.copy(&screen_texture, None, None);
        renderer.present();
    }
}

fn logical_maze_rendering_dimensions<GridIndexType>(grid: &SquareGrid<GridIndexType>,
                                                    options: &RenderOptions)
                                                    -> (u32, u32)
    where GridIndexType: IndexType
{
    let cell_size_pixels = options.cell_side_pixels_length as usize;
    let img_width = cell_size_pixels as u32 * grid.dimension();
    let img_height = cell_size_pixels as u32 * grid.dimension();

    (32 + img_width, 32 + img_height)
}

fn draw_maze_to_texture<GridIndexType>(r: &mut Renderer,
                                       t: Texture,
                                       grid: &SquareGrid<GridIndexType>,
                                       options: &RenderOptions,
                                       sdl_setup: &SdlSetup)
                                       -> Texture
    where GridIndexType: IndexType
{
    // Setup to draw to the given texture. The texture is moved/owned by the `set` call.
    r.render_target()
     .expect("This platform doesn't support render targets")
     .set(t)
     .unwrap(); // Returns the old render target if the function is successful, which we ignore.

    draw_maze(r, &grid, &options, &sdl_setup);

    // Reseting gives us back ownership of the updated texture and restores the default render target
    let updated_texture: Option<Texture> = r.render_target().unwrap().reset().unwrap();
    updated_texture.unwrap()
}

fn colour_mul(colour: Color, scale: f32) -> Color {
    match colour {
        Color::RGB(r, g, b) => Color::RGB((r as f32 * scale) as u8, (g as f32 * scale) as u8, (b as f32 * scale) as u8),
        Color::RGBA(r, g, b, a) => Color::RGBA((r as f32 * scale) as u8, (g as f32 * scale) as u8, (b as f32 * scale) as u8, a),
    }
}

// Research Notes
//
// For a non-text based view of a maze we need a GUI window if
// we want to see anything live.
// To just write out an image we only need an sdl surface
// to draw on and write out as a PNG?
// Renderer seems to require a window...though the window
// could be hidden I guess?
// - sdl_context -> video_subsystem -> window_builder
// - window_builder options:
// - *hidden*, fullscreen, opengl enabled, borderless, resizable
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
// but does not draw any lines, just messes with a mutable [u8] buffer.
//
// # blitting multiple "sprites" to the screen (treating surfaces as sprites, not pixel buffers)
// textures tend to be static once uploaded
// - create texture(s) one per sprite etc. +
// - texture access static
// (or createTextureFromSurface as these two steps)
// (it's annoying that the rust api makes it hard to draw a line to a surface)
//
// # Blit surfaces and modify individual pixels in the framebuffer.
// Round trips--reading data back from textures--can be painfully expensive; generally you want to be pushing data in one direction always.
// You are probably best off, in this case, keeping everything in software until the final push to the screen, so we'll combine the two previous techniques.
// - create 'screen' texture
// - create screen surface
// - compose the final framebuffer into the sdl_surface.
// - update texture from screen surface
// - render clear && render copy 'screen' texture && render present
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
//
//
// Ok, the problem is that if we didn't letterbox, then the game written to run at 640x480 is going to distort if we use a whole 1920x1200 display.
// You have a few options:
// - Don't use the logical size API, and render at the full size of the window.
// - Force the window to be a certain aspect ratio (don't let the user resize it by dragging the window frame, give a list of resolutions they can use, that match the monitor's aspect ratio, in a menu somewhere).
// - Use the logical size API, and adjust your rendering to take aspect ratio into account.
// In a perfect world, Option #1 is your best choice, but it takes more work. The logical size API is meant to say "I absolutely need a grid of WxH pixels, my game can't function in any other way, can you figure out how to make this work right?" �...it's absolutely a lifesaver in moderning older games, but if you're doing new code, consider if you shouldn't just take advantage of all available pixels.
// The idea behind a logical size is that no matter what size the SDL_Window
// is, you have a consistent-sized array of pixels to render to that will fill
// that entire window. (This does mean stretching to fit, if necessary.) I
// originally implemented it with a simple call to glOrtho.
// SDL_RenderSetLogicalSize really should simply set the number of drawable
// pixels in the window equal to the value you passed in, and leave it at
// that. Can we fix the current broken implementation please?
//
// SDL_RenderSetLogicalSize(renderer, width, height);
// Where width and height are the resolution your program will give to
// the SDL functions (i.e. the one that isn't the real resolution). All
// renderer functions called after this will scale their coordinates
// accordingly.
// Of course this only applies if you're using SDL's rendering functions,
// not if you're e.g. using OpenGL directly.

