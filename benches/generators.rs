use criterion::{criterion_group, criterion_main, Criterion};
use mazes::{
    generators,
    grids::medium_rect_grid,
    units::{ColumnLength, RowLength},
};

fn bench_binary_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    c.bench_function("binary_maze_32_u16", move |b| {
        b.iter(|| generators::binary_tree(&mut g))
    });
}

fn bench_sidewinder_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();

    c.bench_function("sidewinder_maze_32_u16", move |b| {
        b.iter(|| generators::sidewinder(&mut g))
    });
}

fn bench_aldous_broder_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();
    c.bench_function("aldous_broder_maze_32_u16", move |b| {
        b.iter(|| generators::aldous_broder(&mut g, None))
    });
}

fn bench_wilson_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();
    c.bench_function("wilson_maze_32_u16", move |b| {
        b.iter(|| generators::wilson(&mut g, None))
    });
}

fn bench_hunt_and_kill_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();
    c.bench_function("hunt_and_kill_maze_32_u16", move |b| {
        b.iter(|| generators::hunt_and_kill(&mut g, None))
    });
}

fn bench_recursive_backtracker_maze_32_u16(c: &mut Criterion) {
    let mut g = medium_rect_grid(RowLength(32), ColumnLength(32)).unwrap();
    c.bench_function("recursive_backtracker_maze_32_u16", move |b| {
        b.iter(|| generators::recursive_backtracker(&mut g, None))
    });
}

criterion_group!(
    benches,
    bench_binary_maze_32_u16,
    bench_sidewinder_maze_32_u16,
    bench_aldous_broder_maze_32_u16,
    bench_wilson_maze_32_u16,
    bench_hunt_and_kill_maze_32_u16,
    bench_recursive_backtracker_maze_32_u16
);
criterion_main!(benches);
