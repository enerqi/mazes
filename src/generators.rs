
use petgraph::graph::IndexType;
use rand;
use rand::Rng;
use squaregrid::{SquareGrid, GridDirection, GridCoordinate};

/// Apply the binary tree maze algorithm to a grid
/// It works simply by visiting each cell in the grid and choosing to carve a passage either north or east.
///
pub fn binary_tree<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
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

pub fn sidewinder<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
    let mut rng = rand::thread_rng();

    for row in grid.iter_row() {
        let mut run = vec![];

        for coord in &row {
            run.push(coord.clone());

            let east_cell = grid.neighbour_at_direction(&coord, GridDirection::East);
            let at_eastern_boundary = east_cell.is_none();
            let at_northern_boundary = grid.neighbour_at_direction(&coord, GridDirection::North).is_none();

            let should_close_out = at_eastern_boundary || (!at_northern_boundary && rng.gen::<usize>() % 2 == 0);

            if should_close_out {
                let sample = rng.gen::<usize>() % run.len();
                let run_member = run[sample];

                let north_of = grid.neighbour_at_direction(&run_member, GridDirection::North);
                if let Some(north_coord) = north_of {
                    grid.link(run_member.clone(), north_coord);
                }
                run.clear();
            }
            else {
                grid.link(coord.clone(), east_cell.unwrap());
            }
        }
    }
}

