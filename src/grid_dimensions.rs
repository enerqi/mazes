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

    fn nodes_count_up_to(&self, row_index: RowIndex) -> Option<NodesCount> {
        let RowIndex(row) = row_index;
        let RowLength(len) = self.row_width;
        Some(NodesCount(row * len))
    }
}

#[derive(Debug, Clone)]
pub struct PolarGridDimensions {
    row_cell_counts: Vec<usize>,
    per_row_cumulative_node_count: Vec<NodesCount>,
    rows: RowsCount, // height (y coord) of the grid
    size: NodesCount
}

impl PolarGridDimensions {
    pub fn new(rows: RowsCount) -> PolarGridDimensions {

        let RowsCount(row_count) = rows;
        let mut cell_counts = Vec::with_capacity(row_count);

        use std::f32::consts::PI;

        // working with a unit circle that can be scaled later
        let row_height = 1.0 / row_count as f32;
        // The circle centre with one cell only that cannot be accessed.
        cell_counts[0] = 1;

        for y in 1..row_count {

            // radius of how far from centre the row inner boundary is
            let inner_radius = y as f32 * row_height;

            // length of inner boundary
            let circumference = 2.0 * PI * inner_radius;

            let previous_row_cell_count: usize = cell_counts[y - 1];

            // If we were to have as many cells as the previous inner row then the
            // cells must be this wide.
            // Tells us how wide each cell would be in this row if we don’t subdivide
            // Our ideal cell width is the same as the height of the row
            let estimated_cell_width = circumference / previous_row_cell_count as f32;

            // How many ideal sized cells fit into this new row
            // Rounded up or down (1 or 2 - maybe more for row 1)
            // We subdivide if the ratio is 2+
            let ratio = (estimated_cell_width / row_height).round();

            let num_cells = previous_row_cell_count * ratio as usize;

            cell_counts[y] = num_cells;
        }

        let per_row_cumulative_node_count = cell_counts
                           .iter()
                           .scan(0, |accumulator: &mut usize, cells_in_row: &usize| {
                               *accumulator = *accumulator + cells_in_row;
                               Some(*accumulator)
                           })
                           .map(|count: usize| NodesCount(count))
                           .collect();

        let size = cell_counts.iter()
                              .cloned()
                              .fold1(|x, y| x + y)
                              .unwrap_or(0);

        PolarGridDimensions {
            row_cell_counts: cell_counts,
            per_row_cumulative_node_count: per_row_cumulative_node_count,
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
        ColumnsCount(1)
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

    fn nodes_count_up_to(&self, row_index: RowIndex) -> Option<NodesCount> {
        let RowIndex(row) = row_index;
        self.per_row_cumulative_node_count.get(row).cloned()
    }
}
