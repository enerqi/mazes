
use petgraph::graph::IndexType;
use rand;
use rand::Rng;
use squaregrid::{SquareGrid, GridDirection, GridCoordinate};

/// Apply the binary tree maze algorithm to a grid
/// It works simply by visiting each cell in the grid and choosing to carve a passage either north or east.
///
pub fn apply<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
    let mut rng = rand::thread_rng();

    for cell_coord in grid.iter() {

        // Get the neighbours to the north and east of this cell
        let neighbours = grid.neighbours_at_directions(&cell_coord,
                                                       &vec![GridDirection::North,
                                                             GridDirection::East])
                             .into_iter()
                             .filter_map(|coord_maybe| coord_maybe)
                             .collect::<Vec<GridCoordinate>>();

        // Randomly choose the north or east neighbour and create a passage to it
        if !neighbours.is_empty() {

            let link_coord = if neighbours.len() > 1 {
                neighbours[rng.gen::<usize>() % neighbours.len()]
            } else {
                neighbours[0]
            };

            grid.link(cell_coord, link_coord);
        }
    }
}
