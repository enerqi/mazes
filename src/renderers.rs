use std::cmp;
use std::fmt;
use std::path::Path;

use sdl2;
use sdl2::event::{Event, WindowEventId};
use sdl2::hint;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::Renderer; //  Teuxture
use sdl2::surface::Surface;
use sdl2_image::SaveSurface;
use sdl2_ttf;

use sdl;
use sdl::SdlSetup;

use cells::{Cell, CompassPrimary, Cartesian2DCoordinate, SquareCell};
use grids::{Grid, IndexType};
use gridTraits::{GridIterators, GridDisplay, GridDimensions, GridPositions};
use pathing;
use units::{RowsCount, ColumnsCount};

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
    start: Option<Cartesian2DCoordinate>,
    end: Option<Cartesian2DCoordinate>,
    show_path: bool,
    distances: Option<&'dist pathing::Distances<SquareCell, u32>>,
    output_file: Option<&'path Path>,
    path: Option<Vec<Cartesian2DCoordinate>>,
    cell_side_pixels_length: u8,
}

#[derive(Debug)]
pub struct RenderOptionsBuilder<'path, 'dist> {
    options: RenderOptions<'path, 'dist>,
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
                start: None,
                end: None,
                show_path: false,
                distances: None,
                output_file: None,
                path: None,
                cell_side_pixels_length: 10,
            },
        }
    }
    pub fn show_on_screen(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.show_on_screen = on;
        self
    }
    pub fn colour_distances(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.colour_distances = on;
        self
    }
    pub fn mark_start_end(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.mark_start_end = on;
        self
    }
    pub fn start(mut self, start: Option<Cartesian2DCoordinate>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.start = start;
        self
    }
    pub fn end(mut self, end: Option<Cartesian2DCoordinate>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.end = end;
        self
    }
    pub fn show_path(mut self, on: bool) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.show_path = on;
        self
    }
    pub fn distances(mut self,
                     distances: Option<&'dist pathing::Distances<SquareCell, u32>>)
                     -> RenderOptionsBuilder<'path, 'dist> {
        self.options.distances = distances;
        self
    }
    pub fn output_file(mut self,
                       output_file: Option<&'path Path>)
                       -> RenderOptionsBuilder<'path, 'dist> {
        self.options.output_file = output_file;
        self
    }
    pub fn path(mut self, path: Option<Vec<Cartesian2DCoordinate>>) -> RenderOptionsBuilder<'path, 'dist> {
        self.options.path = path;
        self
    }
    pub fn cell_side_pixels_length(mut self,
                                   cell_side_pixels_length: u8)
                                   -> RenderOptionsBuilder<'path, 'dist> {
        self.options.cell_side_pixels_length = cell_side_pixels_length;
        self
    }
    pub fn build(self) -> RenderOptions<'path, 'dist> {
        self.options
    }
}


pub fn render_square_grid<GridIndexType, Iters>(grid: &Grid<GridIndexType, SquareCell, Iters>,
                                                options: &RenderOptions)
    where GridIndexType: IndexType,
          Iters: GridIterators<SquareCell>
{
    let sdl_setup = sdl::init();

    // Logically eg. 20x20 grid === 200 x 200 pixels + 32 on the sides (232x232).
    // scaled to whatever the window size is, which maybe a different aspect ratio.
    let (image_w, image_h) = maze_image_dimensions(&grid, &options);

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
    // SDL_HINT_RENDER_SCALE_QUALITY applies per texture, not per renderer.
    hint::set("SDL_RENDER_SCALE_QUALITY", "1");

    draw_maze(&mut software_renderer, &grid, &options, &sdl_setup);

    // Getting the surface from the renderer drops the renderer.
    let maze_surface: Surface = software_renderer.into_surface()
        .expect("Failed to get surface from software renderer");

    if let Some(file_path) = options.output_file {
        maze_surface.save(file_path).expect("Failed to save surface");
    }

    if options.show_on_screen {
        show_maze_on_screen(maze_surface, sdl_setup);
    }
}

fn draw_maze<GridIndexType, Iters>(r: &mut Renderer,
                                   grid: &Grid<GridIndexType, SquareCell, Iters>,
                                   options: &RenderOptions,
                                   sdl_setup: &SdlSetup)
    where GridIndexType: IndexType,
          Iters: GridIterators<SquareCell>
{
    // clear the texture background to white
    r.set_draw_color(WHITE);
    r.clear();

    let distance_colour = GREEN;
    let wall_colour = BLUE;
    r.set_draw_color(wall_colour);

    let cell_size_pixels = options.cell_side_pixels_length as usize;

    // Font creation
    let font_path: &Path = Path::new("resources/Roboto-Regular.ttf");
    let font_px_size = ((cell_size_pixels as f32) * 0.8) as u16;
    let mut font = sdl_setup.ttf_context
        .load_font(&font_path, font_px_size)
        .expect("Failed to load font");
    font.set_style(sdl2_ttf::STYLE_BOLD);

    // Start and end symbol letters rendered to different surfaces
    let s_surface = font.render("S").blended(BLACK).unwrap();
    let e_white_surface = font.render("E").blended(WHITE).unwrap();
    let e_black_surface = font.render("E").blended(BLACK).unwrap();

    let calc_cell_screen_coordinates = |cell_coord: Cartesian2DCoordinate| -> (i32, i32, i32, i32) {
        let column = cell_coord.x as usize;
        let row = cell_coord.y as usize;
        let x1 = (column * cell_size_pixels) as i32;
        let y1 = (row * cell_size_pixels) as i32;
        let x2 = ((column + 1) * cell_size_pixels) as i32;
        let y2 = ((row + 1) * cell_size_pixels) as i32;
        (x1, y1, x2, y2)
    };

    let max_cell_distance = if let Some(dist) = options.distances {
        dist.max()
    } else {
        0
    };
    let max_cell_distance_f: f32 = max_cell_distance as f32;

    for cell in grid.iter() {

        let (x1, y1, x2, y2) = calc_cell_screen_coordinates(cell);

        // special cases north and west to handle first row and column.
        if grid.neighbour_at_direction(cell, CompassPrimary::North).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x2, y1)).unwrap();
        }
        if grid.neighbour_at_direction(cell, CompassPrimary::West).is_none() {
            r.draw_line(Point::new(x1, y1), Point::new(x1, y2)).unwrap();
        }

        // We don't want to draw unnecessary walls for cells that cannot be accessed, so if there are no links to a cell
        // and no links to the neighbour it shares a wall with then the wall need not be drawn.
        let are_links_count_of_valid_cells_zero =
            |c: Cartesian2DCoordinate, neighbour_direction: CompassPrimary| -> bool {
                let cell_links_count_is_zero =
                    |c| grid.links(c).map_or(false, |linked_cells| linked_cells.is_empty());

                if cell_links_count_is_zero(c) {
                    grid.neighbour_at_direction(c, neighbour_direction)
                        .map_or(false, |neighbour| cell_links_count_is_zero(neighbour))
                } else {
                    false
                }
            };

        let must_draw_east_wall = !grid.is_neighbour_linked(cell, CompassPrimary::East) &&
                                  !are_links_count_of_valid_cells_zero(cell, CompassPrimary::East);
        let must_draw_south_wall = !grid.is_neighbour_linked(cell, CompassPrimary::South) &&
                                   !are_links_count_of_valid_cells_zero(cell, CompassPrimary::South);

        if must_draw_east_wall {
            r.draw_line(Point::new(x2, y1), Point::new(x2, y2)).unwrap();
        }
        if must_draw_south_wall {
            r.draw_line(Point::new(x1, y2), Point::new(x2, y2)).unwrap();
        }

        let distance_to_cell = if let Some(dist) = options.distances {
            // The cell maybe unreachable
            dist.distance_from_start_to(cell).unwrap_or(max_cell_distance)
        } else {
            0
        };
        let distance_to_cell_f = distance_to_cell as f32;

        if options.colour_distances || options.mark_start_end {

            // Pixels on which to draw a particular cell
            // offset inside the wall line
            let cell_x1 = x1 + 1;
            let cell_y1 = y1 + 1;
            // extend to cover where not drawing the wall if required
            let cell_x2 = if must_draw_east_wall {
                x2
            } else {
                x2 + 1
            };
            let cell_y2 = if must_draw_south_wall {
                y2
            } else {
                y2 + 1
            };

            let w = (cell_x2 - cell_x1) as u32;
            let h = (cell_y2 - cell_y1) as u32;

            if options.colour_distances {
                let intensity = (max_cell_distance_f - distance_to_cell_f) / max_cell_distance_f;
                let cell_colour = colour_mul(distance_colour, intensity);

                // let cell_colour = rainbow_colour(intensity);

                r.set_draw_color(cell_colour);
                let cell_bg_rect = Rect::new(cell_x1, cell_y1, w, h);
                r.fill_rect(cell_bg_rect).unwrap();
                r.set_draw_color(wall_colour);
            }

            if options.mark_start_end {

                // Start?
                let is_start = if let Some(start_coord) = options.start {
                    start_coord == cell
                } else {
                    distance_to_cell == 0
                };
                if is_start {
                    s_surface.blit(None,
                              r.surface_mut().unwrap(),
                              Some(Rect::new(cell_x1 + 1, cell_y1 - 1, w - 1, h - 1)))
                        .expect("S blit to maze surface failed");
                }

                let is_end = if let Some(end_coord) = options.end {
                    end_coord == cell
                } else {
                    distance_to_cell == max_cell_distance
                };
                if is_end {
                    let end_surface = if options.colour_distances {
                        &e_white_surface
                    } else {
                        &e_black_surface
                    };
                    end_surface.blit(None,
                              r.surface_mut().unwrap(),
                              Some(Rect::new(cell_x1 + 1, cell_y1 - 1, w - 1, h - 1)))
                        .expect("E blit to maze surface failed");
                }
            }
        }
    }

    if let Some(ref path) = options.path {

        let path_long_enough_to_show =
            |path: &[Cartesian2DCoordinate], options: &RenderOptions| -> bool {
                if options.mark_start_end {
                    path.len() >= 4
                } else {
                    path.len() >= 2
                }
            };

        if path_long_enough_to_show(&path, &options) {

            let calc_cell_centre_screen_coordinate = |cell| {
                let (x1, y1, x2, y2) = calc_cell_screen_coordinates(cell);
                let half_w = (x2 - x1) / 2;
                let half_h = (y2 - y1) / 2;
                let mid_x = x1 + half_w;
                let mid_y = y1 + half_h;
                (mid_x, mid_y)
            };

            r.set_draw_color(HOT_PINK);

            let (skip_amount, take_amount) = if options.mark_start_end {
                (1, path.len() - 2)
            } else {
                (0, path.len())
            };
            let mut last_cell_draw_pos = calc_cell_centre_screen_coordinate(path[skip_amount]);

            for cell in path.iter().skip(skip_amount).take(take_amount) {
                let path_line_point_1 = last_cell_draw_pos;
                let path_line_point_2 = calc_cell_centre_screen_coordinate(*cell);

                r.draw_line(Point::from(path_line_point_1),
                            Point::from(path_line_point_2))
                    .unwrap();

                last_cell_draw_pos = path_line_point_2;
            }
        }
    }
}

