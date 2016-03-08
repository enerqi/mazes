use petgraph::graph::IndexType;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::surface::{Surface, SurfaceRef};

use sdl;
use squaregrid::{GridDirection, SquareGrid};

// impl AsRef for &SurfaceRef {
//     fn as_ref(&self) ->
// }


pub fn render_square_grid<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
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
    // # drawing every pixel yourself in sfotware and then doing one big blit (software doom etc.)
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


    let sdl_setup = sdl::init();

    // If the renderer is in fullscreen desktop mode we wouldn't know or care what the size
    // is but we could then SDL_RenderSetLogicalSize(sdlRenderer, 640, 480); for example.
    // The app works with a given logical size but sdl scales it on the GPU, even handling scaling
    // with different aspect ratios and letterboxing the difference.
    let window_width = 1280;
    let window_height = 720;
    let mut window_builder = sdl_setup.video_subsystem.window("Mazes", window_width, window_height);
    let window = window_builder.position_centered()
                               .resizable()
                               .build()
                               .unwrap();
    let mut renderer = window.renderer()
                             .present_vsync()
                             .accelerated()
                             .build()
                             .unwrap();

    let black = Color::RGB(0, 0, 0);
    let white = Color::RGB(0xff, 0xff, 0xff);
    let red = Color::RGB(0xff, 0, 0);
    let green = Color::RGB(0, 0xff, 0);
    let blue = Color::RGB(0, 0, 0xff);
    let yellow = Color::RGB(0xff, 0xff, 0);


    {
        // Well this won't work as there is no SurfaceRef.
        // There is a surface on the current window. Other surfaces can be
        // converted/optimized to the surface pixel format of the window.
        // what does renderer write to? Probably depends upon the builder.
        // http://stackoverflow.com/questions/21007329/what-is-a-sdl-renderer/21007477#21007477
        // Seems rennderer is drawing/creating textures behind the scenes!
        // In that case our problem is the constant re-creation
        //let surfaceRef: &SurfaceRef = renderer.surface()
        //                                      .expect("The renderer does not have a SurfaceRef.");
        // let surface = unsafe {
        //     // &SurfaceRef -> *mut ll::SDL_Surface -> Surface
        //     // just so we can give create_texture_from_surface a generic
        //     // AsRef type that can convert to &SurfaceRef again!
        //     // And I'm still worried about Surface Drop (SDL_FreeSurface)
        //     Surface::from_ll(surfaceRef.raw())
        // };

        //renderer.create_texture_from_surface(surface).expect("Failed on texture from surface");
    }
    // let texture = renderer.create_texture_from_surface(surfaceRef).unwrap();

    let mut done = false;
    let mut events = sdl_setup.sdl_context.event_pump().unwrap();
    'event: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'event,
                Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Q), ..} => break 'event,
                _ => continue,
            }
        }

        if !done {
          // clear the buffer to white
            renderer.set_draw_color(white);
            renderer.clear();

            let scale = 1.0;
            renderer.set_scale(scale, scale);
            renderer.set_draw_color(blue);

            // A limitation of drawing to something that we are showing on the screen is surely that the OS
            // may not want to show a window with a stupidly large pixel size, whereas an image file can
            // be at a much large scale. The window can still be shown and closed though.
            let cell_size_pixels = 10;
            let img_width = cell_size_pixels * grid.dimension();
            let img_height = cell_size_pixels * grid.dimension();

            for cell in grid.iter() {
                let column = cell.x as usize;
                let row = cell.y as usize;

                let x1 = (column * cell_size_pixels) as i32;
                let y1 = (row * cell_size_pixels) as i32;
                let x2 = ((column + 1) * cell_size_pixels) as i32;
                let y2 = ((row + 1) * cell_size_pixels) as i32;

                // special cases north and west to handle first row and column.
                if grid.neighbour_at_direction(&cell, GridDirection::North).is_none() {
                    renderer.draw_line(Point::new(x1, y1), Point::new(x2, y1));
                }
                if grid.neighbour_at_direction(&cell, GridDirection::West).is_none() {
                    renderer.draw_line(Point::new(x1, y1), Point::new(x1, y2));
                }

                if !grid.is_neighbour_linked(&cell, GridDirection::East) {
                    renderer.draw_line(Point::new(x2, y1), Point::new(x2, y2));
                }
                if !grid.is_neighbour_linked(&cell, GridDirection::South) {
                    renderer.draw_line(Point::new(x1, y2), Point::new(x2, y2));
                }
            }

            // why is the cpu maxed when vsync should be on? Number of lines I guess.
            // 1% CPU with 400 cells?, 2% 900 cells, 3.2% 1600 cells, 4.5% 2500 cells,
            // 7.5% 4900 cells, 9% 6400, 10.5% 8100, 13% 10,000. Max single core = 13% @ 4.2GHz.
            // draw_lines() can be used to avoid a context swap but all the lines in one batch must be connected
            // Need to check the FPS, assuming 60Hz until doing 10K line draws.

            done = true;
        }
        renderer.present();

    }
}
