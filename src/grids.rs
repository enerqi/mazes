use std::cmp;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use petgraph::{Graph, Undirected};
use petgraph::graph;
pub use petgraph::graph::IndexType;
use rand::XorShiftRng;

use cells::{Cell, Coordinate, Cartesian2DCoordinate};
use grid_traits::{GridIterators, GridDisplay, GridDimensions, GridPositions};
use units::{RowsCount, RowLength, RowIndex, ColumnsCount, ColumnLength,
            ColumnIndex};


pub struct Grid<GridIndexType: IndexType, CellT: Cell, Iters: GridIterators<CellT>> {

    graph: Graph<(), (), Undirected, GridIndexType>,
    dimensions: Box<GridDimensions>,
    positions: Box<GridPositions<CellT>>,
    iterators: Iters, // cannot be trait without boxing the CellIter/BatchIter types - type CellIter: Box<Iterator...>
    grid_display: Option<Rc<GridDisplay<CellT>>>,
    cell_type: PhantomData<CellT>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CellLinkError {
    InvalidGridCoordinate,
    SelfLink,
}

impl<GridIndexType: IndexType, CellT: Cell, Iters: GridIterators<CellT>> fmt::Debug for Grid<GridIndexType, CellT, Iters> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Grid :: graph: {:?}, rows: {:?}, columns: {:?}",
               self.graph, self.row_length(), self.column_length())
    }
}

impl<GridIndexType: IndexType, CellT: Cell, Iters: GridIterators<CellT>> Grid<GridIndexType, CellT, Iters> {

    pub fn new(dimensions: Box<GridDimensions>,
               positions: Box<GridPositions<CellT>>,
               iterators: Iters) -> Grid<GridIndexType, CellT, Iters> {

        let row_len = dimensions.row_length(None).unwrap();
        let column_len = dimensions.column_length(None);
        let cells_count = row_len.0 * column_len.0;
        let nodes_count_hint = cells_count;
        let edges_count_hint = 4 * cells_count - 4 * cmp::max(row_len.0, column_len.0); // Probably overkill, but don't want any capacity panics

        let mut grid = Grid {
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
            dimensions: dimensions,
            positions: positions,
            iterators: iterators,
            grid_display: None,
            cell_type: PhantomData
        };
        for _ in 0..cells_count {
            let _ = grid.graph.add_node(());
        }

        grid
    }

    pub fn set_grid_display(&mut self, grid_display: Option<Rc<GridDisplay<CellT>>>) {
        self.grid_display = grid_display;
    }

    #[inline(always)]
    pub fn grid_display(&self) -> &Option<Rc<GridDisplay<CellT>>> {
        &self.grid_display
    }

    // Todo: make a macro delegating some functions to sel.data.

    #[inline(always)]
    pub fn dimensions(&self) -> &GridDimensions {
        self.dimensions.as_ref()
    }

    #[inline(always)]
    pub fn positions(&self) -> &GridPositions<CellT> {
        self.positions.as_ref()
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.dimensions.size().0
    }

    #[inline(always)]
    pub fn rows(&self) -> RowsCount {
        self.dimensions.rows()
    }

    #[inline(always)]
    pub fn row_length(&self) -> Option<RowLength> {
        self.dimensions.row_length(None)
        //RowLength(self.columns.0)
    }

    #[inline(always)]
    pub fn columns(&self) -> ColumnsCount {
        self.dimensions.columns()
    }

    #[inline(always)]
    pub fn column_length(&self) -> ColumnLength {
        self.dimensions.column_length(None)
        //ColumnLength(self.rows.0)
    }

    pub fn random_cell(&self, mut rng: &mut XorShiftRng) -> CellT::Coord {
        self.positions.random_cell(&mut rng)
        // let index = rng.gen::<usize>() % self.size();
        // CellT::Coord::from_row_major_index(index, self.row_length(), self.column_length())
    }