fn show_maze_on_screen(maze_surface: Surface, sdl_setup: SdlSetup) {

    // Fit the window size to the texture unless the texture is bigger than the display resolution
    let primary_display_mode = sdl_setup.video_subsystem.current_display_mode(0).unwrap();
    let (maze_w, maze_h) = (maze_surface.width(), maze_surface.height());
    let (display_w, display_h) = (primary_display_mode.w as u32, primary_display_mode.h as u32);
    let maze_image_padding = 32;
    let window_w = cmp::min(display_w, maze_w + maze_image_padding);
    let window_h = cmp::min(display_h, maze_h + maze_image_padding);

    let mut window_builder = sdl_setup.video_subsystem.window("Mazes", window_w, window_h);
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

    let maze_texture = renderer.create_texture_from_surface(maze_surface).unwrap();
    let mut maze_target_rect = centre_rectangle(maze_w, maze_h, window_w, window_h);

    let mut events = sdl_setup.sdl_context.event_pump().unwrap();
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Q), .. } => break 'running,
                Event::Window { win_event_id: WindowEventId::Resized,
                                data1: new_width,
                                data2: new_height,
                                .. } => {
                    maze_target_rect =
                        centre_rectangle(maze_w, maze_h, new_width as u32, new_height as u32);
                }
                _ => continue,
                // todo allow resolution > display size?
                // todo allow control of max on screen window size
            }
        }

        renderer.set_draw_color(WHITE);
        renderer.clear();
        renderer.copy(&maze_texture, None, Some(maze_target_rect));
        renderer.present();
    }
}

