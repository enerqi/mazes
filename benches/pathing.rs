use criterion::{
    Criterion,
    criterion_group,
    criterion_main
};
use mazes::cells::{Cartesian2DCoordinate, SquareCell};
use mazes::generators;
use mazes::grids::large_rect_grid;
use mazes::pathing;
use mazes::units::{ColumnLength, RowLength};

type SquareCellDistances = pathing::Distances<SquareCell, u32>;

fn bench_distances(c: &mut Criterion) {
    c.bench_function("distances", |b| {
        let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
        generators::recursive_backtracker(&mut g, None);
        let start_coord = Cartesian2DCoordinate::new(250, 250);
        b.iter(|| SquareCellDistances::for_grid(&g, start_coord))
    });
}

fn bench_furthest_points(c: &mut Criterion) {
    c.bench_function("furthest_points", |b| {
        let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
        generators::recursive_backtracker(&mut g, None);
        let start_coord = Cartesian2DCoordinate::new(250, 250);
        let distances = SquareCellDistances::for_grid(&g, start_coord).unwrap();
        b.iter(|| distances.furthest_points_on_grid())
    });
}

fn bench_shortest_path(c: &mut Criterion) {
    c.bench_function("shortest_path", |b| {
        let mut g = large_rect_grid(RowLength(350), ColumnLength(350)).unwrap();
        generators::recursive_backtracker(&mut g, None);
        let start_coord = Cartesian2DCoordinate::new(250, 250);
        let distances = SquareCellDistances::for_grid(&g, start_coord).unwrap();
        let end_coord = Cartesian2DCoordinate::new(0, 0);
        b.iter(|| pathing::shortest_path(&g, &distances, end_coord))
    });
}

criterion_group!(benches,
    bench_distances,
    bench_furthest_points,
    bench_shortest_path
);
criterion_main!(benches);
