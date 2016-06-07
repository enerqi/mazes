use std::fmt;
use std::rc::Rc;

use petgraph::{Graph, Undirected};
use petgraph::graph;
pub use petgraph::graph::IndexType;
use rand::Rng;
use smallvec::SmallVec;

use coordinates::{Cell, Coordinate, SquareCell, Cartesian2DCoordinate};

// refactors
//
// mask is_masked
//   convert to cartesian2d
//
// pub fn neighbours(&self, coord: Cartesian2DCoordinate) -> CoordinateSmallVec
//  handle different type of Coordinate -> 2D, Polar etc.
//  only a part of the Grid impl to bounds check the offset neighbours
//
// coordinate smallvecs? More generic return types? SmallVec<[T, 4]> etc?
// maybe make an associated type of some trait like coordinate
//
// GridCoordinate -> Rename CartesianCoordinate
// + PolarCoordinate
// trait Coordinate
//    type Small???
// Coordinate + Directions == Grid Specific Coordinate?? No more like Grid Specific Cell - HexCell
// e.g hex coord is still a cartesian2d coord but it has 6 sides - north, south, north-east, north-west, south-east, south-west.
// but for Triangle Nodes we need to know if upright or not, unless the coord was not in the centre of the triangle, e.g. bottom left, but
// triangles have to alternate facing up or down.... so coord + upright boolean...which means that the directions vary per triangle -> it
// can have 4 directions, some that do not always apply.
// offset coordinate therefore really needs to be a part of the Coordinate Trait? or rather "all offsets"
// direction would need to be a generic type...
//
// Can we have varying sorts of grids? - mixed hex, triangle, square, etc?
// uniform tilings, or Wythoff’s construction, or even Voronoi diagrams
//
// pub trait Cell => type Coordinate, type Directions, fn neighbours_not_bounds_checked
//
// so HexGrid is grid with HexCell and rows * cols
//
// neighbours_at_directions
//   direction - does that make sense for each coordinate system?
//   only a part of the Grid impl to bounds check the offset neighbours
//
// is_neighbour_linked
//   same reliance on direction concept
//
// grid_coordinate_to_index
//   important for node indices, need per coordinate type mapping?
//
// is_valid_coordinate
//   grid instance specific, needs generic info from the coordinate
//
// index to grid coordinate
//   coordinate type specific tranformation given a dimension
//
// CellIter
//
// Prepare Grid - varies -> Polar vs Other etc.
// Random cell_coord - select value from each dimension and form grid index from it
// Drawing
//
// point of coordinate?
// batch iteration in 2 dimensions
// conversion to graph index (which is really a 1D concept)
// bounds check on 2 dimensions (could be 1+ I suppose) -> graph could store the limit of each dimension, WxHxD etc.
// Does polar fit in with this idea? The polar grid tends to try to create areas of roughly the same size, so the
// number of cells increases at each outer layer of the circle


pub type CoordinateSmallVec = SmallVec<[Cartesian2DCoordinate; 4]>;
pub type CoordinateOptionSmallVec = SmallVec<[Option<Cartesian2DCoordinate>; 4]>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GridDirection {
    North,
    South,
    East,
    West,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CellLinkError {
    InvalidGridCoordinate,
    SelfLink,
}

pub trait GridDisplay {
    /// Render the contents of a grid cell as text.
    /// The String should be 3 glyphs long, padded if required.
    fn render_cell_body(&self, _: Cartesian2DCoordinate) -> String {
        String::from("   ")
    }
}

pub struct SquareGrid<GridIndexType: IndexType> { //, CellType: Cell=SquareCell
    graph: Graph<(), (), Undirected, GridIndexType>,
    dimension_size: u32,
    grid_display: Option<Rc<GridDisplay>>,
}

                                              // Note we do not need the Cell trait for this function
impl<GridIndexType: IndexType> fmt::Debug for SquareGrid<GridIndexType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SquareGrid {:?} {:?}", self.graph, self.dimension_size)
    }
}

//impl<GridIndexType: IndexType, CellType: Cell=SquareCell> SquareGrid<GridIndexType> {
impl<GridIndexType: IndexType> SquareGrid<GridIndexType> {
    pub fn new(dimension_size: u32) -> SquareGrid<GridIndexType> {

        let dim_size = dimension_size as usize;
        let cells_count = dim_size * dim_size;
        let nodes_count_hint = cells_count;
        let edges_count_hint = 4 * cells_count - 4 * dim_size; // Probably overkill, but don't want any capacity panics

        let mut grid = SquareGrid {
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
            dimension_size: dimension_size,
            grid_display: None,
        };
        for _ in 0..cells_count {
            let _ = grid.graph.add_node(());
        }

        grid
    }

