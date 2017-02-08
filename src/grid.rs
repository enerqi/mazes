#![allow(unused_qualifications)] // until rust 1.15 is stable or fn small_grid works in beta and stable.


use cells::{Cell, Coordinate};
use grid_traits::{GridCoordinates, GridDimensions, GridDisplay, GridIterators};

use petgraph::{Graph, Undirected};
use petgraph::graph;
pub use petgraph::graph::IndexType;
use rand::XorShiftRng;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::slice;
use units::{ColumnLength, ColumnsCount, EdgesCount, NodesCount, RowLength, RowsCount};


pub struct Grid<GridIndexType: IndexType, CellT: Cell, Iters: GridIterators<CellT>> {
    graph: Graph<(), (), Undirected, GridIndexType>,
    dimensions: Rc<GridDimensions>,
    coordinates: Box<GridCoordinates<CellT>>,
    iterators: Iters, /* cannot be trait without boxing the CellIter/BatchIter types - type CellIter: Box<Iterator...> */
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

impl<GridIndexType: IndexType, CellT: Cell, Iters: GridIterators<CellT>> Grid<GridIndexType,
                                                                              CellT,
                                                                              Iters> {
    pub fn new(dimensions: Rc<GridDimensions>,
               coordinates: Box<GridCoordinates<CellT>>,
               iterators: Iters)
               -> Grid<GridIndexType, CellT, Iters> {

        let (NodesCount(nodes), EdgesCount(edges)) = dimensions.graph_size();

        let mut grid = Grid {
            graph: Graph::with_capacity(nodes, edges),
            dimensions: dimensions.clone(),
            coordinates: coordinates,
            iterators: iterators,
            grid_display: None,
            cell_type: PhantomData,
        };
        for _ in 0..nodes {
            let _ = grid.graph.add_node(());
        }

        grid
    }

    #[inline]
    pub fn set_grid_display(&mut self, grid_display: Option<Rc<GridDisplay<CellT>>>) {
        self.grid_display = grid_display;
    }

    #[inline]
    pub fn grid_display(&self) -> &Option<Rc<GridDisplay<CellT>>> {
        &self.grid_display
    }

    // Todo: make a macro delegating some functions to sel.data.

    #[inline]
    pub fn dimensions(&self) -> &GridDimensions {
        self.dimensions.as_ref()
    }

    #[inline]
    pub fn coordinates(&self) -> &GridCoordinates<CellT> {
        self.coordinates.as_ref()
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.dimensions.size().0
    }

    #[inline]
    pub fn links_count(&self) -> usize {
        self.graph.edge_count()
    }

    #[inline]
    pub fn rows(&self) -> RowsCount {
        self.dimensions.rows()
    }

    #[inline]
    pub fn row_length(&self) -> Option<RowLength> {
        self.dimensions.row_length(None)
    }

    #[inline]
    pub fn columns(&self) -> ColumnsCount {
        self.dimensions.columns()
    }

    #[inline]
    pub fn column_length(&self) -> ColumnLength {
        self.dimensions.column_length(None)
    }

    #[inline]
    pub fn random_cell(&self, mut rng: &mut XorShiftRng) -> CellT::Coord {
        self.coordinates.random_cell(&mut rng, &self.dimensions)
    }

    /// Link two cells
    ///
    /// Todo - only allow links between adjacent cells? If `b` not in `g.neighbours(a)`.
    ///      - better to change the API to take an index and CompassPrimary
    ///
    /// Panics if a cell does not exist.
    pub fn link(&mut self, a: CellT::Coord, b: CellT::Coord) -> Result<(), CellLinkError> {
        if a != b {
            let a_index_opt = self.grid_coordinate_graph_index(a);
            let b_index_opt = self.grid_coordinate_graph_index(b);
            match (a_index_opt, b_index_opt) {
                (Some(a_index), Some(b_index)) => {
                    let _ = self.graph.update_edge(a_index, b_index, ());
                    Ok(())
                }
                _ => Err(CellLinkError::InvalidGridCoordinate),
            }
        } else {
            Err(CellLinkError::SelfLink)
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
                    CellT::Coord::from_row_major_index(grid_node_index.index(), self.dimensions())
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

        let all_dirs: CellT::DirectionSmallVec = CellT::offset_directions(Some(coord),
                                                                          self.dimensions());
        (&all_dirs)
            .iter()
            .cloned()
            .map(|dir: CellT::Direction| CellT::offset_coordinate(coord, dir, self.dimensions()))
            .filter_map(|adjacent_coord_opt: Option<CellT::Coord>| -> Option<CellT::Coord> {
                if let Some(adjacent_coord) = adjacent_coord_opt {
                    if self.is_valid_coordinate(adjacent_coord) {
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

    pub fn neighbours_at_directions(&self,
                                    coord: CellT::Coord,
                                    dirs: &[CellT::Direction])
                                    -> CellT::CoordinateOptionSmallVec {
        dirs.iter()
            .map(|direction| self.neighbour_at_direction(coord, *direction))
            .collect()
    }

    pub fn neighbour_at_direction(&self,
                                  coord: CellT::Coord,
                                  direction: CellT::Direction)
                                  -> Option<CellT::Coord> {
        let neighbour_coord_opt = CellT::offset_coordinate(coord, direction, self.dimensions());

        neighbour_coord_opt.and_then(|neighbour_coord: CellT::Coord| {
            if self.is_valid_coordinate(neighbour_coord) {
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
    #[inline]
    pub fn grid_coordinate_to_index(&self, coord: CellT::Coord) -> Option<usize> {
        self.coordinates.grid_coordinate_to_index(coord, &self.dimensions)
    }

    #[inline]
    pub fn iter(&self) -> Iters::CellIter {
        self.iterators.iter(&self.dimensions)
    }

    #[inline]
    pub fn iter_row(&self) -> Iters::BatchIter {
        self.iterators.iter_row(&self.dimensions)
    }

    #[inline]
    pub fn iter_column(&self) -> Iters::BatchIter {
        self.iterators.iter_column(&self.dimensions)
    }

    pub fn iter_links(&self) -> LinksIter<CellT, GridIndexType> {
        LinksIter {
            graph_edge_iter: self.graph.raw_edges().iter(),
            dimensions: self.dimensions(),
            cell_type: PhantomData,
        }
    }

    /// Is the grid coordinate valid for this grid - within the grid's dimensions
    #[inline]
    pub fn is_valid_coordinate(&self, coord: CellT::Coord) -> bool {
        self.coordinates.is_valid_coordinate(coord, &self.dimensions)
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

pub struct LinksIter<'a, CellT: Cell, GridIndexType: IndexType> {
    graph_edge_iter: slice::Iter<'a, graph::Edge<(), GridIndexType>>,
    dimensions: &'a GridDimensions,
    cell_type: PhantomData<CellT>,
}

impl<'a, CellT: Cell, GridIndexType: IndexType> Iterator for LinksIter<'a, CellT, GridIndexType> {
    type Item = (CellT::Coord, CellT::Coord);

    fn next(&mut self) -> Option<Self::Item> {
        self.graph_edge_iter.next().map(|edge| {
            let src_cell_coord = CellT::Coord::from_row_major_index(edge.source().index(),
                                                                    self.dimensions);
            let dst_cell_coord = CellT::Coord::from_row_major_index(edge.target().index(),
                                                                    self.dimensions);
            (src_cell_coord, dst_cell_coord)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.graph_edge_iter.size_hint()
    }
}
impl<'a, CellT: Cell, GridIndexType: IndexType> ExactSizeIterator
    for LinksIter<'a, CellT, GridIndexType> {
} // default impl using size_hint()

impl<'a, CellT: Cell, GridIndexType: IndexType> fmt::Debug for LinksIter<'a, CellT, GridIndexType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinksIter :: edges iter : {:?}", self.graph_edge_iter)
    }
}

// Converting a &Grid into an iterator CellIter - the default most sensible
// impl<'a, 'd,
//      GridIndexType: IndexType,
//      CellT: Cell,
//      Dimensions: GridDimensions, Positions: GridCoordinates<CellT>,
//      Iters: GridIterators<CellT, Dimensions>>
//     IntoIterator for &'a Grid<GridIndexType, CellT, Data, Iters> {
//     type Item = CellT::Coord;
//     type IntoIter = CellIter<'d, CellT>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

#[cfg(test)]
mod tests {

    use cells::{Cartesian2DCoordinate, CompassPrimary};
    use grids::{SmallRectangularGrid, small_rect_grid};

    use itertools::Itertools; // a trait
    use rand;
    use smallvec::SmallVec;
    use std::u32;

    use super::*;
    use units;

    fn small_grid(w: usize, h: usize) -> SmallRectangularGrid {
        small_rect_grid(units::RowLength(w), units::ColumnLength(h))
            .expect("grid dimensions too large for small grid")
    }

    // Compare a smallvec to e.g. a vec! or &[T].
    // SmallVec really ruins the syntax ergonomics, hence this macro
    // The compiler often succeeds in automatically adding the correct & and derefs (*) but not here
    // - SmallVec does not implement IntoIterator, but you can deref it to [T] and take a slice
    macro_rules! assert_smallvec_eq {
        ($x:expr, $y:expr) => (assert_eq!(&*$x, &*$y))
    }

    #[test]
    fn neighbour_cells() {
        let g = small_grid(10, 10);

        let check_expected_neighbours = |coord, expected_neighbours: &[Cartesian2DCoordinate]| {
            let node_indices: Vec<Cartesian2DCoordinate> =
                g.neighbours(coord).iter().cloned().sorted();
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
        let g = small_grid(2, 2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);

        let check_neighbours =
            |coord, dirs: &[CompassPrimary], neighbour_opts: &[Option<Cartesian2DCoordinate>]| {

                let neighbour_options = g.neighbours_at_directions(coord, dirs);
                assert_eq!(&*neighbour_options, neighbour_opts);
            };
        check_neighbours(gc(0, 0), &[], &[]);
        check_neighbours(gc(0, 0), &[CompassPrimary::North], &[None]);
        check_neighbours(gc(0, 0), &[CompassPrimary::West], &[None]);
        check_neighbours(gc(0, 0),
                         &[CompassPrimary::West, CompassPrimary::North],
                         &[None, None]);
        check_neighbours(gc(0, 0),
                         &[CompassPrimary::East, CompassPrimary::South],
                         &[Some(gc(1, 0)), Some(gc(0, 1))]);

        check_neighbours(gc(1, 1), &[], &[]);
        check_neighbours(gc(1, 1), &[CompassPrimary::South], &[None]);
        check_neighbours(gc(1, 1), &[CompassPrimary::East], &[None]);
        check_neighbours(gc(1, 1),
                         &[CompassPrimary::South, CompassPrimary::East],
                         &[None, None]);
        check_neighbours(gc(1, 1),
                         &[CompassPrimary::West, CompassPrimary::North],
                         &[Some(gc(0, 1)), Some(gc(1, 0))]);
    }

    #[test]
    fn neighbour_at_dir() {
        let g = small_grid(2, 2);
        let gc = |x, y| Cartesian2DCoordinate::new(x, y);
        let check_neighbour = |coord, dir: CompassPrimary, expected| {
            assert_eq!(g.neighbour_at_direction(coord, dir), expected);
        };
        check_neighbour(gc(0, 0), CompassPrimary::North, None);
        check_neighbour(gc(0, 0), CompassPrimary::South, Some(gc(0, 1)));
        check_neighbour(gc(0, 0), CompassPrimary::East, Some(gc(1, 0)));
        check_neighbour(gc(0, 0), CompassPrimary::West, None);

        check_neighbour(gc(1, 1), CompassPrimary::North, Some(gc(1, 0)));
        check_neighbour(gc(1, 1), CompassPrimary::South, None);
        check_neighbour(gc(1, 1), CompassPrimary::East, None);
        check_neighbour(gc(1, 1), CompassPrimary::West, Some(gc(0, 1)));
    }

    #[test]
    fn grid_size() {
        let g = small_grid(10, 10);
        assert_eq!(g.size(), 100);
    }

    #[test]
    fn grid_rows() {
        let g = small_grid(10, 10);
        assert_eq!(g.dimensions().rows().0, 10);
    }

    #[test]
    fn grid_coordinate_as_index() {
        let g = small_grid(3, 3);
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
        let g = small_grid(4, 4);
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
        let g = small_grid(2, 2);
        assert_eq!(g.iter().collect::<Vec<Cartesian2DCoordinate>>(),
                   &[Cartesian2DCoordinate::new(0, 0),
                     Cartesian2DCoordinate::new(1, 0),
                     Cartesian2DCoordinate::new(0, 1),
                     Cartesian2DCoordinate::new(1, 1)]);
    }

    #[test]
    fn row_iter() {
        let g = small_grid(2, 2);
        assert_eq!(g.iter_row().collect::<Vec<Vec<Cartesian2DCoordinate>>>(),
                   &[&[Cartesian2DCoordinate::new(0, 0), Cartesian2DCoordinate::new(1, 0)],
                     &[Cartesian2DCoordinate::new(0, 1), Cartesian2DCoordinate::new(1, 1)]]);
    }

    #[test]
    fn column_iter() {
        let g = small_grid(2, 2);
        assert_eq!(g.iter_column().collect::<Vec<Vec<Cartesian2DCoordinate>>>(),
                   &[&[Cartesian2DCoordinate::new(0, 0), Cartesian2DCoordinate::new(0, 1)],
                     &[Cartesian2DCoordinate::new(1, 0), Cartesian2DCoordinate::new(1, 1)]]);
    }

    #[test]
    fn linking_cells() {
        let mut g = small_grid(4, 4);
        let a = Cartesian2DCoordinate::new(0, 1);
        let b = Cartesian2DCoordinate::new(0, 2);
        let c = Cartesian2DCoordinate::new(0, 3);

        // Testing the expected grid `links`
        let sorted_links = |grid: &SmallRectangularGrid, coord| -> Vec<Cartesian2DCoordinate> {
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
        let all_dirs = [CompassPrimary::North,
                        CompassPrimary::South,
                        CompassPrimary::East,
                        CompassPrimary::West];

        let directional_links_check = |grid: &SmallRectangularGrid,
                                       coord: Cartesian2DCoordinate,
                                       expected_dirs_linked: &[CompassPrimary]| {

            let expected_complement: SmallVec<[CompassPrimary; 4]> = all_dirs.iter()
                .cloned()
                .filter(|dir: &CompassPrimary| !expected_dirs_linked.contains(dir))
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
        check_directional_links!(a, [CompassPrimary::South]);
        check_directional_links!(b, [CompassPrimary::North]);
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

        check_directional_links!(a, [CompassPrimary::South]);
        check_directional_links!(b, [CompassPrimary::North, CompassPrimary::South]);
        check_directional_links!(c, [CompassPrimary::North]);

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
        check_directional_links!(b, [CompassPrimary::South]);
        check_directional_links!(c, [CompassPrimary::North]);

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
        let mut g = small_grid(4, 4);
        let a = Cartesian2DCoordinate::new(0, 0);
        let link_result = g.link(a, a);
        assert_eq!(link_result, Err(CellLinkError::SelfLink));
    }

    #[test]
    fn no_links_to_invalid_coordinates() {
        let mut g = small_grid(4, 4);
        let good_coord = Cartesian2DCoordinate::new(0, 0);
        let invalid_coord = Cartesian2DCoordinate::new(100, 100);
        let link_result = g.link(good_coord, invalid_coord);
        assert_eq!(link_result, Err(CellLinkError::InvalidGridCoordinate));
    }

    #[test]
    fn no_parallel_duplicated_linked_cells() {
        let mut g = small_grid(4, 4);
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
