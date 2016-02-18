use petgraph::graph::IndexType;
use rand;
use rand::{Rng, ThreadRng};
use squaregrid::{SquareGrid, GridDirection};

/// Apply the binary tree maze generation algorithm to a grid
/// It works simply by visiting each cell in the grid and choosing to carve a passage
/// in one of two perpendicular directions.
/// Once picked, the two perpendicular directions are constant for the entire maze generation process,
/// otherwise we'd have a good way for generating many areas with no way in or out. We would not be
/// generating a perfect maze.
pub fn binary_tree<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
    let mut rng = rand::thread_rng();
    let neighbours_to_check = two_perpendicular_directions(&mut rng);

    for cell_coord in grid.iter() {

        // Get the neighbours perpendicular to this cell
        let neighbours = grid.neighbours_at_directions(&cell_coord, &neighbours_to_check)
                             .into_iter()
                             .filter_map(|coord_maybe| coord_maybe)
                             .collect::<Vec<_>>();

        // Unless there are no neighbours, randomly choose a neighbour to connect.
        if !neighbours.is_empty() {

            let link_coord = match neighbours.len() {
                1 => unsafe { *neighbours.get_unchecked(0) }, // * unsafe stuff doesn't get auto deref
                2 => unsafe { *neighbours.get_unchecked(rng.gen::<usize>() % 2) },
                _ => panic!("Should only have a maximum of 2 neighbours to check."),
            };

            grid.link(cell_coord, link_coord);
        }
    }
}

fn two_perpendicular_directions(rng: &mut ThreadRng) -> [GridDirection; 2] {
    [rand_vertical_direction(rng), rand_horizontal_direction(rng)]
}

fn rand_vertical_direction(rng: &mut ThreadRng) -> GridDirection {
    if rng.gen() {
        GridDirection::North
    } else {
        GridDirection::South
    }
}

fn rand_horizontal_direction(rng: &mut ThreadRng) -> GridDirection {
    if rng.gen() {
        GridDirection::East
    } else {
        GridDirection::West
    }
}

/// Apply the sidewinder maze generation algorithm to the grid
/// Sidewinder prefers not to begin at any random place, it wants to start on western column and
/// move eastwards (we could of course start visiting the cells in the grid from the east side
/// and move westwards).
/// Like the simple binary tree algorithm it picks from one of two directions. The difference is
/// that one direction (e.g east/horizontal) just carves in that direction but when we pick to
/// move vertically/north we choose to carve a passage north in a random cell selected from
/// the most recent run of horizontal cells.
/// This algorithm will display a vertical bias, with most passages leading vertically/north.
/// Same as the binary tree algorithm, the two directions that we carve passages in need to be
/// perpendicular to one another, and fixed for the lifetime of the algorithm otherwise we get
/// a lot of closed off rooms in the maze.
/// Note we also end up with two big linking passages along one vertical and horizontal wall
/// if run direction does not match the order the direction/order we visit the cells in.
/// So, if we visit the cells west to east, then the wall carving run direction needs to be east.
/// The run closing out passage carving direction does not matter.
pub fn sidewinder<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
    let mut rng = rand::thread_rng();

    let runs_are_horizontal = rng.gen();
    let (next_in_run_direction, run_close_out_direction, batch_iter) = if runs_are_horizontal {
        (GridDirection::East, rand_vertical_direction(&mut rng), grid.iter_row())
    } else {
        (GridDirection::South, rand_horizontal_direction(&mut rng), grid.iter_column())
    };

    for coordinates_line in batch_iter {
        let mut run = vec![];

        for coord in &coordinates_line {
            run.push(coord);

            let next_in_run_cell = grid.neighbour_at_direction(&coord, next_in_run_direction);
            let at_run_end_boundary = next_in_run_cell.is_none();
            let at_close_out_direction_boundary =
                grid.neighbour_at_direction(&coord, run_close_out_direction)
                    .is_none();

            let should_close_out = at_run_end_boundary ||
                                   (!at_close_out_direction_boundary &&
                                    rng.gen()); // coin flip

            if should_close_out {
                let sample = rng.gen::<usize>() % run.len();
                let run_member = run[sample];

                let close_out_dir = grid.neighbour_at_direction(&run_member,
                                                                run_close_out_direction);
                if let Some(close_out_coord) = close_out_dir {
                    grid.link(*run_member, close_out_coord);
                }
                run.clear();
            } else {
                grid.link(*coord, next_in_run_cell.unwrap());
            }
        }
    }
}