    pub fn set_grid_display(&mut self, grid_display: Option<Rc<GridDisplay>>) {
        self.grid_display = grid_display;
    }

    pub fn size(&self) -> usize {
        self.dimension_size as usize * self.dimension_size as usize
    }

    #[inline]
    pub fn dimension(&self) -> u32 {
        self.dimension_size
    }

    pub fn random_cell<R: Rng>(&self, rng: &mut R) -> Cartesian2DCoordinate {
        let index = rng.gen::<usize>() % self.size();
        index_to_grid_coordinate(self.dimension_size, index)
    }

    /// Link two cells
    ///
    /// Todo - only allow links between adjacent cells? If `b` not in `g.neighbours(a)`.
    ///      - better to change the API to take an index and GridDirection
    ///
    /// Panics if a cell does not exist.
    pub fn link(&mut self, a: Cartesian2DCoordinate, b: Cartesian2DCoordinate) -> Result<(), CellLinkError> {
        if a == b {
            Err(CellLinkError::SelfLink)
        } else {
            let a_index_opt = self.grid_coordinate_graph_index(a);
            let b_index_opt = self.grid_coordinate_graph_index(b);
            match (a_index_opt, b_index_opt) {
                (Some(a_index), Some(b_index)) => {
                    let _ = self.graph.update_edge(a_index, b_index, ());
                    Ok(())
                }
                _ => Err(CellLinkError::InvalidGridCoordinate),
            }
        }
    }

    /// Unlink two cells, if the grid coordinates are valid and a link exists between them.
    /// Returns true if an unlink occurred.
    pub fn unlink(&mut self, a: Cartesian2DCoordinate, b: Cartesian2DCoordinate) -> bool {
        let a_index_opt = self.grid_coordinate_graph_index(a);
        let b_index_opt = self.grid_coordinate_graph_index(b);

        if let (Some(a_index), Some(b_index)) = (a_index_opt, b_index_opt) {
            if let Some(edge_index) = self.graph.find_edge(a_index, b_index) {
                // This will invalidate the last edge index in the graph, which is fine as we
                // are not storing them for any reason.
                self.graph.remove_edge(edge_index);
                return true;
            }
        }

        false
    }

