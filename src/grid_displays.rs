

use cells::{Cartesian2DCoordinate, Cell, CompassPrimary, SquareCell};
use grid::{Grid, IndexType};
use grid_traits::{GridDisplay, GridIterators};
use pathing::{Distances, MaxDistance};
use std::fmt;
use std::marker::PhantomData;
use units::{ColumnsCount, RowsCount};
use utils::FnvHashSet;


impl<CellT, MaxDistanceT> GridDisplay<CellT> for Distances<CellT, MaxDistanceT>
    where CellT: Cell,
          MaxDistanceT: MaxDistance
{
    fn render_cell_body(&self, coord: CellT::Coord) -> String {

        // In case Distances is used with a different grid check for Vec access being in bounds.
        // N.B.
        // Keeping a reference to the grid that was processed is possible, but the circular nature of distances to Grid
        // and Grid to (distances as GridDisplay) means we need Rc and Weak pointers, in particular Rc<RefCell<_>> for the
        // maze so that we could mutate it to inject the (distance as GridDisplay) and the (distance as GridDisplay) could be
        // given an Rc<_> downgraded to Weak<_> to refer to the Grid...or maybe GridDisplay holds a &Grid but that won't
        // work as the lifetime of any Rc is unknown and &Grid would need a 'static lifetime.
        // As the ref from the (distance as GridDisplay) to Grid is not &T and the Rc<RefCell> avoids static borrow check
        // rules there are no guarantees that the graph on the Grid cannot change after distances has been created.
        //
        // *Iff* a Distances were always to be created with every Grid, such that the lifetimes are the same
        // the Grid could have a RefCell<Option<&GridDisplay>> and the GridDisplay could have &Grid which would
        // freeze as immutable the graph of the Grid.

        if let Some(d) = self.distances().get(&coord) {
            // centre align, padding 3, lowercase hexadecimal
            format!("{:^3x}", d)
        } else {
            String::from("   ")
        }
    }
}


#[derive(Debug)]
pub struct PathDisplay<CellT: Cell> {
    on_path_coordinates: FnvHashSet<CellT::Coord>,
}
impl<CellT: Cell> PathDisplay<CellT> {
    pub fn new(path: &[CellT::Coord]) -> Self {
        PathDisplay { on_path_coordinates: path.iter().cloned().collect() }
    }
}
impl<CellT: Cell> GridDisplay<CellT> for PathDisplay<CellT> {
    fn render_cell_body(&self, coord: CellT::Coord) -> String {
        if self.on_path_coordinates.contains(&coord) {
            String::from(" . ")
        } else {
            String::from("   ")
        }
    }
}


#[derive(Debug)]
pub struct StartEndPointsDisplay<CellT: Cell> {
    start_coordinates: CellT::CoordinateSmallVec,
    end_coordinates: CellT::CoordinateSmallVec,
    cell_type: PhantomData<CellT>,
}
impl<CellT: Cell> StartEndPointsDisplay<CellT> {
    pub fn new(starts: CellT::CoordinateSmallVec,
               ends: CellT::CoordinateSmallVec)
               -> StartEndPointsDisplay<CellT> {
        StartEndPointsDisplay {
            start_coordinates: starts,
            end_coordinates: ends,
            cell_type: PhantomData,
        }
    }
}
impl<CellT: Cell> GridDisplay<CellT> for StartEndPointsDisplay<CellT> {
    fn render_cell_body(&self, coord: CellT::Coord) -> String {

        let contains_coordinate =
            |coordinates: &CellT::CoordinateSmallVec| coordinates.iter().any(|&c| c == coord);

        if contains_coordinate(&self.start_coordinates) {
            String::from(" S ")

        } else if contains_coordinate(&self.end_coordinates) {

            String::from(" E ")

        } else {
            String::from("   ")
        }
    }
}


