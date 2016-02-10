use petgraph::{Graph, Undirected};
use petgraph::graph;
use petgraph::graph::IndexType;
use rand;
use rand::distributions::{IndependentSample, Range};
use std::fmt;
use std::iter;

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

#[derive(Copy, Clone)]
pub enum GridDirection {
    North,
    South,
    East,
    West,
}

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

    pub fn random_cell(&self) -> GridCoordinate {
        let range_end_exclusive = self.size();
        let random_pos = Range::new(0, range_end_exclusive);
        let mut rng = rand::thread_rng();
        let index = random_pos.ind_sample(&mut rng);
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
    pub fn links(&self, coord: GridCoordinate) -> Vec<GridCoordinate> {
        self.graph
            .edges(self.grid_coordinate_graph_index(&coord))
            .map(|index_edge_data_pair| {
                let grid_node_index = index_edge_data_pair.0.clone();
                index_to_grid_coordinate(self.dimension_size.index(), grid_node_index.index())
            })
            .collect()
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    pub fn neighbours(&self, coord: GridCoordinate) -> Vec<GridCoordinate> {

        vec![offset_coordinate(&coord, GridDirection::North),
             offset_coordinate(&coord, GridDirection::South),
             offset_coordinate(&coord, GridDirection::East),
             offset_coordinate(&coord, GridDirection::West)]
            .into_iter()
            .filter(|adjacent_coord| self.is_valid_coordinate(adjacent_coord))
            .collect()
    }

    pub fn neighbours_at_directions(&self,
                                    coord: &GridCoordinate,
                                    dirs: &Vec<GridDirection>)
                                    -> Vec<Option<GridCoordinate>> {
        dirs.iter()
            .map(|direction| self.neighbour_at_direction(coord, direction.clone()))
            .collect()
    }

    pub fn neighbour_at_direction(&self,
                                  coord: &GridCoordinate,
                                  direction: GridDirection)
                                  -> Option<GridCoordinate> {
        let neighbour_coord = offset_coordinate(coord, direction.clone());
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

        // we could try to make an educated guess for the capacity of the string
        let mut output = "+".to_string();
        // have to explictly convert to String from &str
        let maze_top: String = iter::repeat("---+").take(self.dimension_size.index()).collect();
        output.push_str(&maze_top);
        output.push_str("\n");

        // for row in self.iter_row() {

        //     let top = "|";
        //     let bottom = "+";
        //     for &cell_coord in row {

        //         let body = "   "; // 3 spaces
        //         let east_boundary =
        //             if self.is_linked(cell_coord,
        //                               self.neighbour_at_direction(&cell_coord,
        //                                                           GridDirection::East)) {
        //                 " "
        //             } else {
        //                 "|"
        //             };
        //     }
        // }


        Ok(())
    }
}

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
}

enum BatchIterType {
    Row,
    Column,
}
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
mod test {

    use super::*;
    use itertools::Itertools; // a trait

    type SmallGrid = SquareGrid<u8>;

    #[test]
    fn neighbour_cells() {
        let g = SmallGrid::new(10);

        let check_expected_neighbours = |coord, vec_expected_neighbours: Vec<GridCoordinate>| {
            let node_indices = g.neighbours(coord)
                                .into_iter()
                                .sorted();
            let expected_indices = vec_expected_neighbours.into_iter()
                                                          .sorted();
            assert_eq!(node_indices, expected_indices);
        };
        let gc = |x, y| GridCoordinate::new(x, y);

        // corners
        check_expected_neighbours(gc(0, 0), vec![gc(1, 0), gc(0, 1)]);
        check_expected_neighbours(gc(9, 0), vec![gc(8, 0), gc(9, 1)]);
        check_expected_neighbours(gc(0, 9), vec![gc(0, 8), gc(1, 9)]);
        check_expected_neighbours(gc(9, 9), vec![gc(9, 8), gc(8, 9)]);

        // side element examples
        check_expected_neighbours(gc(1, 0), vec![gc(0, 0), gc(1, 1), gc(2, 0)]);
        check_expected_neighbours(gc(0, 1), vec![gc(0, 0), gc(0, 2), gc(1, 1)]);
        check_expected_neighbours(gc(0, 8), vec![gc(1, 8), gc(0, 7), gc(0, 9)]);
        check_expected_neighbours(gc(9, 8), vec![gc(9, 7), gc(9, 9), gc(8, 8)]);

        // Some place with 4 neighbours inside the grid
        check_expected_neighbours(gc(1, 1), vec![gc(0, 1), gc(1, 0), gc(2, 1), gc(1, 2)]);
    }