    /// Cell nodes that are linked to a particular node by a passage.
    pub fn links(&self, coord: Cartesian2DCoordinate) -> Option<CoordinateSmallVec> {

        if let Some(graph_node_index) = self.grid_coordinate_graph_index(coord) {

            let linked_cells = self.graph
                .edges(graph_node_index)
                .map(|index_edge_data_pair| {
                    let grid_node_index = index_edge_data_pair.0;
                    index_to_grid_coordinate(self.dimension_size, grid_node_index.index())
                })
                .collect();
            Some(linked_cells)
        } else {
            None
        }
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    pub fn neighbours<CellType: Cell>(&self, coord: CellType::Coord) -> CellType::CoordinateFixedSizeVec {

        let all_dirs: CellType::DirectionFixedSizeVec = CellType::offset_directions(&Some(coord));
        let offsets: CellType::CoordinateOptionFixedSizeVec = all_dirs.into_iter()
                                                                 .map(|dir| Cell::offset_coordinate(coord, dir))
                                                                 .collect();
       //CellType::CoordinateFixedSizeVec::new()
      // CoordinateSmallVec::new()
        // [offset_coordinate(coord, GridDirection::North),
        //  offset_coordinate(coord, GridDirection::South),
        //  offset_coordinate(coord, GridDirection::East),
        //  offset_coordinate(coord, GridDirection::West)]
        offsets
            .into_iter()
            //.map(|dir| Cell::offset_coordinate(coord, dir))
            .filter(|adjacent_coord_opt| {
                if let Some(adjacent_coord) = adjacent_coord_opt {
                    self.is_valid_coordinate(adjacent_coord.as_cartesian_2d())
                } else {
                    false
                }
            })
                // no unwrap defined by the trait itself `CoordinateOptionFixedSizeVec`
                // need custom trait for unwrap/extract e.g. Monad::run ?
                // why do I want to unwrap? I really want to map it from Option to Coord directly.
                // so Option<Coord> to Coord. Not that Option<Coord> is explicit in the trait.
            .map(|some_valid_coord| some_valid_coord.unwrap())
            .collect()
    }

    pub fn neighbours_at_directions(&self,
                                    coord: Cartesian2DCoordinate,
                                    dirs: &[GridDirection])
                                    -> CoordinateOptionSmallVec {
        dirs.iter()
            .map(|direction| self.neighbour_at_direction(coord, *direction))
            .collect()
    }

    pub fn neighbour_at_direction<CellType: Cell>(&self,
                                  coord: CellType::Coord,
                                  direction: CellType::Direction)
                                  -> Option<CellType::Coord> {
        let neighbour_coord_opt = Cell::offset_coordinate(coord, direction);

        neighbour_coord_opt.and_then(|neighbour_coord: CellType::Coord| {
            if self.is_valid_coordinate(neighbour_coord.as_cartesian_2d()) {
                Some(neighbour_coord)
            } else {
                None
            }
        })
    }

    /// Are two cells in the grid linked?
    pub fn is_linked(&self, a: Cartesian2DCoordinate, b: Cartesian2DCoordinate) -> bool {
        let a_index_opt = self.grid_coordinate_graph_index(a);
        let b_index_opt = self.grid_coordinate_graph_index(b);
        if let (Some(a_index), Some(b_index)) = (a_index_opt, b_index_opt) {
            self.graph.find_edge(a_index, b_index).is_some()
        } else {
            false
        }
    }

    pub fn is_neighbour_linked(&self, coord: Cartesian2DCoordinate, direction: GridDirection) -> bool {
        self.neighbour_at_direction(coord, direction)
            .map_or(false,
                    |neighbour_coord| self.is_linked(coord, neighbour_coord))
    }

    /// Convert a grid coordinate to a one dimensional index in the range 0...grid.size().
    /// Returns None if the grid coordinate is invalid.
    pub fn grid_coordinate_to_index(&self, coord: Cartesian2DCoordinate) -> Option<usize> {
        if self.is_valid_coordinate(coord) {
            Some((coord.y as usize * self.dimension_size as usize) + coord.x as usize)
        } else {
            None
        }
    }

    pub fn iter(&self) -> CellIter {
        let dim_size = self.dimension_size;
        CellIter {
            current_cell_number: 0,
            dimension_size: dim_size,
            cells_count: self.size(),
        }
    }

    pub fn iter_row(&self) -> BatchIter {
        BatchIter {
            iter_type: BatchIterType::Row,
            current_index: 0,
            dimension_size: self.dimension_size,
        }
    }

    pub fn iter_column(&self) -> BatchIter {
        BatchIter {
            iter_type: BatchIterType::Column,
            current_index: 0,
            dimension_size: self.dimension_size,
        }
    }

    /// Is the grid coordinate valid for this grid - within the grid's dimensions
    pub fn is_valid_coordinate(&self, coord: Cartesian2DCoordinate) -> bool {
        coord.x < self.dimension_size && coord.y < self.dimension_size
    }

    fn is_neighbour<CellType: Cell>(&self, a: CellType::Coord, b: CellType::Coord) -> bool {
        self.neighbours(a).iter().any(|&coord| coord == b)
    }

    /// Convert a grid coordinate into petgraph nodeindex
    /// Returns None if the grid coordinate is invalid (out of the grid's dimensions).
    fn grid_coordinate_graph_index(&self,
                                   coord: Cartesian2DCoordinate)
                                   -> Option<graph::NodeIndex<GridIndexType>> {
        let grid_index_raw_opt = self.grid_coordinate_to_index(coord);
        grid_index_raw_opt.map(graph::NodeIndex::<GridIndexType>::new)
    }
}

//impl<GridIndexType: IndexType, CellType: Cell> fmt::Display for SquareGrid<GridIndexType, CellType> {
impl<GridIndexType: IndexType> fmt::Display for SquareGrid<GridIndexType> {
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

        let columns_count = self.dimension_size;
        let rows_count = columns_count;