    /// Link two cells
    ///
    /// Todo - only allow links between adjacent cells? If `b` not in `g.neighbours(a)`.
    ///      - better to change the API to take an index and GridDirection
    ///
    /// Panics if a cell does not exist.
    pub fn link(&mut self, a: CellT::Coord, b: CellT::Coord) -> Result<(), CellLinkError> {
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
    pub fn unlink(&mut self, a: CellT::Coord, b: CellT::Coord) -> bool {
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
    pub fn links(&self, coord: CellT::Coord) -> Option<CellT::CoordinateSmallVec> {

        if let Some(graph_node_index) = self.grid_coordinate_graph_index(coord) {

            let linked_cells = self.graph
                .edges(graph_node_index)
                .map(|index_edge_data_pair| {
                    let grid_node_index = index_edge_data_pair.0;
                    CellT::Coord::from_row_major_index(grid_node_index.index(),
                                                       self.dimensions())
                })
                .collect();
            Some(linked_cells)
        } else {
            None
        }
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    pub fn neighbours(&self, coord: CellT::Coord) -> CellT::CoordinateSmallVec {

        let all_dirs: CellT::DirectionSmallVec = CellT::offset_directions(&Some(coord));
        (&all_dirs).iter()
                 .cloned()
                 .map(|dir: CellT::Direction| CellT::offset_coordinate(coord, dir))
                 .filter_map(|adjacent_coord_opt: Option<CellT::Coord>| -> Option<CellT::Coord> {
                    if let Some(adjacent_coord) = adjacent_coord_opt {
                        if self.is_valid_coordinate(adjacent_coord.as_cartesian_2d()) {
                            adjacent_coord_opt
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                 })
                 .collect::<CellT::CoordinateSmallVec>()
    }

    pub fn neighbours_at_directions(&self, coord: CellT::Coord, dirs: &[CellT::Direction]) -> CellT::CoordinateOptionSmallVec {
        dirs.iter()
            .map(|direction| self.neighbour_at_direction(coord, *direction))
            .collect()
    }

    pub fn neighbour_at_direction(&self,
                                  coord: CellT::Coord,
                                  direction: CellT::Direction)
                                  -> Option<CellT::Coord> {
        let neighbour_coord_opt = CellT::offset_coordinate(coord, direction);

        neighbour_coord_opt.and_then(|neighbour_coord: CellT::Coord| {
            if self.is_valid_coordinate(neighbour_coord.as_cartesian_2d()) {
                Some(neighbour_coord)
            } else {
                None
            }
        })
    }

    /// Are two cells in the grid linked?
    pub fn is_linked(&self, a: CellT::Coord, b: CellT::Coord) -> bool {
        let a_index_opt = self.grid_coordinate_graph_index(a);
        let b_index_opt = self.grid_coordinate_graph_index(b);
        if let (Some(a_index), Some(b_index)) = (a_index_opt, b_index_opt) {
            self.graph.find_edge(a_index, b_index).is_some()
        } else {
            false
        }
    }

    pub fn is_neighbour_linked(&self, coord: CellT::Coord, direction: CellT::Direction) -> bool {
        self.neighbour_at_direction(coord, direction)
            .map_or(false,
                    |neighbour_coord| self.is_linked(coord, neighbour_coord))
    }

    /// Convert a grid coordinate to a one dimensional index in the range 0...grid.size().
    /// Returns None if the grid coordinate is invalid.
    pub fn grid_coordinate_to_index(&self, coord: CellT::Coord) -> Option<usize> {

        self.positions.grid_coordinate_to_index(coord)

        // let grid_2d_coord = coord.as_cartesian_2d();
        // if self.is_valid_coordinate(grid_2d_coord) {
        //     let RowLength(row_size) = self.row_length();
        //     Some((grid_2d_coord.y as usize * row_size) + grid_2d_coord.x as usize)
        // } else {
        //     None
        // }
    }

    #[inline(always)]
    pub fn iter(&self) -> Iters::CellIter {
        self.iterators.iter(self.dimensions.as_ref())
    }

    #[inline(always)]
    pub fn iter_row(&self) -> Iters::BatchIter {
        self.iterators.iter_row(self.dimensions.as_ref())
    }

    #[inline(always)]
    pub fn iter_column(&self) -> Iters::BatchIter {
        self.iterators.iter_column(self.dimensions.as_ref())
    }

    /// Is the grid coordinate valid for this grid - within the grid's dimensions
    #[inline]
    pub fn is_valid_coordinate(&self, coord: Cartesian2DCoordinate) -> bool {
        //let row_index = coord.y as usize;
        let RowLength(width) = self.row_length() //.row_length(Some(RowIndex(row_index)))
                                   .expect("RowIndex invalid");
        let ColumnLength(height) = self.column_length();
        (coord.x as usize) < width && (coord.y as usize) < height
    }

    fn is_neighbour(&self, a: CellT::Coord, b: CellT::Coord) -> bool {
                                // For .iter Coord satifies `Deref<Target=[Self::Coord]>`
        self.neighbours(a).iter().any(|&coord| coord == b)
    }

    /// Convert a grid coordinate into petgraph nodeindex
    /// Returns None if the grid coordinate is invalid (out of the grid's dimensions).
    #[inline]
    fn grid_coordinate_graph_index(&self,
                                   coord: CellT::Coord)
                                   -> Option<graph::NodeIndex<GridIndexType>> {
        let grid_index_raw_opt = self.grid_coordinate_to_index(coord);
        grid_index_raw_opt.map(graph::NodeIndex::<GridIndexType>::new)
    }
}


#[derive(Copy, Clone)]
pub struct CellIter<'d, CellT: Cell> {
    current_cell_number: usize,
    dimensions: &'d GridDimensions,
    row_length: RowLength,
    col_length: ColumnLength,
    cells_count: usize,
    cell_type: PhantomData<CellT>
}
impl<'d, CellT: Cell> fmt::Debug for CellIter<'d, CellT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CellIter :: current_cell_number: {:?}, cells_count: {:?}",
               self.current_cell_number, self.cells_count)
    }
}

impl<'d, CellT: Cell> ExactSizeIterator for CellIter<'d, CellT> { } // default impl using size_hint()
impl<'d, CellT: Cell> Iterator for CellIter<'d, CellT> {
    type Item = CellT::Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cell_number < self.cells_count {
            let coord = Self::Item::from_row_major_index(self.current_cell_number,
                                                         self.dimensions);
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

// Converting a &Grid into an iterator CellIter - the default most sensible
// impl<'a, 'd,
//      GridIndexType: IndexType,
//      CellT: Cell,
//      Dimensions: GridDimensions, Positions: GridPositions<CellT>,
//      Iters: GridIterators<CellT, Dimensions>>
//     IntoIterator for &'a Grid<GridIndexType, CellT, Data, Iters> {
//     type Item = CellT::Coord;
//     type IntoIter = CellIter<'d, CellT>;

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
pub struct BatchIter<CellT> {
    iter_type: BatchIterType,
    iter_initial_length: usize,
    current_index: usize,
    row_length: RowLength,
    rows_size: RowsCount,
    col_length: ColumnLength,
    cols_size: ColumnsCount,
    cell_type: PhantomData<CellT>,
}
impl<CellT: Cell> ExactSizeIterator for BatchIter<CellT> { } // default impl using size_hint()
impl<CellT: Cell> Iterator for BatchIter<CellT> {
    type Item = Vec<CellT::Coord>;
    fn next(&mut self) -> Option<Self::Item> {

        if let BatchIterType::Row = self.iter_type {

            let RowsCount(count) = self.rows_size;
            if self.current_index < count {
                let RowLength(length) = self.row_length;
                let coords = (0..length)
                    .into_iter()
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(ColumnIndex(i), RowIndex(self.current_index))
                    })
                    .collect();
                self.current_index += 1;
                Some(coords)
            } else {
                None
            }

        } else {

            let ColumnsCount(count) = self.cols_size;
            if self.current_index < count {
                let ColumnLength(length) = self.col_length;
                let coords = (0..length)
                    .into_iter()
                    .map(|i: usize| {
                        CellT::Coord::from_row_column_indices(ColumnIndex(self.current_index), RowIndex(i))
                    })
                    .collect();
                self.current_index += 1;
                Some(coords)
            } else {
                None
            }

        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower_bound = self.iter_initial_length - self.current_index;
        let upper_bound = lower_bound;
        (lower_bound, Some(upper_bound))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use cells::Cartesian2DCoordinate;
    use itertools::Itertools; // a trait
    use rand;
    use smallvec::SmallVec;
    use std::u32;

    type SmallGrid<'a> = Grid<u8>;

    // Compare a smallvec to e.g. a vec! or &[T].
    // SmallVec really ruins the syntax ergonomics, hence this macro
    // The compiler often succeeds in automatically adding the correct & and derefs (*) but not here
    // - SmallVec does not implement IntoIterator, but you can deref it to [T] and take a slice
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
            |coord, dirs: &[CompassPrimary], neighbour_opts: &[Option<Cartesian2DCoordinate>]| {

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
