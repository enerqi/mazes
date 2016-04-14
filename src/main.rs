#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate docopt;
extern crate mazes;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;

use docopt::Docopt;

use mazes::squaregrid::{CoordinateSmallVec, GridCoordinate, GridDisplay, SquareGrid};
use mazes::generators;
use mazes::renderers;
use mazes::pathing;

const USAGE: &'static str = "Mazes

Usage:
    mazes_driver -h | --help
    mazes_driver [--grid-size=<n>]
    mazes_driver render (binary|sidewinder) [text --text-out=<path> (--show-distances|--show-path) (--find-furthest-point-from-start|--path-end-x=<e1> --path-end-y=<e2>) (--path-start-x=<x> --path-start-y=<y>)] [image --image-out=<path> --cell-pixels=<n> --screen-view] [--grid-size=<n>]

Options:
    -h --help           Show this screen.
    --grid-size=<n>     The grid size is n * n [default: 20].
    --text-out=<path>   Output file path for a textual rendering of a maze.
    --path-start-x=<x>
    --path-start-y=<y>
    --end-point-x=<e1>
    --end-point-y=<e2>
    --furthest-point    Show the furthest (points) from the path-start-x/path-start-y coordinate
    --image-out=<path>  Output file path for an image rendering of a maze.
    --cell-pixels=<n>   Pixel count to render one cell wall in a maze [default: 10] max 255.
    --screen-view       When rendering to an image and saving to a file, also show the image on the screen.
";
#[derive(RustcDecodable, Debug)]
struct MazeArgs {
    flag_grid_size: u32,
    cmd_render: bool,
    cmd_binary: bool,
    cmd_sidewinder: bool,
    cmd_text: bool,
    flag_text_out: String,
    cmd_image: bool,
    flag_image_out: String,
    flag_cell_pixels: u8,
    flag_screen_view: bool,
    flag_show_distances: bool,
    flag_show_path: bool,
    flag_find_furthest_point_from_start: bool,
    flag_path_start_x: Option<u32>,
    flag_path_start_y: Option<u32>,
    flag_end_point_x: Option<u32>,
    flag_end_point_y: Option<u32>,
}


fn main() {

    let args: MazeArgs = Docopt::new(USAGE)
                             .and_then(|d| d.decode())
                             .unwrap_or_else(|e| e.exit());

    let grid_size = args.flag_grid_size;
    let any_render_option = args.cmd_text || args.cmd_image;

    // Do whatever defaults we want if not given a specific 'render' command
    let do_image_render = !args.cmd_render || args.cmd_image ||
                          (!any_render_option && grid_size >= 25);
    let do_text_render = args.cmd_render &&
                         (args.cmd_text || (!any_render_option && grid_size < 25));

    let mut maze_grid = SquareGrid::<u32>::new(grid_size);

    generate_maze_on_grid(&mut maze_grid, &args);

    if do_text_render {

        set_maze_griddisplay(&mut maze_grid, &args);

        if args.flag_text_out.is_empty() {
            println!("{}", maze_grid);
        } else {
            write_text_to_file(&format!("{}", maze_grid), &args.flag_text_out)
                .expect(&format!("Failed to write maze to text file {}", args.flag_text_out));
        }
    }

    if do_image_render {
        let is_image_path_set = !args.flag_image_out.is_empty();
        let out_image_path = if is_image_path_set {
            Some(Path::new(&args.flag_image_out))
        } else {
            None
        };
        let render_opts = renderers::RenderOptions::new(args.flag_screen_view ||
                                                        !is_image_path_set,
                                                        out_image_path,
                                                        args.flag_cell_pixels);
        renderers::render_square_grid(&maze_grid, &render_opts);
    }
}

fn generate_maze_on_grid(mut maze_grid: &mut SquareGrid<u32>, maze_args: &MazeArgs) {

    if maze_args.cmd_render {
        if maze_args.cmd_binary {
            generators::binary_tree(&mut maze_grid);
        }
        if maze_args.cmd_sidewinder {
            generators::sidewinder(&mut maze_grid);
        }
    } else {
        generators::sidewinder(&mut maze_grid);
    }
}

fn set_maze_griddisplay(maze_grid: &mut SquareGrid<u32>, maze_args: &MazeArgs) {

    if let (Some(x), Some(y)) = (maze_args.flag_path_start_x, maze_args.flag_path_start_y) {

        let distances = Rc::new(pathing::DijkstraDistances::<u32>::new(&maze_grid, GridCoordinate::new(x, y))
                .unwrap_or_else(|| exit_with_msg("Provided invalid start coordinate from which to show path distances.")));

        if let (Some(end_x), Some(end_y)) = (maze_args.flag_end_point_x, maze_args.flag_end_point_y) {

            // Given a start and end point - show the shortest path between these two points
            let path_opt = pathing::shortest_path(&maze_grid, &distances, GridCoordinate::new(end_x, end_y));
            if let Some(path) = path_opt {

                let display_path = Rc::new(pathing::PathDisplay::new(&path));
                maze_grid.set_grid_display(Some(display_path as Rc<GridDisplay>));
            } else {

                let start_points = as_coordinate_smallvec(GridCoordinate::new(x, y));
                let end_points = as_coordinate_smallvec(GridCoordinate::new(end_x, end_y));
                let display_start_end_points = Rc::new(pathing::StartEndPointsDisplay::new(start_points, end_points));
                maze_grid.set_grid_display(Some(display_start_end_points as Rc<GridDisplay>));
            }
        }
        else if maze_args.flag_find_furthest_point_from_start {

            // Given a start point and asked to show furthest point away from it - find an end point
            let start_points = as_coordinate_smallvec(GridCoordinate::new(x, y));
            let end_points = pathing::furthest_points_on_grid(&maze_grid, &distances);
            let display_start_end_points = Rc::new(pathing::StartEndPointsDisplay::new(start_points, end_points));
            maze_grid.set_grid_display(Some(display_start_end_points as Rc<GridDisplay>));

        } else {

            // Only given a start point - show the distances to everywhere else
            maze_grid.set_grid_display(Some(distances.clone() as Rc<GridDisplay>));
        }

    } else if maze_args.flag_find_furthest_point_from_start {

        // Not given a start point. Asking for the furthest point means find 2 arbitrary places that are max distance from one another.
        let longest_path: Vec<GridCoordinate> = pathing::dijkstra_longest_path::<_, u32>(&maze_grid).expect("Not a perfect maze, no longest path exists.");
        let start = longest_path.first().unwrap();
        let end = longest_path.last().unwrap();
        let display_start_end_points = Rc::new(pathing::StartEndPointsDisplay::new(as_coordinate_smallvec(*start),
                                                                                   as_coordinate_smallvec(*end)));
        maze_grid.set_grid_display(Some(display_start_end_points as Rc<GridDisplay>));
    }
}

fn as_coordinate_smallvec(coord: GridCoordinate) -> CoordinateSmallVec {
    [coord].into_iter().cloned().collect::<CoordinateSmallVec>()
}

fn write_text_to_file(data: &str, file_name: &str) -> io::Result<()> {
    let mut f = try!(File::create(file_name));
    try!(f.write_all(data.as_bytes()));
    Ok(())
}

fn exit_with_msg(e: &str) -> ! {
    writeln!(&mut io::stderr(), "{}", e).expect(format!("Failed to print error: {}", e).as_str());
    exit(1);
}
