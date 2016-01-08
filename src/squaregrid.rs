use petgraph::{Graph, Undirected};
use rand;
use rand::distributions::{IndependentSample, Range};

enum GridDirection {
    North,
    South,
    East,
    West,
}

type GridIndexType = u16;
type GridGraphNodeIndex = ::petgraph::graph::NodeIndex<GridIndexType>;

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
    graph_node_indices: Vec<GridGraphNodeIndex>,
    graph: Graph<AdjacentCells, (), Undirected, GridIndexType>,
}

impl SquareGrid {
    fn new(cells_count: GridIndexType) -> Result<SquareGrid, String> {
        let dim_size: GridIndexType = try!(dimension_size(cells_count));
        let nodes_count_hint = cells_count as usize;
        let edges_count_hint = 4 * cells_count as usize - 4 * dim_size as usize;

        let mut grid = SquareGrid {
            graph_node_indices: Vec::with_capacity(nodes_count_hint),
            graph: Graph::with_capacity(nodes_count_hint, edges_count_hint),
        };

        // Although it does seem to, there is no guarantee that the index values are monotonically increasing when calling graph.add_node
        // So we maintain our own symbol table mapping gridcoordinates converted to 1d (the implicit Vec indices) to the NodeIndexes created
        // by adding a node to the graph.
        for _ in 0..cells_count {
            let node_index = grid.graph.add_node(AdjacentCells::new());
            grid.graph_node_indices.push(node_index);
        }

        for (index, node_index) in grid.graph_node_indices.iter().enumerate() {
            let y = index / dim_size as usize;
            let x = (y * dim_size as usize) - index;
            let coord = GridCoordinate {
                x: x as isize,
                y: y as isize,
            };

            // limit the lifetime of the find_neighbour_nodeindex closure so we don't get grid borrow conflict
            let (north_index, south_index, east_index, west_index) = {
                let find_neighbour_nodeindex = |dir: GridDirection| -> Option<GridGraphNodeIndex> {
                    let neighbour_coord = offset_coordinate(&coord, dir);
                    if is_valid_coordinate(&neighbour_coord, dim_size) {
                        let node_indices_index = to_1D_index(&neighbour_coord, dim_size);
                        let node_index = grid.graph_node_indices[node_indices_index as usize];
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

            let this_node_data: &mut AdjacentCells = grid.graph
                                                         .node_weight_mut(node_index.clone())
                                                         .unwrap();
        }

        Ok(grid)
    }

    fn random_cell(&self) -> GridGraphNodeIndex {
        let random_pos = Range::new(0, self.graph_node_indices.len());
        let mut rng = rand::thread_rng();
        let index = random_pos.ind_sample(&mut rng);
        self.graph_node_indices[index]
    }


    // The link and unlink implementation is complicated by add_edge returning an edge index
    // which is required to later unlink the nodes. This is needed because of parallel edge support
    // in the petgraph::Graph.
    fn link(&mut self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) {
        //self.graph.add_edge(a, b, ());
    }

    fn unlink(&mut self, a: GridGraphNodeIndex, b: GridGraphNodeIndex) {
        //self.graph.remove_edge(edge_index)
    }

    /// Cell nodes that are linked to a particular node by a passage.
    fn links(&self, node_index: GridGraphNodeIndex) -> Vec<GridGraphNodeIndex> {
        self.graph
            .edges(node_index)
            .map(|index_edge_data_pair| {index_edge_data_pair.0.clone()})
            .collect()
    }

    /// Cell nodes that are to the North, South, East or West of a particular node, but not
    /// necessarily linked by a passage.
    fn neighbours(&self, node_index: GridGraphNodeIndex) -> Vec<GridGraphNodeIndex> {

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

fn dimension_size(grid_cells_count: GridIndexType) -> Result<GridIndexType, String> {

    let dim_size = (grid_cells_count as f64).sqrt().trunc() as GridIndexType;
    if dim_size * dim_size == grid_cells_count {
        Ok(dim_size)
    } else {
        Err(format!("For a square grid the integer square root of the grid_cells_count {} should \
                     equal grid_cells_count.",
                    grid_cells_count))
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

fn is_valid_coordinate(coord: &GridCoordinate, dim_size: GridIndexType) -> bool {
    if coord.x < 0 || coord.y < 0 {
        return false;
    }
    let (x, y) = (coord.x, coord.y);
    if x >= dim_size as isize || y >= dim_size as isize {
        return false;
    }
    true
}

fn to_1D_index(coord: &GridCoordinate, dim_size: GridIndexType) -> GridIndexType {
    ((coord.y * dim_size as isize) + coord.x) as GridIndexType
}