    #[test]
    fn neighbours_at_dirs() {
        let g = SmallGrid::new(2);
        let gc = |x, y| GridCoordinate::new(x, y);

        let check_neighbours = |coord, dirs, vec_neighbour_opts: Vec<Option<GridCoordinate>>| {
            let neighbour_options = g.neighbours_at_directions(&coord, &dirs);
            assert_eq!(neighbour_options, vec_neighbour_opts);
        };
        check_neighbours(gc(0, 0), vec![], vec![]);
        check_neighbours(gc(0, 0), vec![GridDirection::North], vec![None]);
        check_neighbours(gc(0, 0), vec![GridDirection::West], vec![None]);
        check_neighbours(gc(0, 0),
                         vec![GridDirection::West, GridDirection::North],
                         vec![None, None]);
        check_neighbours(gc(0, 0),
                         vec![GridDirection::East, GridDirection::South],
                         vec![Some(gc(1, 0)), Some(gc(0, 1))]);

        check_neighbours(gc(1, 1), vec![], vec![]);
        check_neighbours(gc(1, 1), vec![GridDirection::South], vec![None]);
        check_neighbours(gc(1, 1), vec![GridDirection::East], vec![None]);
        check_neighbours(gc(1, 1),
                         vec![GridDirection::South, GridDirection::East],
                         vec![None, None]);
        check_neighbours(gc(1, 1),
                         vec![GridDirection::West, GridDirection::North],
                         vec![Some(gc(0, 1)), Some(gc(1, 0))]);
    }

    #[test]
    fn grid_size() {
        let g = SmallGrid::new(10);
        assert_eq!(g.size(), 100);
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
                   vec![GridCoordinate::new(0, 0),
                        GridCoordinate::new(1, 0),
                        GridCoordinate::new(0, 1),
                        GridCoordinate::new(1, 1)]);
    }

    #[test]
    fn row_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_row().collect::<Vec<Vec<GridCoordinate>>>(),
                   vec![vec![GridCoordinate::new(0, 0), GridCoordinate::new(1, 0)],
                        vec![GridCoordinate::new(0, 1), GridCoordinate::new(1, 1)]]);
    }

    #[test]
    fn column_iter() {
        let g = SmallGrid::new(2);
        assert_eq!(g.iter_column().collect::<Vec<Vec<GridCoordinate>>>(),
                   vec![vec![GridCoordinate::new(0, 0), GridCoordinate::new(0, 1)],
                        vec![GridCoordinate::new(1, 0), GridCoordinate::new(1, 1)]]);
    }

    #[test]
    fn linking_cells() {
        let mut g = SmallGrid::new(4);
        let a = GridCoordinate::new(0, 1);
        let b = GridCoordinate::new(0, 2);
        let c = GridCoordinate::new(0, 3);

        // I'd rather use a closure, but it needs to borrow the graph immutably until
        // it goes out of scope and it's ugly to pass the grid into each function call
        macro_rules! links_sorted {
            ($x:expr) => (g.links($x).into_iter().sorted())
        }
        // Testing that the order of the arguments to `is_linked` does not matter
        macro_rules! bi_check_linked {
            ($x:expr, $y:expr) => (g.is_linked($x, $y) && g.is_linked($y, $x))
        }

        // a, b and c start with no links
        assert!(!bi_check_linked!(a, b));
        assert!(!bi_check_linked!(a, c));
        assert!(!bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);

        g.link(a, b);
        // a - b linked bi-directionally
        assert!(bi_check_linked!(a, b));
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a]);

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

        // a - b unlinked
        // b still linked to c bi-directionally
        g.unlink(a, b);
        assert!(!bi_check_linked!(a, b));
        assert!(bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![c]);
        assert_eq!(links_sorted!(c), vec![b]);

        // a, b and c start all unlinked again
        g.unlink(b, c);
        assert!(!bi_check_linked!(a, b));
        assert!(!bi_check_linked!(a, c));
        assert!(!bi_check_linked!(b, c));
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);
    }

    #[test]
    fn no_self_linked_cycles() {
        let mut g = SmallGrid::new(4);
        let a = GridCoordinate::new(0, 0);
        g.link(a, a);
        assert_eq!(g.links(a), vec![]);
    }

    #[test]
    fn no_parallel_duplicated_linked_cells() {
        let mut g = SmallGrid::new(4);
        let a = GridCoordinate::new(0, 0);
        let b = GridCoordinate::new(0, 1);
        g.link(a, b);
        g.link(a, b);
        assert_eq!(g.links(a), vec![b]);
        assert_eq!(g.links(b), vec![a]);

        g.unlink(a, b);
        assert_eq!(g.links(a), vec![]);
        assert_eq!(g.links(b), vec![]);
    }
}
