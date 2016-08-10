use std::cmp;

use itertools::Itertools;

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

#[derive(Debug, Clone)]
pub struct PolarGridDimensions {
    row_cell_counts: Vec<usize>,
    rows: RowsCount, // height (y coord) of the grid
    size: NodesCount
}

impl PolarGridDimensions {
    pub fn new(rows: RowsCount) -> PolarGridDimensions {

        let cell_counts = Vec::with_capacity(rows.0);

        use std::f32::consts::PI;
        let row_count = rows.0;
        let row_height = 1.0 / row_count as f32;
        let mut previous_rows_length = 1.0f32; // The centre circle with one cell only.
        for y in 0..row_count {
            let radius = y as f32 * row_height;
            let circumference = 2.0 * PI * radius;

            // If we were to have as many cells as the previous inner row then the
            // cells must be this wide:
            let cell_width = circumference / previous_rows_length;
        }

        let size = cell_counts.iter()
                              .cloned()
                              .fold1(|x, y| x + y)
                              .unwrap_or(0);
        PolarGridDimensions {
            row_cell_counts: cell_counts,
            rows: rows,
            size: NodesCount(size)
        }
    }
}

impl GridDimensions for PolarGridDimensions {
    #[inline(always)]
    fn size(&self) -> NodesCount {
        self.size
    }

    #[inline(always)]
    fn rows(&self) -> RowsCount {
        self.rows
    }

    fn row_length(&self, row_index: Option<RowIndex>) -> Option<RowLength> {
        match row_index {
            Some(row) => self.row_cell_counts.get(row.0)
                                             .map(|row_len| RowLength(*row_len)),
            None => None,
        }
    }

    #[inline(always)]
    fn columns(&self) -> ColumnsCount {
        // There is no 'column' on a polar grid going all the way through from the
        // outer row to the inner centre.
        ColumnsCount(0)
    }

    #[inline(always)]
    fn column_length(&self, _: Option<ColumnIndex>) -> ColumnLength {
        ColumnLength(self.rows.0)
    }

    fn graph_size(&self) -> (NodesCount, EdgesCount) {
        let cells_count = self.size();
        let edges_count_hint = self.row_cell_counts.last()
                                                   .map(|&outer_row| outer_row * 2 * 4)
                                                   .unwrap_or(0);
        (cells_count, EdgesCount(edges_count_hint))
    }
}
