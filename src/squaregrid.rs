use petgraph::{Graph, Undirected};
use rand;
use rand::distributions::{IndependentSample, Range};


pub type GridIndexType = u16;
pub type GridGraphNodeIndex = ::petgraph::graph::NodeIndex<GridIndexType>;

pub enum GridDirection {
    North,
    South,
    East,
    West,
}

struct AdjacentCells {
    north: Option<GridGraphNodeIndex>,
    south: Option<GridGraphNodeIndex>,
    east: Option<GridGraphNodeIndex>,
    west: Option<GridGraphNodeIndex>,
}
impl AdjacentCells {
    fn new() -> Self {
        AdjacentCells {
            north: None,
            south: None,
            east: None,
            west: None,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct GridCoordinate {
    pub x: isize,
    pub y: isize,
}

pub struct SquareGrid {
    /// For mapping a 2D gridcoordinate (converted to a 1D index) to a graph node index.
    grid_coordinate_to_node_index_lookup: Vec<GridGraphNodeIndex>,
    graph: Graph<AdjacentCells, (), Undirected, GridIndexType>,
    dimension_size: GridIndexType,
}

impl SquareGrid {
    pub fn new(dimension_size: GridIndexType) -> SquareGrid {

        let cells_count = dimension_size * dimension_size;
        let nodes_count_hint = cells_count as usize;
        let edges_count_hint = 4 * cells_count as usize - 4 * dimension_size as usize; // Probably overkill, but don't want any capacity panics

        let mut grid = SquareGrid {
            grid_coordinate_to_node_index_lookup: Vec::with_capacity(nodes_count_hint),
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
            dimension_size: dimension_size,
        };

        // Although it does seem to, there is no guarantee that the index values are monotonically increasing when calling graph.add_node
        // So we maintain our own symbol table mapping gridcoordinates converted to 1d (the implicit Vec indices) to the NodeIndexes created
        // by adding a node to the graph.
        for _ in 0..cells_count {
            let node_index = grid.graph.add_node(AdjacentCells::new());
            grid.grid_coordinate_to_node_index_lookup.push(node_index);
        }

        for (index, node_index) in grid.grid_coordinate_to_node_index_lookup.iter().enumerate() {

            let coord = grid.index_to_grid_coordinate(index);

            // Limit the lifetime of the find_neighbour_nodeindex closure so we don't get a grid
            // borrow conflict when mutating the adjacent_cells
            let (north_index, south_index, east_index, west_index) = {
                let find_neighbour_nodeindex = |dir: GridDirection| -> Option<GridGraphNodeIndex> {
                    let neighbour_coord = offset_coordinate(&coord, dir);
                    if grid.is_valid_coordinate(&neighbour_coord) {
                        let node_indices_index = grid.grid_coordinate_to_1d_index(&neighbour_coord);
                        let node_index =
                            grid.grid_coordinate_to_node_index_lookup[node_indices_index as usize];
                        Some(node_index)
                    } else {
                        None
                    }
                };
                (find_neighbour_nodeindex(GridDirection::North),
                 find_neighbour_nodeindex(GridDirection::South),
                 find_neighbour_nodeindex(GridDirection::East),
                 find_neighbour_nodeindex(GridDirection::West))
            };

            let adjacent_cells = grid.graph
                                     .node_weight_mut(node_index.clone())
                                     .unwrap();
            adjacent_cells.north = north_index;
            adjacent_cells.south = south_index;
            adjacent_cells.east = east_index;
            adjacent_cells.west = west_index;
        }

        grid
    }

    pub fn random_cell(&self) -> GridGraphNodeIndex {
        let random_pos = Range::new(0, self.grid_coordinate_to_node_index_lookup.len());
        let mut rng = rand::thread_rng();
        let index = random_pos.ind_sample(&mut rng);
        self.grid_coordinate_to_node_index_lookup[index]
    }

    /// Link two cells
    ///
    /// Todo - only allow links between adjacent cells? If `b` not in `g.neighbours(a)`.
    ///      - better to change the API to take an index and GridDirection
    ///
    /// Panics if a cell does not exist.
    pub fn link(&mut self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) {
        if a != b {
            let _ = self.graph.update_edge(a, b, ());
        }
    }

    /// Unlink two cells, if a link exists between them.
    pub fn unlink(&mut self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) {

        if let Some(edge_index) = self.graph.find_edge(a, b) {
            // This will invalidate the last edge index in the graph, which is fine as we
            // are not storing them for any reason.
            self.graph.remove_edge(edge_index);
        }
    }

    /// Cell nodes that are linked to a particular node by a passage.
    pub fn links(&self, node_index: GridGraphNodeIndex) -> Vec<GridGraphNodeIndex> {
        self.graph
            .edges(node_index)
            .map(|index_edge_data_pair| index_edge_data_pair.0.clone())
            .collect()
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    pub fn neighbours(&self, node_index: GridGraphNodeIndex) -> Vec<GridGraphNodeIndex> {

        if let Some(adj) = self.graph.node_weight(node_index) {
            vec![adj.north, adj.south, adj.east, adj.west]
                .iter()
                .filter_map(|&maybe_adj| maybe_adj.clone())
                .collect()
        } else {
            vec![]
        }
    }

    fn is_neighbour(&self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) -> bool {
        macro_rules! index_node_match {
            ($opt_grid_index:expr) => (if let Some(node_index) = $opt_grid_index {
                                           if node_index == b {
                                               return true;
                                           }
                                       })
        }
        if let Some(adjacents_of_a) = self.graph.node_weight(a) {
            index_node_match!(adjacents_of_a.north);
            index_node_match!(adjacents_of_a.south);
            index_node_match!(adjacents_of_a.east);
            index_node_match!(adjacents_of_a.west);
        }
        false
    }

    fn is_valid_coordinate(&self, coord: &GridCoordinate) -> bool {
        let (x, y) = (coord.x, coord.y);
        let dim_size = self.dimension_size as isize;
        if x < 0 || y < 0 || x >= dim_size || y >= dim_size {
            return false;
        }
        true
    }

    fn grid_coordinate_to_1d_index(&self, coord: &GridCoordinate) -> GridIndexType {
        ((coord.y * self.dimension_size as isize) + coord.x) as GridIndexType
    }

    fn index_to_grid_coordinate(&self, one_dimensional_index: usize) -> GridCoordinate {
        let dim_size = self.dimension_size as usize;
        let y = one_dimensional_index / dim_size;
        let x = one_dimensional_index - (y * dim_size);
        GridCoordinate {
            x: x as isize,
            y: y as isize,
        }
    }
}

fn offset_coordinate(coord: &GridCoordinate, dir: GridDirection) -> GridCoordinate {
    let (x, y) = (coord.x, coord.y);
    match dir {
        GridDirection::North => GridCoordinate { x: x, y: y - 1 },
        GridDirection::South => GridCoordinate { x: x, y: y + 1 },
        GridDirection::East => GridCoordinate { x: x + 1, y: y },
        GridDirection::West => GridCoordinate { x: x - 1, y: y },
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use itertools::Itertools; // a trait

    #[test]
    fn neighbour_cells() {
        let g = SquareGrid::new(10);

        let check_expected_neighbours = |node_index, vec_expected_neighbour_indices: Vec<usize>| {
            let node_indices = g.neighbours(GridGraphNodeIndex::new(node_index))
                                .into_iter()
                                .sorted();
            let expected_indices =
                vec_expected_neighbour_indices.into_iter()
                                              .sorted()
                                              .into_iter()
                                              .map(|index: usize| GridGraphNodeIndex::new(index))
                                              .collect::<Vec<GridGraphNodeIndex>>();
            assert_eq!(node_indices, expected_indices);
        };

        // corners
        check_expected_neighbours(0, vec![1, 10]);
        check_expected_neighbours(9, vec![8, 19]);
        check_expected_neighbours(90, vec![80, 91]);
        check_expected_neighbours(99, vec![89, 98]);

        // side element examples
        check_expected_neighbours(10, vec![0, 11, 20]);
        check_expected_neighbours(1, vec![0, 2, 11]);
        check_expected_neighbours(80, vec![70, 81, 90]);
        check_expected_neighbours(89, vec![79, 88, 99]);
        check_expected_neighbours(19, vec![9, 18, 29]);
        check_expected_neighbours(8, vec![7, 9, 18]);

        // Some place with 4 neighbours inside the grid
        check_expected_neighbours(12, vec![2, 11, 13, 22]);
    }

    #[test]
    fn random_cell() {
        let g = SquareGrid::new(4);
        let cells_count = 4 * 4;
        for _ in 0..1000 {
            let grid_index = g.random_cell();
            assert!(grid_index.index() >= 0 && grid_index.index() < cells_count);
        }
    }

    #[test]
    fn linking_cells() {
        let mut g = SquareGrid::new(4);
        let a = GridGraphNodeIndex::new(0);
        let b = GridGraphNodeIndex::new(1);
        let c = GridGraphNodeIndex::new(2);

        // I'd rather use a closure, but it needs to borrow the graph immutably until
        // it goes out of scope
        // Passing the graph in allows me to use a plain function:
        // let links_sorted = |n: GridGraphNodeIndex, g: &SquareGrid| -> Vec<GridGraphNodeIndex> {
        //     g.links(n).into_iter().sorted()
        // };
        // but it's uglier in that I need to explicitly pass in the SquareGrid ref all the time
        macro_rules! links_sorted {
            ($x:expr) => (g.links($x).into_iter().sorted())
        }

        // a, b and c start with no links
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);

        g.link(a, b);
        // a - b linked bi-directionally
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a]);

        g.link(b, c);
        // a - b still linked bi-directionally after linking b - c
        // b linked to a & c bi-directionally
        // c linked to b bi-directionally
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a, c]);
        assert_eq!(links_sorted!(c), vec![b]);

        // add the same link - ensuring we can't add parallel links
        g.link(a, b);

        // a - b still linked bi-directionally after updating exist link
        assert_eq!(links_sorted!(a), vec![b]);
        assert_eq!(links_sorted!(b), vec![a, c]);

        // a - b unlinked
        // b still linked to c bi-directionally
        g.unlink(a, b);
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![c]);
        assert_eq!(links_sorted!(c), vec![b]);

        // a, b and c start all unlinked again
        g.unlink(b, c);
        assert_eq!(links_sorted!(a), vec![]);
        assert_eq!(links_sorted!(b), vec![]);
        assert_eq!(links_sorted!(c), vec![]);

        // Deny cycle - self links
        g.link(a, a);
        assert_eq!(links_sorted!(a), vec![]);
    }
}
