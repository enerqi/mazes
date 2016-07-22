use std::cmp;

use grid_traits::GridDimensions;
use units::{ColumnIndex, ColumnLength, ColumnsCount, EdgesCount, NodesCount, RowIndex, RowLength,
            RowsCount};


#[derive(Debug, Copy, Clone)]
pub struct RectGridDimensions {
    row_width: RowLength,
    column_height: ColumnLength,
}

impl RectGridDimensions {
    pub fn new(row_width: RowLength, column_height: ColumnLength) -> RectGridDimensions {
        RectGridDimensions {
            row_width: row_width,
            column_height: column_height,
        }
    }
}

impl GridDimensions for RectGridDimensions {

    #[inline(always)]
    fn size(&self) -> NodesCount {
        NodesCount(self.row_width.0 * self.column_height.0)
    }

    #[inline(always)]
    fn rows(&self) -> RowsCount {
        RowsCount(self.column_height.0)
    }

    #[inline(always)]
    fn row_length(&self, _: Option<RowIndex>) -> Option<RowLength> {
        Some(self.row_width)
    }

    #[inline(always)]
    fn columns(&self) -> ColumnsCount {
        ColumnsCount(self.row_width.0)
    }

    #[inline(always)]
    fn column_length(&self, _: Option<ColumnIndex>) -> ColumnLength {
        self.column_height
    }

    fn graph_size(&self) -> (NodesCount, EdgesCount) {
        let cells_count = self.size();
        let edges_count_hint = 4 * cells_count.0 -
                               4 * cmp::max(self.row_width.0, self.column_height.0);
        (cells_count, EdgesCount(edges_count_hint))
    }
}
