use petgraph::{Graph, Undirected};
use petgraph::graph;
use petgraph::graph::IndexType; // Todo pub use this indextype?
use rand;
use rand::Rng;
use smallvec::SmallVec;
use std::fmt;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct GridCoordinate {
    pub x: isize,
    pub y: isize,
}
impl GridCoordinate {
    pub fn new(x: isize, y: isize) -> GridCoordinate {
        GridCoordinate { x: x, y: y }
    }
}
pub type CoordinateSmallVec = SmallVec<[GridCoordinate; 4]>;
pub type CoordinateOptionSmallVec = SmallVec<[Option<GridCoordinate>; 4]>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GridDirection {
    North,
    South,
    East,
    West,
}

#[derive(Debug)]
pub struct SquareGrid<GridIndexType: IndexType> {
    graph: Graph<(), (), Undirected, GridIndexType>,
    dimension_size: GridIndexType,
}

impl<GridIndexType: IndexType> SquareGrid<GridIndexType> {
    pub fn new(dimension_size: GridIndexType) -> SquareGrid<GridIndexType> {

        let dim_size = dimension_size.index();
        let cells_count = dim_size * dim_size;
        let nodes_count_hint = cells_count;
        let edges_count_hint = 4 * cells_count - 4 * dim_size; // Probably overkill, but don't want any capacity panics

        let mut grid = SquareGrid {
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
            dimension_size: dimension_size,
        };
        for _ in 0..cells_count {
            let _ = grid.graph.add_node(());
        }

        grid
    }

    pub fn size(&self) -> usize {
        self.dimension_size.index() * self.dimension_size.index()
    }

    pub fn dimension(&self) -> usize {
        self.dimension_size.index()
    }

    pub fn random_cell(&self) -> GridCoordinate {
        let mut rng = rand::thread_rng();
        let index = rng.gen::<usize>() % self.size();
        index_to_grid_coordinate(self.dimension_size.index(), index)
    }

    /// Link two cells
    ///
    /// Todo - only allow links between adjacent cells? If `b` not in `g.neighbours(a)`.
    ///      - better to change the API to take an index and GridDirection
    ///
    /// Panics if a cell does not exist.
    pub fn link(&mut self, a: GridCoordinate, b: GridCoordinate) {
        if a != b {
            let a_index = self.grid_coordinate_graph_index(&a);
            let b_index = self.grid_coordinate_graph_index(&b);
            let _ = self.graph.update_edge(a_index, b_index, ());
        }
    }

    /// Unlink two cells, if a link exists between them.
    pub fn unlink(&mut self, a: GridCoordinate, b: GridCoordinate) {
        let a_index = self.grid_coordinate_graph_index(&a);
        let b_index = self.grid_coordinate_graph_index(&b);
        if let Some(edge_index) = self.graph.find_edge(a_index, b_index) {
            // This will invalidate the last edge index in the graph, which is fine as we
            // are not storing them for any reason.
            self.graph.remove_edge(edge_index);
        }
    }

    /// Cell nodes that are linked to a particular node by a passage.
    pub fn links(&self, coord: GridCoordinate) -> CoordinateSmallVec {
        self.graph
            .edges(self.grid_coordinate_graph_index(&coord))
            .map(|index_edge_data_pair| {
                let grid_node_index = index_edge_data_pair.0;
                index_to_grid_coordinate(self.dimension_size.index(), grid_node_index.index())
            })
            .collect()
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    pub fn neighbours(&self, coord: GridCoordinate) -> CoordinateSmallVec {

        [offset_coordinate(&coord, GridDirection::North),
         offset_coordinate(&coord, GridDirection::South),
         offset_coordinate(&coord, GridDirection::East),
         offset_coordinate(&coord, GridDirection::West)]
            .into_iter()
            .filter(|adjacent_coord| self.is_valid_coordinate(adjacent_coord))
            .cloned()
            .collect()
    }

    pub fn neighbours_at_directions(&self,
                                    coord: &GridCoordinate,
                                    dirs: &[GridDirection])
                                    -> CoordinateOptionSmallVec {
        dirs.iter()
            .map(|direction| self.neighbour_at_direction(coord, *direction))
            .collect()
    }

    pub fn neighbour_at_direction(&self,
                                  coord: &GridCoordinate,
                                  direction: GridDirection)
                                  -> Option<GridCoordinate> {
        let neighbour_coord = offset_coordinate(coord, direction);
        if self.is_valid_coordinate(&neighbour_coord) {
            Some(neighbour_coord)
        } else {
            None
        }
    }

    /// Are two cells in the grid linked?
    pub fn is_linked(&self, a: GridCoordinate, b: GridCoordinate) -> bool {
        let a_index = self.grid_coordinate_graph_index(&a);
        let b_index = self.grid_coordinate_graph_index(&b);
        self.graph.find_edge(a_index, b_index).is_some()
    }

    pub fn iter(&self) -> CellIter {
        let dim_size = self.dimension_size.index();
        CellIter {
            current_cell_number: 0,
            dimension_size: dim_size,
            cells_count: dim_size * dim_size,
        }
    }

    pub fn iter_row(&self) -> BatchIter {
        BatchIter {
            iter_type: BatchIterType::Row,
            current_index: 0,
            dimension_size: self.dimension_size.index(),
        }
    }

    pub fn iter_column(&self) -> BatchIter {
        BatchIter {
            iter_type: BatchIterType::Column,
            current_index: 0,
            dimension_size: self.dimension_size.index(),
        }
    }

    fn is_neighbour(&self, a: GridCoordinate, b: GridCoordinate) -> bool {
        self.neighbours(a).iter().any(|&coord| coord == b)
    }

    pub fn is_neighbour_linked(&self, coord: &GridCoordinate, direction: GridDirection) -> bool {
        self.neighbour_at_direction(coord, direction)
            .map_or(false,
                    |neighbour_coord| self.is_linked(*coord, neighbour_coord))
    }

    fn is_valid_coordinate(&self, coord: &GridCoordinate) -> bool {
        let (x, y) = (coord.x, coord.y);
        let dim_size = self.dimension_size.index() as isize;
        if x < 0 || y < 0 || x >= dim_size || y >= dim_size {
            return false;
        }
        true
    }

    fn grid_coordinate_graph_index(&self,
                                   coord: &GridCoordinate)
                                   -> graph::NodeIndex<GridIndexType> {
        let grid_index_raw = ((coord.y * self.dimension_size.index() as isize) + coord.x) as usize;
        graph::NodeIndex::<GridIndexType>::new(grid_index_raw)
    }
}

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

