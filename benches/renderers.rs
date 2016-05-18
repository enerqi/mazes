#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::generators;
use mazes::renderers;
use mazes::pathing;
use mazes::squaregrid;

use test::Bencher;

#[bench]
fn render_grid(b: &mut Bencher) {

    let grid_size = 200;
    let mut maze_grid = squaregrid::SquareGrid::<u32>::new(grid_size);
    let start_coord = squaregrid::GridCoordinate::new(0, 0);
    generators::recursive_backtracker(&mut maze_grid);
    let distances = pathing::DijkstraDistances::<u32>::new(&maze_grid, start_coord);

    let render_options = renderers::RenderOptionsBuilder::new()
             .colour_distances(true)
             .mark_start_end(true)
             .start(Some(start_coord))
             .end(Some(squaregrid::GridCoordinate::new(199, 199)))
             .show_path(true)
             .distances(distances.as_ref())
             .build();

    // Why does SDL_LogCritical get called so much by fill_rect/draw_line? At least according to the CodeXL sampling profiler.
    // render recursive-backtracker image --mark-start-end --grid-size 2500 --cell-pixels 8 --show-path --image-out=perf-test-maze.png --colour-distances
    // 23% of samples and it does not improve when setting SDL_LogSetOutputFunction to nothing:
    // extern "C" {
    //                                         // maybe null function pointer
    // pub fn SDL_LogSetOutputFunction(callback: Option<extern fn()>,
    //                                 userdata: Option<extern fn()>);
    // }
    // unsafe { SDL_LogSetOutputFunction(None, None); }
    // Probably a non-issue:
    // It happens with msvc aswell.
    // It disappears when using a rebuilt from scratch 64bit msvc SDL lib/dll.
    // The runtime is basically the same
    b.iter(|| renderers::render_square_grid(&maze_grid, &render_options));
}