fn maze_image_dimensions<GridIndexType, CellT, Iters>(grid: &Grid<GridIndexType, CellT, Iters>,
                                        options: &RenderOptions)
                                        -> (u32, u32)
    where GridIndexType: IndexType,
          CellT: Cell,
          Iters: GridIterators<CellT>
{
    let cell_size_pixels = options.cell_side_pixels_length as usize;
    let img_width = cell_size_pixels as u32 * grid.row_length().0 as u32;
    let img_height = cell_size_pixels as u32 * grid.column_length().0 as u32;

    (img_width + 1, img_height + 1)
}

// fn draw_maze_to_texture<GridIndexType, CellT>(r: &mut Renderer,
//                                        t: Texture,
//                                        grid: &Grid<GridIndexType, CellT>,
//                                        options: &RenderOptions,
//                                        sdl_setup: &SdlSetup)
//                                        -> Texture
//     where GridIndexType: IndexType,
//           CellT: Cell
// {
//     // Setup to draw to the given texture. The texture is moved/owned by the `set` call.
//     r.render_target()
//         .expect("This platform doesn't support render targets")
//         .set(t)
//         .unwrap(); // Returns the old render target if the function is successful, which we ignore.

//     draw_maze(r, &grid, &options, &sdl_setup);

//     // Reseting gives us back ownership of the updated texture and restores the default render target
//     let updated_texture: Option<Texture> = r.render_target().unwrap().reset().unwrap();
//     updated_texture.unwrap()
// }

