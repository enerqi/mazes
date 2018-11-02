use crate::{
    cells::{Cell, Coordinate},
    units::{ColumnIndex, ColumnLength, ColumnsCount, EdgesCount, NodesCount, RowIndex, RowLength, RowsCount}
};

use rand::XorShiftRng;
use std::rc::Rc;


pub trait GridDimensions {
    fn size(&self) -> NodesCount;
    fn rows(&self) -> RowsCount;
    fn row_length(&self, row_index: Option<RowIndex>) -> Option<RowLength>;
    fn columns(&self) -> ColumnsCount;
    fn column_length(&self, column_index: Option<ColumnIndex>) -> ColumnLength;
    fn graph_size(&self) -> (NodesCount, EdgesCount);
    fn nodes_count_up_to(&self, row_index: RowIndex) -> Option<NodesCount>;
}

pub trait GridCoordinates<CellT: Cell> {
    fn grid_coordinate_to_index(&self,
                                coord: CellT::Coord,
                                dimensions: &Rc<GridDimensions>)
                                -> Option<usize>;
    fn is_valid_coordinate(&self, coord: CellT::Coord, dimensions: &Rc<GridDimensions>) -> bool {

        let grid_2d_coord = coord.as_cartesian_2d();
        let RowLength(width) = dimensions
            .row_length(Some(RowIndex(grid_2d_coord.y as usize)))
            .expect("RowIndex invalid");
        let ColumnLength(height) =
            dimensions.column_length(Some(ColumnIndex(grid_2d_coord.x as usize)));
        (grid_2d_coord.x as usize) < width && (grid_2d_coord.y as usize) < height
    }
    fn random_cell(&self, rng: &mut XorShiftRng, dimensions: &Rc<GridDimensions>) -> CellT::Coord; // consider &Rng simple trait object. Note <R : Rng> meant GridCoordinates could not be made a trait object
}

pub trait GridIterators<CellT: Cell> {
    type CellIter: Iterator<Item = CellT::Coord>;
    type BatchIter: Iterator<Item = Vec<CellT::Coord>>; // consider &[CellT::Coord] instead
    fn iter(&self, dimensions: &Rc<GridDimensions>) -> Self::CellIter;
    fn iter_row(&self, dimensions: &Rc<GridDimensions>) -> Self::BatchIter;
    fn iter_column(&self, dimensions: &Rc<GridDimensions>) -> Self::BatchIter;
}

pub trait GridDisplay<CellT: Cell> {
    /// Render the contents of a grid cell as text.
    /// The String should be 3 glyphs long, padded if required.
    fn render_cell_body(&self, _: CellT::Coord) -> String {
        String::from("   ")
    }
}
