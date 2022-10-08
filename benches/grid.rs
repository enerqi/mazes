use criterion::{criterion_group, criterion_main, Criterion};

use mazes::{
    cells::{Cartesian2DCoordinate, Coordinate},
    grids::{large_rect_grid, medium_rect_grid, small_rect_grid},
    units::{ColumnLength, RowLength},
};

fn bench_maze_11_u8(c: &mut Criterion) {
    c.bench_function("maze 11 u8", |b| {
        b.iter(|| small_rect_grid(RowLength(11), ColumnLength(11)).unwrap())
    });
}

fn bench_maze_11_u16(c: &mut Criterion) {
    c.bench_function("maze 11 u16", |b| {
        b.iter(|| medium_rect_grid(RowLength(11), ColumnLength(11)).unwrap())
    });
}

fn bench_maze_11_u32(c: &mut Criterion) {
    c.bench_function("maze 11 u32", |b| {
        b.iter(|| large_rect_grid(RowLength(11), ColumnLength(11)).unwrap())
    });
}

fn bench_maze_128_u16(c: &mut Criterion) {
    c.bench_function("maze 128 u16", |b| {
        b.iter(|| medium_rect_grid(RowLength(128), ColumnLength(128)).unwrap())
    });
}

fn bench_maze_128_u32(c: &mut Criterion) {
    c.bench_function("maze 128 u32", |b| {
        b.iter(|| large_rect_grid(RowLength(128), ColumnLength(128)).unwrap())
    });
}

fn bench_maze_500(c: &mut Criterion) {
    c.bench_function("maze 500", |b| {
        b.iter(|| large_rect_grid(RowLength(500), ColumnLength(500)).unwrap())
    });
}

fn bench_index_to_gridcoordinate(c: &mut Criterion) {
    c.bench_function("index to gridcoordinate", |b| {
        let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
        let dims = g.dimensions();
        b.iter(|| Cartesian2DCoordinate::from_row_major_index(93, dims))
    });
}

fn bench_neighbours_corner_of_grid(c: &mut Criterion) {
    c.bench_function("neighbours at grid corner", |b| {
        let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
        let corner = Cartesian2DCoordinate::new(0, 0);
        b.iter(|| g.neighbours(corner))
    });
}

fn bench_neighbours_middle_of_grid(c: &mut Criterion) {
    c.bench_function("neighbours in grid middle", |b| {
        let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
        let mid = Cartesian2DCoordinate::new(5, 5);
        b.iter(|| g.neighbours(mid))
    });
}

criterion_group!(
    benches,
    bench_maze_11_u8,
    bench_maze_11_u16,
    bench_maze_11_u32,
    bench_maze_128_u16,
    bench_maze_128_u32,
    bench_maze_500,
    bench_index_to_gridcoordinate,
    bench_neighbours_corner_of_grid,
    bench_neighbours_middle_of_grid
);
criterion_main!(benches);