fn colour_mul(colour: Color, scale: f32) -> Color {
    match colour {
        Color::RGB(r, g, b) => {
            Color::RGB((r as f32 * scale) as u8,
                       (g as f32 * scale) as u8,
                       (b as f32 * scale) as u8)
        }
        Color::RGBA(r, g, b, a) => {
            Color::RGBA((r as f32 * scale) as u8,
                        (g as f32 * scale) as u8,
                        (b as f32 * scale) as u8,
                        a)
        }
    }
}

fn rainbow_colour(cycle_complete_percent: f32) -> Color {

    let rainbow_point = match cycle_complete_percent {
        n if n > 1.0 => 1.0,
        n if n < 0.0 => 0.0,
        n => n,
    };
    let center = 128.0;
    let width = 127.0;
    let red_frequency = 0.7;
    let green_frequency = 0.8;
    let blue_frequency = 0.9;
    let len = 250.0;
    let red_phase = 0.0;
    let green_phase = 2.0;
    let blue_phase = 4.0;
    let i = len - (rainbow_point * len);
    let red = (red_frequency * i + red_phase) * width + center;
    let green = (green_frequency * i + green_phase) * width + center;
    let blue = (blue_frequency * i + blue_phase) * width + center;

    Color::RGB(red as u8, green as u8, blue as u8)
}

