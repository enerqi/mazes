#![feature(test)]

use mazes::cells::{Cartesian2DCoordinate, Coordinate};
use mazes::grids::{large_rect_grid, medium_rect_grid, small_rect_grid};
use mazes::units::{ColumnLength, RowLength};
use test::Bencher;


#[bench]
fn bench_maze_11_u8(b: &mut Bencher) {

    b.iter(|| small_rect_grid(RowLength(11), ColumnLength(11)).unwrap());
}

#[bench]
fn bench_maze_11_u16(b: &mut Bencher) {

    b.iter(|| medium_rect_grid(RowLength(11), ColumnLength(11)).unwrap());
}

#[bench]
fn bench_maze_11_u32(b: &mut Bencher) {

    b.iter(|| large_rect_grid(RowLength(11), ColumnLength(11)).unwrap());
}

#[bench]
fn bench_maze_128_u16(b: &mut Bencher) {

    b.iter(|| medium_rect_grid(RowLength(128), ColumnLength(128)).unwrap());
}

#[bench]
fn bench_maze_128_u32(b: &mut Bencher) {

    b.iter(|| large_rect_grid(RowLength(128), ColumnLength(128)).unwrap());
}

#[bench]
fn bench_maze_500(b: &mut Bencher) {

    b.iter(|| large_rect_grid(RowLength(500), ColumnLength(500)).unwrap());
}

#[bench]
fn bench_index_to_gridcoordinate(b: &mut Bencher) {
    let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
    let dims = g.dimensions();

    b.iter(|| Cartesian2DCoordinate::from_row_major_index(93, dims));
}

#[bench]
fn bench_neighbours_corner_of_grid(b: &mut Bencher) {
    let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
    let corner = Cartesian2DCoordinate::new(0, 0);
    b.iter(|| g.neighbours(corner));
}

#[bench]
fn bench_neighbours_middle_of_grid(b: &mut Bencher) {
    let g = large_rect_grid(RowLength(11), ColumnLength(11)).unwrap();
    let mid = Cartesian2DCoordinate::new(5, 5);
    b.iter(|| g.neighbours(mid));
}