        // Start by special case rendering the text for the north most boundary
        let first_grid_row: &Vec<Cartesian2DCoordinate> =
            &self.iter_row().take(1).collect::<Vec<Vec<_>>>()[0];
        let mut output = String::from(WALL_RD);
        for (index, coord) in first_grid_row.iter().enumerate() {
            output.push_str(WALL_LR_3);
            let is_east_open = self.is_neighbour_linked(*coord, GridDirection::East);
            if is_east_open {
                output.push_str(WALL_LR);
            } else {
                let is_last_cell = index == (columns_count - 1) as usize;
                if is_last_cell {
                    output.push_str(WALL_LD);
                } else {
                    output.push_str(WALL_LRD);
                }
            }
        }
        output.push_str("\n");

        for (index_row, row) in self.iter_row().enumerate() {

            let is_last_row = index_row == (rows_count - 1) as usize;

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
                let is_last_column = index_column == (columns_count - 1) as usize;
                let east_open = self.is_neighbour_linked(cell_coord, GridDirection::East);
                let south_open = self.is_neighbour_linked(cell_coord, GridDirection::South);

                // Each cell will simply use the southern wall of the cell above
                // it as its own northern wall, so we only need to worry about the cell’s body (room space),
                // its eastern boundary ('|'), and its southern boundary ('---+') minus the south west corner.
                let east_boundary = render_cell_side(GridDirection::East, " ", WALL_UD);

                // Cell Body
                if let Some(ref displayer) = self.grid_display {
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
                let south_boundary = render_cell_side(GridDirection::South, "   ", WALL_LR_3);
                row_bottom_section_render.push_str(south_boundary);

                let corner = match (is_last_row, is_last_column) {
                    (true, true) => WALL_LU,
                    (true, false) => {
                        if east_open {
                            WALL_LR
                        } else {
                            WALL_LRU
                        }
                    }
                    (false, true) => {
                        if south_open {
                            WALL_UD
                        } else {
                            WALL_LUD
                        }
                    }
                    (false, false) => {
                        let access_se_from_east =
                            self.neighbour_at_direction(cell_coord, GridDirection::East)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, GridDirection::South));
                        let access_se_from_south =
                            self.neighbour_at_direction(cell_coord, GridDirection::South)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(c, GridDirection::East));
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

#[derive(Debug, Copy, Clone)]
pub struct CellIter {
    current_cell_number: usize,
    dimension_size: u32,
    cells_count: usize,
}
impl Iterator for CellIter {
    type Item = Cartesian2DCoordinate;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cell_number < self.cells_count {
            let coord = index_to_grid_coordinate(self.dimension_size, self.current_cell_number);
            self.current_cell_number += 1;
            Some(coord)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = self.cells_count - self.current_cell_number;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}

// Converting the Grid into an iterator (CellIter - the default most sensible)
// This form is useful if you have the SquareGrid by value and take a reference to it
// but seems unhelpful when you already have a reference then we need to do &*grid which
// it just plain uglier than `grid.iter()`
// impl<'a, GridIndexType: IndexType, CellType: Cell> IntoIterator for &'a SquareGrid<GridIndexType, CellType> {
//     type Item = CellType::Coord;
//     type IntoIter = CellIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

#[derive(Debug, Copy, Clone)]
enum BatchIterType {
    Row,
    Column,
}
#[derive(Debug, Copy, Clone)]
pub struct BatchIter {
    iter_type: BatchIterType,
    current_index: u32,
    dimension_size: u32,
}
impl Iterator for BatchIter {
    type Item = Vec<Cartesian2DCoordinate>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.dimension_size {
            let coords = (0..self.dimension_size)
                .into_iter()
                .map(|i: u32| {
                    if let BatchIterType::Row = self.iter_type {
                        Cartesian2DCoordinate::new(i, self.current_index)
                    } else {
                        Cartesian2DCoordinate::new(self.current_index, i)
                    }
                })
                .collect();
            self.current_index += 1;
            Some(coords)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = (self.dimension_size - self.current_index) as usize;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}

pub fn index_to_grid_coordinate(dimension_size: u32,
                                one_dimensional_index: usize)
                                -> Cartesian2DCoordinate {
    let x = one_dimensional_index % dimension_size as usize;
    let y = one_dimensional_index / dimension_size as usize;
    Cartesian2DCoordinate {
        x: x as u32,
        y: y as u32,
    }
}

/// Create a new `Cartesian2DCoordinate` offset 1 cell away in the given direction.
/// Returns None if the Coordinate is not representable (x < 0 or y < 0).
fn offset_coordinate(coord: Cartesian2DCoordinate, dir: GridDirection) -> Option<Cartesian2DCoordinate> {
    let (x, y) = (coord.x, coord.y);
    match dir {
        GridDirection::North => {
            if y > 0 {
                Some(Cartesian2DCoordinate { x: x, y: y - 1 })
            } else {
                None
            }
        }
        GridDirection::South => Some(Cartesian2DCoordinate { x: x, y: y + 1 }),
        GridDirection::East => Some(Cartesian2DCoordinate { x: x + 1, y: y }),
        GridDirection::West => {
            if x > 0 {
                Some(Cartesian2DCoordinate { x: x - 1, y: y })
            } else {
                None
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use coordinates::Cartesian2DCoordinate;
    use itertools::Itertools; // a trait
    use rand;
    use smallvec::SmallVec;
    use std::u32;

    type SmallGrid<'a> = SquareGrid<u8>;

    // Compare a smallvec to e.g. a vec! or &[T].
    // SmallVec really ruins the syntax ergonomics, hence this macro
    // The compiler often succeeds in automatically adding the correct & and derefs (*) but not here
    macro_rules! assert_smallvec_eq {
        ($x:expr, $y:expr) => (assert_eq!(&*$x, &*$y))
    }

    #[test]
    fn neighbour_cells() {
        let g = SmallGrid::new(10);

        let check_expected_neighbours = |coord, expected_neighbours: &[Cartesian2DCoordinate]| {
            let node_indices: Vec<Cartesian2DCoordinate> = g.neighbours(coord).iter().cloned().sorted();
            let expected_indices: Vec<Cartesian2DCoordinate> = expected_neighbours.into_iter()
                .cloned()
                .sorted();
            assert_eq!(node_indices, expected_indices);
        };
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);

        // corners
        check_expected_neighbours(gc(0, 0), &[gc(1, 0), gc(0, 1)]);
        check_expected_neighbours(gc(9, 0), &[gc(8, 0), gc(9, 1)]);
        check_expected_neighbours(gc(0, 9), &[gc(0, 8), gc(1, 9)]);
        check_expected_neighbours(gc(9, 9), &[gc(9, 8), gc(8, 9)]);

        // side element examples
        check_expected_neighbours(gc(1, 0), &[gc(0, 0), gc(1, 1), gc(2, 0)]);
        check_expected_neighbours(gc(0, 1), &[gc(0, 0), gc(0, 2), gc(1, 1)]);
        check_expected_neighbours(gc(0, 8), &[gc(1, 8), gc(0, 7), gc(0, 9)]);
        check_expected_neighbours(gc(9, 8), &[gc(9, 7), gc(9, 9), gc(8, 8)]);

        // Some place with 4 neighbours inside the grid
        check_expected_neighbours(gc(1, 1), &[gc(0, 1), gc(1, 0), gc(2, 1), gc(1, 2)]);
    }

    #[test]
    fn neighbours_at_dirs() {
        let g = SmallGrid::new(2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);

        let check_neighbours =
            |coord, dirs: &[GridDirection], neighbour_opts: &[Option<Cartesian2DCoordinate>]| {

                let neighbour_options: CoordinateOptionSmallVec =
                    g.neighbours_at_directions(coord, dirs);
                assert_eq!(&*neighbour_options, neighbour_opts);
            };
        check_neighbours(gc(0, 0), &[], &[]);
        check_neighbours(gc(0, 0), &[GridDirection::North], &[None]);
        check_neighbours(gc(0, 0), &[GridDirection::West], &[None]);
        check_neighbours(gc(0, 0),
                         &[GridDirection::West, GridDirection::North],
                         &[None, None]);
        check_neighbours(gc(0, 0),
                         &[GridDirection::East, GridDirection::South],
                         &[Some(gc(1, 0)), Some(gc(0, 1))]);

        check_neighbours(gc(1, 1), &[], &[]);
        check_neighbours(gc(1, 1), &[GridDirection::South], &[None]);
        check_neighbours(gc(1, 1), &[GridDirection::East], &[None]);
        check_neighbours(gc(1, 1),
                         &[GridDirection::South, GridDirection::East],
                         &[None, None]);
        check_neighbours(gc(1, 1),
                         &[GridDirection::West, GridDirection::North],
                         &[Some(gc(0, 1)), Some(gc(1, 0))]);
    }

    #[test]
    fn neighbour_at_dir() {
        let g = SmallGrid::new(2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);
        let check_neighbour = |coord, dir: GridDirection, expected| {
            assert_eq!(g.neighbour_at_direction(coord, dir), expected);
        };
        check_neighbour(gc(0, 0), GridDirection::North, None);
        check_neighbour(gc(0, 0), GridDirection::South, Some(gc(0, 1)));
        check_neighbour(gc(0, 0), GridDirection::East, Some(gc(1, 0)));
        check_neighbour(gc(0, 0), GridDirection::West, None);

        check_neighbour(gc(1, 1), GridDirection::North, Some(gc(1, 0)));
        check_neighbour(gc(1, 1), GridDirection::South, None);
        check_neighbour(gc(1, 1), GridDirection::East, None);
        check_neighbour(gc(1, 1), GridDirection::West, Some(gc(0, 1)));
    }

    #[test]
    fn grid_size() {
        let g = SmallGrid::new(10);
        assert_eq!(g.size(), 100);
    }

    #[test]
    fn grid_dimension() {
        let g = SmallGrid::new(10);
        assert_eq!(g.dimension(), 10);
    }

    #[test]
    fn grid_coordinate_as_index() {
        let g = SmallGrid::new(3);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);
        let coords = &[gc(0, 0), gc(1, 0), gc(2, 0), gc(0, 1), gc(1, 1), gc(2, 1), gc(0, 2),
                       gc(1, 2), gc(2, 2)];
        let indices: Vec<Option<usize>> = coords.into_iter()
            .map(|coord| g.grid_coordinate_to_index(*coord))
            .collect();
        let expected = (0..9).map(|n| Some(n)).collect::<Vec<Option<usize>>>();
        assert_eq!(expected, indices);

        assert_eq!(g.grid_coordinate_to_index(gc(2, 3)), None);
        assert_eq!(g.grid_coordinate_to_index(gc(3, 2)), None);
        assert_eq!(g.grid_coordinate_to_index(gc(u32::MAX, u32::MAX)), None);
    }

    #[test]
    fn random_cell() {
        let g = SmallGrid::new(4);
        let cells_count = 4 * 4;
        let mut rng = rand::weak_rng();
        for _ in 0..1000 {
            let coord = g.random_cell(&mut rng);
            assert!(coord.x < cells_count);
            assert!(coord.y < cells_count);
        }
    }

    #[test]
    fn cell_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter().collect::<Vec<Cartesian2DCoordinate>>(),
                   &[Cartesian2DCoordinate::new(0, 0),
                     Cartesian2DCoordinate::new(1, 0),
                     Cartesian2DCoordinate::new(0, 1),
                     Cartesian2DCoordinate::new(1, 1)]);
    }

    #[test]
    fn row_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_row().collect::<Vec<Vec<Cartesian2DCoordinate>>>(),
                   &[&[Cartesian2DCoordinate::new(0, 0), Cartesian2DCoordinate::new(1, 0)],
                     &[Cartesian2DCoordinate::new(0, 1), Cartesian2DCoordinate::new(1, 1)]]);
    }

    #[test]
    fn column_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_column().collect::<Vec<Vec<Cartesian2DCoordinate>>>(),
                   &[&[Cartesian2DCoordinate::new(0, 0), Cartesian2DCoordinate::new(0, 1)],
                     &[Cartesian2DCoordinate::new(1, 0), Cartesian2DCoordinate::new(1, 1)]]);
    }

