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

    // Todo - why does SDL_LogCritical get called so much by fill_rect/draw_line?
    b.iter(|| renderers::render_square_grid(&maze_grid, &render_options));
}