// Todo - displaying other grid types, e.g. impl<GridIndexType: IndexType> fmt::Display for Grid<GridIndexType, HexCell>
impl<GridIndexType, Iters> fmt::Display for Grid<GridIndexType, SquareCell, Iters>
    where GridIndexType: IndexType,
          Iters: GridIterators<SquareCell>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const WALL_L: &'static str = "╴";
        const WALL_R: &'static str = "╶";
        const WALL_U: &'static str = "╵";
        const WALL_D: &'static str = "╷";
        const WALL_LR_3: &'static str = "───";
        const WALL_LR: &'static str = "─";
        const WALL_UD: &'static str = "│";
        const WALL_LD: &'static str = "┐";
        const WALL_RU: &'static str = "└";
        const WALL_LU: &'static str = "┘";
        const WALL_RD: &'static str = "┌";
        const WALL_LRU: &'static str = "┴";
        const WALL_LRD: &'static str = "┬";
        const WALL_LRUD: &'static str = "┼";
        const WALL_RUD: &'static str = "├";
        const WALL_LUD: &'static str = "┤";
        let default_cell_body = String::from("   ");

        let ColumnsCount(columns_count) = self.columns();
        let RowsCount(rows_count) = self.rows();

        // Start by special case rendering the text for the north most boundary
        let first_grid_row: &Vec<Cartesian2DCoordinate> =
            &self.iter_row().take(1).collect::<Vec<Vec<_>>>()[0];
        let mut output = String::from(WALL_RD);
        for (index, coord) in first_grid_row.iter().enumerate() {
            output.push_str(WALL_LR_3);
            let is_east_open = self.is_neighbour_linked(*coord, CompassPrimary::East);
            if is_east_open {
                output.push_str(WALL_LR);
            } else {
                let is_last_cell = index == (columns_count - 1);
                if is_last_cell {
                    output.push_str(WALL_LD);
                } else {
                    output.push_str(WALL_LRD);
                }
            }
        }
        output.push_str("\n");

        for (index_row, row) in self.iter_row().enumerate() {

            let is_last_row = index_row == (rows_count - 1);

            // Starts of by special case rendering the west most boundary of the row
            // The top section of the cell is done by the previous row.
            let mut row_middle_section_render = String::from(WALL_UD);
            let mut row_bottom_section_render = String::from("");

            for (index_column, cell_coord) in row.into_iter().enumerate() {

                let render_cell_side = |direction, passage_clear_text, blocking_wall_text| {
                    self.neighbour_at_direction(cell_coord, direction)
                        .map_or(blocking_wall_text, |neighbour_coord| {
                            if self.is_linked(cell_coord, neighbour_coord) {
                                passage_clear_text
                            } else {
                                blocking_wall_text
                            }
                        })
                };
                let is_first_column = index_column == 0;
                let is_last_column = index_column == (columns_count - 1);
                let east_open = self.is_neighbour_linked(cell_coord, CompassPrimary::East);
                let south_open = self.is_neighbour_linked(cell_coord, CompassPrimary::South);

                // Each cell will simply use the southern wall of the cell above
                // it as its own northern wall, so we only need to worry about the cell’s body (room space),
                // its eastern boundary ('|'), and its southern boundary ('---+') minus the south west corner.
                let east_boundary = render_cell_side(CompassPrimary::East, " ", WALL_UD);

                // Cell Body
                if let Some(ref displayer) = *self.grid_display() {
                    row_middle_section_render.push_str(displayer.render_cell_body(cell_coord)
                        .as_str());
                } else {
                    row_middle_section_render.push_str(default_cell_body.as_str());
                }

                row_middle_section_render.push_str(east_boundary);

                if is_first_column {
                    row_bottom_section_render = if is_last_row {
                        String::from(WALL_RU)
                    } else if south_open {
                        String::from(WALL_UD)
                    } else {
                        String::from(WALL_RUD)
                    };

                }
                let south_boundary = render_cell_side(CompassPrimary::South, "   ", WALL_LR_3);
                row_bottom_section_render.push_str(south_boundary);

                let corner = match (is_last_row, is_last_column) {
                    (true, true) => WALL_LU,
                    (true, false) => if east_open { WALL_LR } else { WALL_LRU },
                    (false, true) => if south_open { WALL_UD } else { WALL_LUD },
                    (false, false) => {
                        let access_se_from_east =
                            self.neighbour_at_direction(cell_coord, CompassPrimary::East)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, CompassPrimary::South));
                        let access_se_from_south =
                            self.neighbour_at_direction(cell_coord, CompassPrimary::South)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, CompassPrimary::East));
                        let show_right_section = !access_se_from_east;
                        let show_down_section = !access_se_from_south;
                        let show_up_section = !east_open;
                        let show_left_section = !south_open;

                        match (show_left_section,
                               show_right_section,
                               show_up_section,
                               show_down_section) {
                            (true, true, true, true) => WALL_LRUD,
                            (true, true, true, false) => WALL_LRU,
                            (true, true, false, true) => WALL_LRD,
                            (true, false, true, true) => WALL_LUD,
                            (false, true, true, true) => WALL_RUD,
                            (true, true, false, false) => WALL_LR,
                            (false, false, true, true) => WALL_UD,
                            (false, true, true, false) => WALL_RU,
                            (true, false, false, true) => WALL_LD,
                            (true, false, true, false) => WALL_LU,
                            (false, true, false, true) => WALL_RD,
                            (true, false, false, false) => WALL_L,
                            (false, true, false, false) => WALL_R,
                            (false, false, true, false) => WALL_U,
                            (false, false, false, true) => WALL_D,
                            _ => " ",
                        }
                    }
                };

                row_bottom_section_render.push_str(corner.as_ref());
            }

            output.push_str(row_middle_section_render.as_ref());
            output.push_str("\n");
            output.push_str(row_bottom_section_render.as_ref());
            output.push_str("\n");
        }

        write!(f, "{}", output)
    }
}