    #[test]
    fn linking_cells() {
        let mut g = SmallGrid::new(4);
        let a = Cartesian2DCoordinate::new(0, 1);
        let b = Cartesian2DCoordinate::new(0, 2);
        let c = Cartesian2DCoordinate::new(0, 3);

        // Testing the expected grid `links`
        let sorted_links = |grid: &SmallGrid, coord| -> Vec<Cartesian2DCoordinate> {
            grid.links(coord).expect("coordinate is invalid").iter().cloned().sorted()
        };
        macro_rules! links_sorted {
            ($x:expr) => (sorted_links(&g, $x))
        }

        // Testing that the order of the arguments to `is_linked` does not matter
        macro_rules! bi_check_linked {
            ($x:expr, $y:expr) => (g.is_linked($x, $y) && g.is_linked($y, $x))
        }

        // Testing `is_neighbour_linked` for all directions
        let all_dirs =
            [GridDirection::North, GridDirection::South, GridDirection::East, GridDirection::West];

        let directional_links_check =
            |grid: &SmallGrid, coord: Cartesian2DCoordinate, expected_dirs_linked: &[GridDirection]| {

                let expected_complement: SmallVec<[GridDirection; 4]> = all_dirs.iter()
                    .cloned()
                    .filter(|dir: &GridDirection| !expected_dirs_linked.contains(dir))
                    .collect();
                for exp_dir in expected_dirs_linked {
                    assert!(grid.is_neighbour_linked(coord, *exp_dir));
                }
                for not_exp_dir in expected_complement.iter() {
                    assert!(!grid.is_neighbour_linked(coord, *not_exp_dir));
                }
            };
        macro_rules! check_directional_links {
            ($coord:expr, $expected:expr) => (directional_links_check(&g, $coord, &$expected))
        }

        // a, b and c start with no links
        assert!(!bi_check_linked!(a, b));
        assert!(!bi_check_linked!(a, c));
        assert!(!bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);
        check_directional_links!(a, []);
        check_directional_links!(b, []);
        check_directional_links!(c, []);

        g.link(a, b).expect("link failed");
        // a - b linked bi-directionally
        assert!(bi_check_linked!(a, b));
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a]);
        check_directional_links!(a, [GridDirection::South]);
        check_directional_links!(b, [GridDirection::North]);
        check_directional_links!(c, []);

        g.link(b, c).expect("link failed");
        // a - b still linked bi-directionally after linking b - c
        // b linked to a & c bi-directionally
        // c linked to b bi-directionally
        assert!(bi_check_linked!(a, b));
        assert!(bi_check_linked!(b, c));
        assert!(!bi_check_linked!(a, c));
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a, c]);
        assert_eq!(links_sorted!(c), vec![b]);

        // a - b still linked bi-directionally after updating exist link
        assert!(bi_check_linked!(a, b));
        assert!(bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a, c]);

        check_directional_links!(a, [GridDirection::South]);
        check_directional_links!(b, [GridDirection::North, GridDirection::South]);
        check_directional_links!(c, [GridDirection::North]);

        // a - b unlinked
        // b still linked to c bi-directionally
        let is_ab_unlinked = g.unlink(a, b);
        assert!(is_ab_unlinked);
        assert!(!bi_check_linked!(a, b));
        assert!(bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![c]);
        assert_eq!(links_sorted!(c), vec![b]);
        check_directional_links!(a, []);
        check_directional_links!(b, [GridDirection::South]);
        check_directional_links!(c, [GridDirection::North]);

        // a, b and c start all unlinked again
        let is_bc_unlinked = g.unlink(b, c);
        assert!(is_bc_unlinked);
        assert!(!bi_check_linked!(a, b));
        assert!(!bi_check_linked!(a, c));
        assert!(!bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);
        check_directional_links!(a, []);
        check_directional_links!(b, []);
        check_directional_links!(c, []);
    }

    #[test]
    fn no_self_linked_cycles() {
        let mut g = SmallGrid::new(4);
        let a = Cartesian2DCoordinate::new(0, 0);
        let link_result = g.link(a, a);
        assert_eq!(link_result, Err(CellLinkError::SelfLink));
    }

    #[test]
    fn no_links_to_invalid_coordinates() {
        let mut g = SmallGrid::new(4);
        let good_coord = Cartesian2DCoordinate::new(0, 0);
        let invalid_coord = Cartesian2DCoordinate::new(100, 100);
        let link_result = g.link(good_coord, invalid_coord);
        assert_eq!(link_result, Err(CellLinkError::InvalidGridCoordinate));
    }

    #[test]
    fn no_parallel_duplicated_linked_cells() {
        let mut g = SmallGrid::new(4);
        let a = Cartesian2DCoordinate::new(0, 0);
        let b = Cartesian2DCoordinate::new(0, 1);
        g.link(a, b).expect("link failed");
        g.link(a, b).expect("link failed");
        assert_smallvec_eq!(g.links(a).unwrap(), &[b]);
        assert_smallvec_eq!(g.links(b).unwrap(), &[a]);

        g.unlink(a, b);
        assert_smallvec_eq!(g.links(a).unwrap(), &[]);
        assert_smallvec_eq!(g.links(b).unwrap(), &[]);
    }
}
