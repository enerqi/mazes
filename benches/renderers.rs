#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::generators;
use mazes::renderers;
use mazes::pathing;
use mazes::squaregrid;

use std::ptr;
use test::Bencher;

extern "C" {
                                            // maybe null function pointer
    pub fn SDL_LogSetOutputFunction(callback: Option<extern fn()>,
                                    userdata: Option<extern fn()>);
}

#[bench]
fn render_grid(b: &mut Bencher) {

    unsafe { SDL_LogSetOutputFunction(None, None); }

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

    // Todo - why does SDL_LogCritical get called so much by fill_rect/draw_line?
    // render recursive-backtracker image --mark-start-end --grid-size 2500 --cell-pixels 8 --show-path --image-out=perf-test-maze.png --colour-distances
    // 23% of samples
    // and it does not improve when setting SDL_LogSetOutputFunction to nothing
    b.iter(|| renderers::render_square_grid(&maze_grid, &render_options));
}

// void SDL_LogSetOutputFunction(SDL_LogOutputFunction callback, void *userdata);
// static void SDL_LogOutput(void *userdata,
//                           int category, SDL_LogPriority priority,
//                           const char *message);

// ?
// static void
// debug_print(const char *fmt, ...)
// {
//     va_list ap;
//     va_start(ap, fmt);
//     SDL_LogMessageV(SDL_LOG_CATEGORY_ASSERT, SDL_LOG_PRIORITY_WARN, fmt, ap);
//     va_end(ap);
// }
//
// SDL_SetError if in debug
//
//  #if LOG_WINDOW_EVENTS==1
