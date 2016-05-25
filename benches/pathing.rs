#![feature(test)]

extern crate mazes;
extern crate test;

use mazes::coordinates::GridCoordinate
use mazes::squaregrid::SquareGrid;
use mazes::generators;
use mazes::pathing;
use test::Bencher;


#[bench]
fn bench_distances(b: &mut Bencher) {

    let mut g = SquareGrid::<u32>::new(350);
    generators::recursive_backtracker(&mut g);
    let start_coord = GridCoordinate::new(250,250);

    b.iter(|| pathing::DijkstraDistances::<u32>::new(&g, start_coord));
}

#[bench]
fn bench_furthest_points(b: &mut Bencher) {
    let mut g = SquareGrid::<u32>::new(350);
    generators::recursive_backtracker(&mut g);
    let start_coord = GridCoordinate::new(250,250);
    let distances = pathing::DijkstraDistances::<u32>::new(&g, start_coord).unwrap();

    b.iter(|| distances.furthest_points_on_grid());
}

#[bench]
fn bench_shortest_path(b: &mut Bencher) {
    let mut g = SquareGrid::<u32>::new(350);
    generators::recursive_backtracker(&mut g);
    let start_coord = GridCoordinate::new(250,250);
    let distances = pathing::DijkstraDistances::<u32>::new(&g, start_coord).unwrap();
    let end_coord = GridCoordinate::new(0, 0);

    b.iter(|| pathing::shortest_path(&g, &distances, end_coord));
}
