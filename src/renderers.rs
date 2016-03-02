use petgraph::graph::IndexType;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};

use sdl;
use squaregrid::SquareGrid;


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

    let sdl_setup = sdl::init();
    let window_width = 1280;
    let window_height = 720;
    let mut window_builder = sdl_setup.video_subsystem.window("Mazes", window_width, window_height);
    let window = window_builder.position_centered()
                               //.borderless()
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

    let mut events = sdl_setup.sdl_context.event_pump().unwrap();
    'event: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'event,
                Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Q), ..} => break 'event,
                _ => continue,
            }
        }

        // clear the buffer to white
        renderer.set_draw_color(white);
        renderer.clear();

        // Test rectangle
        let rect = Rect::new(window_width as i32 / 4,
                             window_height as i32 / 4,
                             window_width / 2,
                             window_height / 2)
                       .ok()
                       .expect("sdl create rect failed")
                       .expect("width or height must not be 0");
        renderer.set_draw_color(green);
        renderer.fill_rect(rect);

        // Test Line
        renderer.set_draw_color(blue);
        renderer.draw_line(Point::new(0, window_height as i32 / 2),
                           Point::new(window_width as i32, window_height as i32 / 2));


        // A limitation of drawing to something that we are showing on the screen is surely that the OS
        // may not want to show a window with a stupidly large pixel size, whereas an image file can
        // be at a much large scale. The window can still be shown and closed though.
        let cell_size_pixels = 10;
        let img_width = cell_size_pixels * grid.dimension();
        let img_height = cell_size_pixels * grid.dimension();

        renderer.present();
    }
}