/// Return a Rect that is centered within a parent rectangle. The rectangle will be scaled down to fit within the parent rectangle
/// if it is bigger than the parent rectangle's width or height.
/// `rect_width` - width of some rectangle to centre.
/// `rect_height` - height of some rectangle to centre.
/// `parent_rect_width` - width of the parent rectangle within which we centre a rectangle.
/// `parent_rect_height` - height of the parent rectangle within which we centre a rectangle.
fn centre_rectangle(rect_width: u32,
                    rect_height: u32,
                    parent_rect_width: u32,
                    parent_rect_height: u32)
                    -> Rect {

    let rect_width_f = rect_width as f32;
    let rect_height_f = rect_height as f32;
    let parent_rect_width_f = parent_rect_width as f32;
    let parent_rect_height_f = parent_rect_height as f32;
    let parent_rect_width_i = parent_rect_width as i32;
    let parent_rect_height_i = parent_rect_height as i32;

    let width_ratio = rect_width_f / parent_rect_width_f;
    let height_ratio = rect_height_f / parent_rect_height_f;

    let (w, h) = if width_ratio > 1.0 || height_ratio > 1.0 {

        if width_ratio > height_ratio {
            let h = (rect_height_f / width_ratio) as i32;
            (parent_rect_width as i32, h)
        } else {
            let w = (rect_width_f / height_ratio) as i32;
            (w, parent_rect_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (parent_rect_width_i - w) / 2;
    let cy = (parent_rect_height_i - h) / 2;
    Rect::new(cx, cy, w as u32, h as u32)
}

// Todo - displaying other grid types, e.g. impl<GridIndexType: IndexType> fmt::Display for Grid<GridIndexType, HexCell>
impl<GridIndexType, Iters> fmt::Display for Grid<GridIndexType, SquareCell, Iters>
    where GridIndexType: IndexType,
          Iters: GridIterators<SquareCell>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const WALL_L: &'static str = "╴";
        const WALL_R: &'static str = "╶";
        const WALL_U: &'static str = "╵";
        const WALL_D: &'static str = "╷";
        const WALL_LR_3: &'static str = "───";
        const WALL_LR: &'static str = "─";
        const WALL_UD: &'static str = "│";
        const WALL_LD: &'static str = "┐";
        const WALL_RU: &'static str = "└";
        const WALL_LU: &'static str = "┘";
        const WALL_RD: &'static str = "┌";
        const WALL_LRU: &'static str = "┴";
        const WALL_LRD: &'static str = "┬";
        const WALL_LRUD: &'static str = "┼";
        const WALL_RUD: &'static str = "├";
        const WALL_LUD: &'static str = "┤";
        let default_cell_body = String::from("   ");

        let ColumnsCount(columns_count) = self.columns();
        let RowsCount(rows_count) = self.rows();

        // Start by special case rendering the text for the north most boundary
        let first_grid_row: &Vec<Cartesian2DCoordinate> =
            &self.iter_row().take(1).collect::<Vec<Vec<_>>>()[0];
        let mut output = String::from(WALL_RD);
        for (index, coord) in first_grid_row.iter().enumerate() {
            output.push_str(WALL_LR_3);
            let is_east_open = self.is_neighbour_linked(*coord, CompassPrimary::East);
            if is_east_open {
                output.push_str(WALL_LR);
            } else {
                let is_last_cell = index == (columns_count - 1);
                if is_last_cell {
                    output.push_str(WALL_LD);
                } else {
                    output.push_str(WALL_LRD);
                }
            }
        }
        output.push_str("\n");

        for (index_row, row) in self.iter_row().enumerate() {

            let is_last_row = index_row == (rows_count - 1);

            // Starts of by special case rendering the west most boundary of the row
            // The top section of the cell is done by the previous row.
            let mut row_middle_section_render = String::from(WALL_UD);
            let mut row_bottom_section_render = String::from("");

            for (index_column, cell_coord) in row.into_iter().enumerate() {

                let render_cell_side = |direction, passage_clear_text, blocking_wall_text| {
                    self.neighbour_at_direction(cell_coord, direction)
                        .map_or(blocking_wall_text, |neighbour_coord| {
                            if self.is_linked(cell_coord, neighbour_coord) {
                                passage_clear_text
                            } else {
                                blocking_wall_text
                            }
                        })
                };
                let is_first_column = index_column == 0;
                let is_last_column = index_column == (columns_count - 1);
                let east_open = self.is_neighbour_linked(cell_coord, CompassPrimary::East);
                let south_open = self.is_neighbour_linked(cell_coord, CompassPrimary::South);

                // Each cell will simply use the southern wall of the cell above
                // it as its own northern wall, so we only need to worry about the cell’s body (room space),
                // its eastern boundary ('|'), and its southern boundary ('---+') minus the south west corner.
                let east_boundary = render_cell_side(CompassPrimary::East, " ", WALL_UD);

                // Cell Body
                if let Some(ref displayer) = *self.grid_display() {
                    row_middle_section_render.push_str(displayer.render_cell_body(cell_coord)
                        .as_str());
                } else {
                    row_middle_section_render.push_str(default_cell_body.as_str());
                }

                row_middle_section_render.push_str(east_boundary);

                if is_first_column {
                    row_bottom_section_render = if is_last_row {
                        String::from(WALL_RU)
                    } else if south_open {
                        String::from(WALL_UD)
                    } else {
                        String::from(WALL_RUD)
                    };

                }
                let south_boundary = render_cell_side(CompassPrimary::South, "   ", WALL_LR_3);
                row_bottom_section_render.push_str(south_boundary);

                let corner = match (is_last_row, is_last_column) {
                    (true, true) => WALL_LU,
                    (true, false) => {
                        if east_open {
                            WALL_LR
                        } else {
                            WALL_LRU
                        }
                    }
                    (false, true) => {
                        if south_open {
                            WALL_UD
                        } else {
                            WALL_LUD
                        }
                    }
                    (false, false) => {
                        let access_se_from_east =
                            self.neighbour_at_direction(cell_coord, CompassPrimary::East)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, CompassPrimary::South));
                        let access_se_from_south =
                            self.neighbour_at_direction(cell_coord, CompassPrimary::South)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, CompassPrimary::East));
                        let show_right_section = !access_se_from_east;
                        let show_down_section = !access_se_from_south;
                        let show_up_section = !east_open;
                        let show_left_section = !south_open;

                        match (show_left_section,
                               show_right_section,
                               show_up_section,
                               show_down_section) {
                            (true, true, true, true) => WALL_LRUD,
                            (true, true, true, false) => WALL_LRU,
                            (true, true, false, true) => WALL_LRD,
                            (true, false, true, true) => WALL_LUD,
                            (false, true, true, true) => WALL_RUD,
                            (true, true, false, false) => WALL_LR,
                            (false, false, true, true) => WALL_UD,
                            (false, true, true, false) => WALL_RU,
                            (true, false, false, true) => WALL_LD,
                            (true, false, true, false) => WALL_LU,
                            (false, true, false, true) => WALL_RD,
                            (true, false, false, false) => WALL_L,
                            (false, true, false, false) => WALL_R,
                            (false, false, true, false) => WALL_U,
                            (false, false, false, true) => WALL_D,
                            _ => " ",
                        }
                    }
                };

                row_bottom_section_render.push_str(corner.as_ref());
            }

            output.push_str(row_middle_section_render.as_ref());
            output.push_str("\n");
            output.push_str(row_bottom_section_render.as_ref());
            output.push_str("\n");
        }

        write!(f, "{}", output)
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
