use rand::XorShiftRng;

use cells::{Cell, Coordinate, Cartesian2DCoordinate};
use units::{RowsCount, RowLength, RowIndex, ColumnsCount, ColumnLength,
            ColumnIndex, NodesCount, EdgesCount};

pub trait GridDimensions {
    fn size(&self) -> NodesCount;
    fn rows(&self) -> RowsCount;
    fn row_length(&self, rowIndex: Option<RowIndex>) -> Option<RowLength>;
    fn columns(&self) -> ColumnsCount;
    fn column_length(&self, columnIndex: Option<ColumnIndex>) -> ColumnLength;
    fn graph_size(&self) -> (NodesCount, EdgesCount);
}

pub trait GridPositions<CellT: Cell> {
    fn grid_coordinate_to_index(&self, coord: CellT::Coord) -> Option<usize>;
    fn random_cell(&self, rng: &mut XorShiftRng) -> CellT::Coord; // consider &Rng simple trait object. Note <R : Rng> meant GridPositions could not be made a trait object   
}

pub trait GridIterators<CellT: Cell, Dimensions: GridDimensions> {
    type CellIter: Iterator<Item=CellT::Coord>;
    type BatchIter: Iterator<Item=Vec<CellT::Coord>>; // consider &[CellT::Coord] instead
    fn iter(&self, data: &Dimensions) -> Self::CellIter;
    fn iter_row(&self, data: &Dimensions) -> Self::BatchIter;
    fn iter_column(&self, data: &Dimensions) -> Self::BatchIter;
}

pub trait GridDisplay<CellT: Cell> {
    /// Render the contents of a grid cell as text.
    /// The String should be 3 glyphs long, padded if required.
    fn render_cell_body(&self, _: CellT::Coord) -> String {
        String::from("   ")
    }
}
