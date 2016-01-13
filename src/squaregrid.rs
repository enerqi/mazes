use petgraph::{Graph, Undirected};
use rand;
use rand::distributions::{IndependentSample, Range};


pub type GridIndexType = u16;
pub type GridGraphNodeIndex = ::petgraph::graph::NodeIndex<GridIndexType>;

enum GridDirection {
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
pub struct GridCoordinate {
    pub x: isize,
    pub y: isize,
}

pub struct SquareGrid {
    /// For mapping a 2D gridcoordinate (converted to a 1D index) to a graph node index.
    grid_coordinate_to_node_index_lookup: Vec<GridGraphNodeIndex>,
    graph: Graph<AdjacentCells, (), Undirected, GridIndexType>,
}

impl SquareGrid {
    pub fn new(dimension_size: GridIndexType) -> SquareGrid {

        let cells_count = dimension_size * dimension_size;
        let nodes_count_hint = cells_count as usize;
        let edges_count_hint = 4 * cells_count as usize - 4 * dimension_size as usize; // Probably overkill, but don't want any capacity panics

        let mut grid = SquareGrid {
            grid_coordinate_to_node_index_lookup: Vec::with_capacity(nodes_count_hint),
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
        };

        // Although it does seem to, there is no guarantee that the index values are monotonically increasing when calling graph.add_node
        // So we maintain our own symbol table mapping gridcoordinates converted to 1d (the implicit Vec indices) to the NodeIndexes created
        // by adding a node to the graph.
        for _ in 0..cells_count {
            let node_index = grid.graph.add_node(AdjacentCells::new());
            grid.grid_coordinate_to_node_index_lookup.push(node_index);
        }

        for (index, node_index) in grid.grid_coordinate_to_node_index_lookup.iter().enumerate() {

            let coord = to_grid_coordinate(index, dimension_size);

            // limit the lifetime of the find_neighbour_nodeindex closure so we don't get grid borrow conflict
            let (north_index, south_index, east_index, west_index) = {
                let find_neighbour_nodeindex = |dir: GridDirection| -> Option<GridGraphNodeIndex> {
                    let neighbour_coord = offset_coordinate(&coord, dir);
                    if is_valid_coordinate(&neighbour_coord, dimension_size) {
                        let node_indices_index = to_1d_index(&neighbour_coord, dimension_size);
                        let node_index = grid.grid_coordinate_to_node_index_lookup[node_indices_index as usize];
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

    pub fn link(&mut self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) {
        let _ = self.graph.update_edge(a, b, ());
    }

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
                .filter(|&maybe_adj| maybe_adj.is_some())
                .map(|&opt| opt.unwrap().clone())
                .collect()
        } else {
            vec![]
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

fn is_valid_coordinate(coord: &GridCoordinate, dimension_size: GridIndexType) -> bool {
    if coord.x < 0 || coord.y < 0 {
        return false;
    }
    let (x, y) = (coord.x, coord.y);
    if x >= dimension_size as isize || y >= dimension_size as isize {
        return false;
    }
    true
}

fn to_1d_index(coord: &GridCoordinate, dimension_size: GridIndexType) -> GridIndexType {
    ((coord.y * dimension_size as isize) + coord.x) as GridIndexType
}

fn to_grid_coordinate(one_dimensional_index: usize, dimension_size: GridIndexType) -> GridCoordinate {
    let y = one_dimensional_index / dimension_size as usize;
    let x = (y * dimension_size as usize) - one_dimensional_index;
    GridCoordinate {
        x: x as isize,
        y: y as isize,
    }
}