        let columns_count = self.dimension_size.index();
        let rows_count = columns_count;

        // Start by special case rendering the text for the north most boundary
        let first_grid_row: &Vec<GridCoordinate> =
            &self.iter_row().take(1).collect::<Vec<Vec<_>>>()[0];
        let mut output = String::from(WALL_RD);
        for (index, coord) in first_grid_row.iter().enumerate() {
            output.push_str(WALL_LR_3);
            let is_east_open = self.is_neighbour_linked(&coord, GridDirection::East);
            if is_east_open {
                output.push_str(WALL_LR);
            } else {
                let is_last_cell = index == columns_count - 1;
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
                    self.neighbour_at_direction(&cell_coord, direction)
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
                let east_open = self.is_neighbour_linked(&cell_coord, GridDirection::East);
                let south_open = self.is_neighbour_linked(&cell_coord, GridDirection::South);

                // Each cell will simply use the southern wall of the cell above
                // it as its own northern wall, so we only need to worry about the cell’s body (room space),
                // its eastern boundary ('|'), and its southern boundary ('---+') minus the south west corner.
                let body = "   "; // 3 spaces
                let east_boundary = render_cell_side(GridDirection::East, " ", WALL_UD);
                row_middle_section_render.push_str(body);
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
                            self.neighbour_at_direction(&cell_coord, GridDirection::East)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(&c, GridDirection::South));
                        let access_se_from_south =
                            self.neighbour_at_direction(&cell_coord, GridDirection::South)
                                .map_or(false,
                                        |c| self.is_neighbour_linked(&c, GridDirection::East));
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
    dimension_size: usize,
    cells_count: usize,
}
impl Iterator for CellIter {
    type Item = GridCoordinate;
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
impl<'a, GridIndexType: IndexType> IntoIterator for &'a SquareGrid<GridIndexType> {
    type Item = GridCoordinate;
    type IntoIter = CellIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Copy, Clone)]
enum BatchIterType {
    Row,
    Column,
}
#[derive(Debug, Copy, Clone)]
pub struct BatchIter {
    iter_type: BatchIterType,
    current_index: usize,
    dimension_size: usize,
}
impl Iterator for BatchIter {
    type Item = Vec<GridCoordinate>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.dimension_size {
            let coords = (0..self.dimension_size)
                             .into_iter()
                             .map(|i| {
                                 if let BatchIterType::Row = self.iter_type {
                                     GridCoordinate::new(i as isize, self.current_index as isize)
                                 } else {
                                     GridCoordinate::new(self.current_index as isize, i as isize)
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
        let lower_bound = self.dimension_size - self.current_index;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}

fn index_to_grid_coordinate(dimension_size: usize, one_dimensional_index: usize) -> GridCoordinate {
    let y = one_dimensional_index / dimension_size;
    let x = one_dimensional_index - (y * dimension_size);
    GridCoordinate {
        x: x as isize,
        y: y as isize,
    }
}

fn offset_coordinate(coord: &GridCoordinate, dir: GridDirection) -> GridCoordinate {
    let (x, y) = (coord.x, coord.y);
    match dir {
        GridDirection::North => GridCoordinate { y: y - 1, ..*coord },
        GridDirection::South => GridCoordinate { y: y + 1, ..*coord },
        GridDirection::East => GridCoordinate { x: x + 1, ..*coord },
        GridDirection::West => GridCoordinate { x: x - 1, ..*coord },
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use itertools::Itertools; // a trait
    use smallvec::SmallVec;

    type SmallGrid = SquareGrid<u8>;

    // Compare a smallvec to e.g. a vec! or &[T].
    // SmallVec really ruins the syntax ergonomics, hence this macro
    // The compiler often succeeds in automatically adding the correct & and derefs (*) but not here
    macro_rules! assert_smallvec_eq {
        ($x:expr, $y:expr) => (assert_eq!(&*$x, &*$y))
    }

    #[test]
    fn neighbour_cells() {
        let g = SmallGrid::new(10);

        let check_expected_neighbours = |coord, expected_neighbours: &[GridCoordinate]| {
            let node_indices: Vec<GridCoordinate> = g.neighbours(coord).iter().cloned().sorted();
            let expected_indices: Vec<GridCoordinate> = expected_neighbours.into_iter()
                                                                           .cloned()
                                                                           .sorted();
            assert_eq!(node_indices, expected_indices);
        };
        let gc = |x, y| GridCoordinate::new(x, y);

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
        let gc = |x, y| GridCoordinate::new(x, y);

        let check_neighbours = |coord,
                                dirs: &[GridDirection],
                                neighbour_opts: &[Option<GridCoordinate>]| {

            let neighbour_options: CoordinateOptionSmallVec = g.neighbours_at_directions(&coord,
                                                                                         dirs);
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
        let gc = |x, y| GridCoordinate::new(x, y);
        let check_neighbour = |coord, dir: GridDirection, expected| {
            assert_eq!(g.neighbour_at_direction(&coord, dir), expected);
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
    fn random_cell() {
        let g = SmallGrid::new(4);
        let cells_count = 4 * 4;
        for _ in 0..1000 {
            let coord = g.random_cell();
            assert!(coord.x >= 0 && coord.x < cells_count);
            assert!(coord.y >= 0 && coord.y < cells_count);
        }
    }

    #[test]
    fn cell_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter().collect::<Vec<GridCoordinate>>(),
                   &[GridCoordinate::new(0, 0),
                     GridCoordinate::new(1, 0),
                     GridCoordinate::new(0, 1),
                     GridCoordinate::new(1, 1)]);
    }

    #[test]
    fn row_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_row().collect::<Vec<Vec<GridCoordinate>>>(),
                   &[&[GridCoordinate::new(0, 0), GridCoordinate::new(1, 0)],
                     &[GridCoordinate::new(0, 1), GridCoordinate::new(1, 1)]]);
    }

    #[test]
    fn column_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_column().collect::<Vec<Vec<GridCoordinate>>>(),
                   &[&[GridCoordinate::new(0, 0), GridCoordinate::new(0, 1)],
                     &[GridCoordinate::new(1, 0), GridCoordinate::new(1, 1)]]);
    }

    #[test]
    fn linking_cells() {
        let mut g = SmallGrid::new(4);
        let a = GridCoordinate::new(0, 1);
        let b = GridCoordinate::new(0, 2);
        let c = GridCoordinate::new(0, 3);

        // Testing the expected grid `links`
        let sorted_links = |grid: &SmallGrid, coord| -> Vec<GridCoordinate> {
            grid.links(coord).iter().cloned().sorted()
        };
        macro_rules! links_sorted {
            ($x:expr) => (sorted_links(&g, $x))
        }

        // Testing that the order of the arguments to `is_linked` does not matter
        macro_rules! bi_check_linked {
            ($x:expr, $y:expr) => (g.is_linked($x, $y) && g.is_linked($y, $x))
        }

        // Testing `is_neighbour_linked` for all directions
        let all_dirs = [GridDirection::North, GridDirection::South,
                        GridDirection::East, GridDirection::West];

        let directional_links_check = |grid: &SmallGrid, coord: &GridCoordinate,
                                       expected_dirs_linked: &[GridDirection]| {

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
            ($coord:expr, $expected:expr) => (directional_links_check(&g, &$coord, &$expected))
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

        g.link(a, b);
        // a - b linked bi-directionally
        assert!(bi_check_linked!(a, b));
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a]);
        check_directional_links!(a, [GridDirection::South]);
        check_directional_links!(b, [GridDirection::North]);
        check_directional_links!(c, []);

        g.link(b, c);
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
        g.unlink(a, b);
        assert!(!bi_check_linked!(a, b));
        assert!(bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![c]);
        assert_eq!(links_sorted!(c), vec![b]);
        check_directional_links!(a, []);
        check_directional_links!(b, [GridDirection::South]);
        check_directional_links!(c, [GridDirection::North]);

        // a, b and c start all unlinked again
        g.unlink(b, c);
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
        let a = GridCoordinate::new(0, 0);
        g.link(a, a);
        assert!(g.links(a).is_empty());
    }

    #[test]
    fn no_parallel_duplicated_linked_cells() {
        let mut g = SmallGrid::new(4);
        let a = GridCoordinate::new(0, 0);
        let b = GridCoordinate::new(0, 1);
        g.link(a, b);
        g.link(a, b);
        assert_smallvec_eq!(g.links(a), &[b]);
        assert_smallvec_eq!(g.links(b), &[a]);

        g.unlink(a, b);
        assert_smallvec_eq!(g.links(a), &[]);
        assert_smallvec_eq!(g.links(b), &[]);
    }
}
