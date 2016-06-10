use bit_set::BitSet;
use rand;
use rand::Rng;
use smallvec::SmallVec;

use coordinates::Cartesian2DCoordinate;
use masks::BinaryMask2D;
use grids::{CoordinateSmallVec, GridDirection, IndexType, SquareGrid};
use grids;
use utils;
use utils::FnvHashSet;

/// Apply the binary tree maze generation algorithm to a grid
/// It works simply by visiting each cell in the grid and choosing to carve a passage
/// in one of two perpendicular directions.
/// Once picked, the two perpendicular directions are constant for the entire maze generation process,
/// otherwise we'd have a good way for generating many areas with no way in or out. We would not be
/// generating a perfect maze.
pub fn binary_tree<GridIndexType>(grid: &mut SquareGrid<GridIndexType>)
    where GridIndexType: IndexType
{
    let mut rng = rand::weak_rng();
    let neighbours_to_check = two_perpendicular_directions(&mut rng);

    for cell_coord in grid.iter() {

        // Get the neighbours perpendicular to this cell
        let neighbours = grid.neighbours_at_directions(cell_coord, &neighbours_to_check)
            .iter()
            .filter_map(|coord_maybe| *coord_maybe)
            .collect::<CoordinateSmallVec>();

        // Unless there are no neighbours, randomly choose a neighbour to connect.
        if !neighbours.is_empty() {

            let neighbours_count = neighbours.len();
            let link_coord = match neighbours_count {
                1 => neighbours[0],
                _ => neighbours[rng.gen::<usize>() % neighbours_count],
            };

            grid.link(cell_coord, link_coord).expect("Failed to link a cell to its neighbour");
        }
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
    let mut rng = rand::weak_rng();

    let runs_are_horizontal = rng.gen();
    let (next_in_run_direction, run_close_out_direction, batch_iter) = if runs_are_horizontal {
        (GridDirection::East, rand_vertical_direction(&mut rng), grid.iter_row())
    } else {
        (GridDirection::South, rand_horizontal_direction(&mut rng), grid.iter_column())
    };

    for coordinates_line in batch_iter {
        let mut run = SmallVec::<[&Cartesian2DCoordinate; 12]>::new(); // 1/5000 chance to get a run of 12 coin flips. SmallVec is still growable.

        for coord in &coordinates_line {
            run.push(coord);

            let next_in_run_cell = grid.neighbour_at_direction(*coord, next_in_run_direction);
            let at_run_end_boundary = next_in_run_cell.is_none();
            let at_close_out_direction_boundary =
                grid.neighbour_at_direction(*coord, run_close_out_direction)
                    .is_none();

            let should_close_out = at_run_end_boundary ||
                                   (!at_close_out_direction_boundary && rng.gen()); // coin flip

            if should_close_out {
                let sample = rng.gen::<usize>() % run.len();
                let run_member = run[sample];

                let close_out_dir =
                    grid.neighbour_at_direction(*run_member, run_close_out_direction);
                if let Some(close_out_coord) = close_out_dir {
                    grid.link(*run_member, close_out_coord)
                        .expect("Failed to link a cell to close out a run.");
                }
                run.clear();
            } else {
                grid.link(*coord, next_in_run_cell.unwrap())
                    .expect("Failed to link a cell to the next cell in a run.");
            }
        }
    }
}

/// Apply the Aldous-Broder random walk maze generation algorithm to the grid.
/// Randomly walk from one cell to another until all have been visited. A new cell
/// in the walk is linked to the previous one in the walks path whenever it is unvisited.
/// Warning: can be painfully slow to visit all cells in a large grid due to the pure random walking.
///
/// Todo: handle masks that have walled off unreachable areas, making some unmasked cells unvisitable
///       and causing the algorithm to run forever.
pub fn aldous_broder<GridIndexType>(grid: &mut SquareGrid<GridIndexType>,
                                    mask: Option<&BinaryMask2D>)
    where GridIndexType: IndexType
{
    let cells_count = grid.size();
    let unmasked_count = unmasked_cells_count(&grid, mask);
    let mut rng = rand::weak_rng();

    let current_cell_opt = random_cell(&grid, mask.map(|m| (m, unmasked_count)), &mut rng);
    if current_cell_opt.is_none() {
        return;
    }

    // We may not need a bit set that large, but we want to keep the bit_index mapping predictable.
    let mut visited_cells = BitSet::with_capacity(cells_count);
    let mut visited_count = 0;

    let mut current_cell = current_cell_opt.unwrap();

    visit_cell(current_cell,
               &mut visited_cells,
               Some(&mut visited_count),
               &grid);

    while visited_count < unmasked_count {

        let next_cell = if let Some(m) = mask {
            random_unmasked_neighbour(current_cell, &grid, &m, &mut rng)
        } else {
            random_neighbour(current_cell, &grid, &mut rng)
        };

        // The random neighbour may not return a new cell that we can go to it, but it
        // will at least eventually backtrack.
        // random_unmasked_neighbour should achieve the same, even if the only unmasked neighbour
        // is backtracking to a previously visited cell
        if let Some(new_cell) = next_cell {

            if !is_cell_in_visited_set(new_cell, &visited_cells, &grid) {

                grid.link(current_cell, new_cell)
                    .expect("Failed to link a cell on random walk.");

                visit_cell(new_cell,
                           &mut visited_cells,
                           Some(&mut visited_count),
                           &grid);
            }

            current_cell = new_cell;
        }
    }
}

/// Todo: handle masks that have walled off unreachable areas, making some unmasked cells unvisitable
///       and causing the algorithm to run forever.
pub fn wilson<GridIndexType>(grid: &mut SquareGrid<GridIndexType>, mask: Option<&BinaryMask2D>)
    where GridIndexType: IndexType
{
    let unmasked_count = unmasked_cells_count(&grid, mask);
    let mask_with_unmasked_count: Option<(&BinaryMask2D, usize)> =
        mask.map(|m| (m, unmasked_count));
    let mut rng = rand::weak_rng();

    let start_cell = random_cell(&grid, mask_with_unmasked_count, &mut rng);
    if start_cell.is_none() {
        return;
    }

    let cells_count = grid.size();
    // We may not need a bit set that large, but we want to keep the bit_index mapping predictable.
    let mut visited_cells = BitSet::with_capacity(cells_count);
    let mut visited_count = 0;

    // Visit one cell randomly to start things off
    visit_cell(start_cell.unwrap(),
               &mut visited_cells,
               Some(&mut visited_count),
               &grid);

    // Need to keep the current walk's path, preferably with a quick way to check if a new cell forms a loop with the path.
    // The path is a sequence, i.e. Vec/Stack, but we want a quick way to look up if any particular coordinate is in that path.
    let mut cells_on_random_walk: FnvHashSet<Cartesian2DCoordinate> =
        utils::fnv_hashset(grid.dimension() as usize);
    let mut random_walk_path: Vec<Cartesian2DCoordinate> = Vec::new();

    while visited_count < unmasked_count {

        // A loop erased random walk until any visited cell is encountered
        // Keep walking randomly until we find a visited cell then link up all the cells on the path to the visited cell found.
        cells_on_random_walk.clear();
        random_walk_path.clear();

        let walk_start_cell = random_unvisited_unmasked_cell(&grid,
                                                             Some((&visited_cells, visited_count)),
                                                             mask_with_unmasked_count,
                                                             &mut rng)
            .expect("Error exhausted unmasked/unvisited cells");
        random_walk_path.push(walk_start_cell);
        cells_on_random_walk.insert(walk_start_cell);

        'walking: loop {

            let current_walk_cell = random_walk_path.last().unwrap().clone();

            if is_cell_in_visited_set(current_walk_cell, &visited_cells, &grid) {

                // We have a completed random walk path
                // Link up the cells and visit them.
                for (walk_index, cell) in random_walk_path.iter().enumerate() {

                    visit_cell(*cell, &mut visited_cells, Some(&mut visited_count), &grid);

                    if walk_index > 0 {

                        let path_previous_cell = random_walk_path[walk_index - 1];
                        grid.link(*cell, path_previous_cell)
                            .expect("Failed to link a cell on loop erased random walk path.");
                    }
                }

                // Look to start a new walk if there are any unvisited cells
                break 'walking;

            } else {

                // Still randomly walking...
                let walk_next = if let Some(m) = mask {
                    random_unmasked_neighbour(current_walk_cell, &grid, &m, &mut rng)
                } else {
                    random_neighbour(current_walk_cell, &grid, &mut rng)
                };

                if let Some(new_cell) = walk_next {

                    // We have new cell that is within the bounds of the maze grid and not masked...
                    if cells_on_random_walk.contains(&new_cell) {

                        // There is a loop in the current walk, erase it by dropping the path after this point.
                        // We also have to remove the dropped cells from the bitset
                        let loop_start_index = random_walk_path.iter()
                            .position(|walk_cell| *walk_cell == new_cell)
                            .unwrap();
                        let altered_path_length = loop_start_index + 1;
                        for cell in random_walk_path.iter().skip(altered_path_length) {
                            cells_on_random_walk.remove(cell);
                        }
                        random_walk_path.truncate(altered_path_length);

                    } else {

                        // Extend the walk
                        random_walk_path.push(new_cell);
                        cells_on_random_walk.insert(new_cell);
                    }
                }
            }
        }
    }
}

/// Generates a maze with lots of "river"/meandering - that is long runs before you encounter a dead end.
/// Memory efficient - little beyond the grid to maintain.
/// Compute challenged - visits every cells 2+ times, once in the walk and again in hunt phase.
/// Executing the hunt phase many times can visit a cell many times.
pub fn hunt_and_kill<GridIndexType>(grid: &mut SquareGrid<GridIndexType>,
                                    mask: Option<&BinaryMask2D>)
    where GridIndexType: IndexType
{
    let unmasked_count = unmasked_cells_count(&grid, mask);
    let mask_with_unmasked_count: Option<(&BinaryMask2D, usize)> =
        mask.map(|m| (m, unmasked_count));
    let mut rng = rand::weak_rng();

    let start_cell = random_cell(&grid, mask_with_unmasked_count, &mut rng);
    if start_cell.is_none() {
        return;
    }
    let mut current_cell = start_cell.unwrap();

    let cells_count = grid.size();

    // We may not need a bit set that large, but we want to keep the bit_index mapping predictable.
    let mut visited_cells = BitSet::with_capacity(cells_count);
    let mut visited_count = 0;

    let is_any_neighbour_visited =
        |cell, visited_set: &BitSet, grid: &SquareGrid<GridIndexType>| -> bool {
            grid.neighbours(cell)
                .iter()
                .any(|c| is_cell_in_visited_set(*c, &visited_set, &grid))
        };

    let visited_neighbours = |cell: Cartesian2DCoordinate,
                              visited_set: &BitSet,
                              grid: &SquareGrid<GridIndexType>|
                              -> Option<CoordinateSmallVec> {
        let vn: CoordinateSmallVec = grid.neighbours(cell)
            .iter()
            .cloned()
            .filter(|c| is_cell_in_visited_set(*c, &visited_set, &grid))
            .collect();
        if vn.is_empty() {
            None
        } else {
            Some(vn)
        }
    };

    let are_all_neighbours_visited_or_masked = |cell,
                                                visited_set: &BitSet,
                                                grid: &SquareGrid<GridIndexType>,
                                                mask: Option<&BinaryMask2D>|
                                                -> bool {
        if let Some(m) = mask {
            grid.neighbours(cell)
                .iter()
                .all(|c| is_cell_in_visited_set(*c, &visited_set, &grid) || m.is_masked(*c))
        } else {
            grid.neighbours(cell).iter().all(|c| is_cell_in_visited_set(*c, &visited_set, &grid))
        }
    };

    visit_cell(current_cell,
               &mut visited_cells,
               Some(&mut visited_count),
               &grid);

    while visited_count < unmasked_count {

        let next_cell = if let Some(m) = mask {
            random_unmasked_neighbour(current_cell, &grid, &m, &mut rng)
        } else {
            random_neighbour(current_cell, &grid, &mut rng)
        };

        if let Some(new_cell) = next_cell {

            if !is_cell_in_visited_set(new_cell, &visited_cells, &grid) {

                grid.link(current_cell, new_cell)
                    .expect("Failed to link a cell on random walk.");

                visit_cell(new_cell,
                           &mut visited_cells,
                           Some(&mut visited_count),
                           &grid);

                current_cell = new_cell;

            } else {

                // The new_cell has been seen before, we are not allowed to go here...
                // We will just try another random neighbour unless there are no unvisited neighbours
                // in which case we take special steps to find one
                if are_all_neighbours_visited_or_masked(current_cell, &visited_cells, &grid, mask) {

                    // There are no other unvisited cells next to this
                    // Start from (0,0) in the grid and find the first *unvisited* cell that is next to a visited one.
                    let (hunted_cell, hunteds_visited_neighbours): (Cartesian2DCoordinate, CoordinateSmallVec) =
                        grid.iter()
                            .skip_while(|cell| is_cell_in_visited_set(*cell, &visited_cells, &grid) ||
                                               mask.map_or(false, |m| m.is_masked(*cell)) ||
                                               !is_any_neighbour_visited(*cell, &visited_cells, &grid))
                            .take(1)
                            .fold(None, |_, cell|
                                        Some((cell, visited_neighbours(cell, &visited_cells, &grid)
                                              .expect("This cell should have 1+ visited neighbours"))))
                            .expect("We should always be able to find a cell in the grid with at least one visited neighbour.");

                    // Link the hunted_cell to any random neighbour that is visited
                    // Visit the hunted cell and make it the new current cell in the walk
                    let random_visited_neighbour =
                        hunteds_visited_neighbours[rng.gen::<usize>() %
                                                   hunteds_visited_neighbours.len()];
                    grid.link(hunted_cell, random_visited_neighbour)
                        .expect("Failed to link the hunted cell to a random visited neighbour.");
                    visit_cell(hunted_cell,
                               &mut visited_cells,
                               Some(&mut visited_count),
                               &grid);
                    current_cell = hunted_cell;
                }
            }
        }
    }
}

/// aka Depth First Search
/// Generates a maze with lots of "river"/meandering - that is long runs before you encounter a dead end.
/// Compute efficient - visits each cell exactly twice
/// Memory challenged - the search stack can get very deep, up to grid size deep.
pub fn recursive_backtracker<GridIndexType>(grid: &mut SquareGrid<GridIndexType>,
                                            mask: Option<&BinaryMask2D>)
    where GridIndexType: IndexType
{
    let mut rng = rand::weak_rng();
    let cells_count = grid.size();
    let unmasked_count = unmasked_cells_count(&grid, mask);

    let start_cell_opt = random_cell(&grid, mask.map(|m| (m, unmasked_count)), &mut rng);
    if start_cell_opt.is_none() {
        return;
    }

    let start_cell = start_cell_opt.unwrap();
    // We may not need a bit set that large, but we want to keep the bit_index mapping predictable.
    let mut visited_cells = BitSet::with_capacity(cells_count);
    let mut dfs_stack = vec![start_cell];

    let unvisited_neighbours = |cell: Cartesian2DCoordinate,
                                visited_set: &BitSet,
                                grid: &SquareGrid<GridIndexType>|
                                -> Option<CoordinateSmallVec> {

        let vn: CoordinateSmallVec = if let Some(m) = mask {
            grid.neighbours(cell)
                .iter()
                .cloned()
                .filter(|c: &Cartesian2DCoordinate| {
                    !is_cell_in_visited_set(*c, &visited_set, &grid) && !m.is_masked(*c)
                })
                .collect()
        } else {
            grid.neighbours(cell)
                .iter()
                .cloned()
                .filter(|c: &Cartesian2DCoordinate| !is_cell_in_visited_set(*c, &visited_set, &grid))
                .collect()
        };

        if vn.is_empty() {
            None
        } else {
            Some(vn)
        }
    };

    while !dfs_stack.is_empty() {

        let cell = *dfs_stack.last().expect("dfs stack should not be empty");
        visit_cell(cell, &mut visited_cells, None, &grid);

        let unvisited_neighbours_opt = unvisited_neighbours(cell, &visited_cells, &grid);

        if let Some(unvisited) = unvisited_neighbours_opt {

            let unvisited_count = unvisited.len();
            let next_cell = match unvisited_count {
                1 => unvisited[0],
                _ => unvisited[rng.gen::<usize>() % unvisited_count],
            };

            grid.link(cell, next_cell)
                .expect("Failed to link cells in depth first search walk.");
            dfs_stack.push(next_cell);

        } else {

            dfs_stack.pop();
        }
    }
}

fn two_perpendicular_directions<R: Rng>(rng: &mut R) -> [GridDirection; 2] {
    [rand_vertical_direction(rng), rand_horizontal_direction(rng)]
}

fn rand_vertical_direction<R: Rng>(rng: &mut R) -> GridDirection {
    if rng.gen() {
        GridDirection::North
    } else {
        GridDirection::South
    }
}

fn rand_horizontal_direction<R: Rng>(rng: &mut R) -> GridDirection {
    if rng.gen() {
        GridDirection::East
    } else {
        GridDirection::West
    }
}

fn rand_direction<R: Rng>(rng: &mut R) -> GridDirection {
    const DIRS_COUNT: usize = 4;
    const DIRS: [GridDirection; DIRS_COUNT] =
        [GridDirection::North, GridDirection::South, GridDirection::East, GridDirection::West];
    let dir_index = rng.gen::<usize>() % DIRS_COUNT;
    DIRS[dir_index]
}

fn random_neighbour<GridIndexType, R>(cell: Cartesian2DCoordinate,
                                      grid: &SquareGrid<GridIndexType>,
                                      mut rng: &mut R)
                                      -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{
    grid.neighbour_at_direction(cell, rand_direction(&mut rng))
}

fn random_cell<GridIndexType, R>(grid: &SquareGrid<GridIndexType>,
                                 mask_with_unmasked_count: Option<(&BinaryMask2D, usize)>,
                                 mut rng: &mut R)
                                 -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{
    if let Some(mask_and_count) = mask_with_unmasked_count {
        random_unmasked_cell(&grid, mask_and_count, &mut rng)
    } else {
        Some(grid.random_cell(&mut rng))
    }
}

fn bit_index<GridIndexType>(cell: Cartesian2DCoordinate, grid: &SquareGrid<GridIndexType>) -> usize
    where GridIndexType: IndexType
{
    grid.grid_coordinate_to_index(cell)
        .expect("bit_index impossible: invalid cell")
}

fn is_cell_in_visited_set<GridIndexType>(cell: Cartesian2DCoordinate,
                                         visited_set: &BitSet,
                                         grid: &SquareGrid<GridIndexType>)
                                         -> bool
    where GridIndexType: IndexType
{
    visited_set.contains(bit_index(cell, &grid))
}

fn visit_cell<GridIndexType>(cell: Cartesian2DCoordinate,
                             visited_set: &mut BitSet,
                             visited_count: Option<&mut usize>,
                             grid: &SquareGrid<GridIndexType>)
                             -> bool
    where GridIndexType: IndexType
{
    let is_previously_unvisited = visited_set.insert(bit_index(cell, &grid));
    if let Some(count) = visited_count {

        if is_previously_unvisited {
            *count += 1;
        }
        is_previously_unvisited

    } else {

        is_previously_unvisited
    }
}

fn undo_cell_visit<GridIndexType>(cell: Cartesian2DCoordinate,
                                  visited_set: &mut BitSet,
                                  visited_count: Option<&mut usize>,
                                  grid: &SquareGrid<GridIndexType>)
                                  -> bool
    where GridIndexType: IndexType
{
    let index = bit_index(cell, &grid);
    let was_present = visited_set.remove(index);

    if was_present {

        if let Some(count) = visited_count {
            *count -= 1;
        }
    }

    was_present
}

fn random_unvisited_cell<GridIndexType, R>(grid: &SquareGrid<GridIndexType>,
                                           visited_set_with_count: (&BitSet, usize),
                                           mut rng: &mut R)
                                           -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{
    let cells_count = grid.size();
    let (visited_set, visited_count) = visited_set_with_count;
    let remaining_unvisited_count = cells_count - visited_count;
    if remaining_unvisited_count > 0 {

        let n = rng.gen::<usize>() % remaining_unvisited_count;

        let cell_index = (0..cells_count)
            .filter(|bit_index| !visited_set.contains(*bit_index))
            .nth(n)
            .unwrap();

        Some(grids::index_to_grid_coordinate(grid.dimension(), cell_index))

    } else {
        None
    }
}

fn random_unmasked_cell<GridIndexType, R>(grid: &SquareGrid<GridIndexType>,
                                          mask_with_unmasked_count: (&BinaryMask2D, usize),
                                          mut rng: &mut R)
                                          -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{
    let (mask, unmasked_cells) = mask_with_unmasked_count;
    if unmasked_cells != 0 {

        let n = rng.gen::<usize>() % unmasked_cells;
        let cells_count = grid.size();
        let cell_index = (0..cells_count)
            .filter(|i| {
                let coord = grids::index_to_grid_coordinate(grid.dimension(), *i);
                !mask.is_masked(coord)
            })
            .nth(n)
            .unwrap();
        Some(grids::index_to_grid_coordinate(grid.dimension(), cell_index))

    } else {
        None
    }
}

fn random_unvisited_unmasked_cell<GridIndexType, R>(grid: &SquareGrid<GridIndexType>,
                                                    visited_set_with_count: Option<(&BitSet,
                                                                                    usize)>,
                                                    mask_with_unmasked_count: Option<(&BinaryMask2D,
                                                                                      usize)>,
                                                    mut rng: &mut R)
                                                    -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{
    match (visited_set_with_count, mask_with_unmasked_count) {

        (None, None) => Some(grid.random_cell(&mut rng)),

        (None, Some(mask_and_count)) => random_unmasked_cell(&grid, mask_and_count, &mut rng),

        (Some(set_and_count), None) => random_unvisited_cell(&grid, set_and_count, &mut rng),

        (Some((visited, visited_count)), Some((mask, unmasked_count))) => {

            let cells_count = grid.size();
            let masked_count = cells_count - unmasked_count;
            let remaining_cells = cells_count - visited_count - masked_count;

            if remaining_cells != 0 {

                let n = rng.gen::<usize>() % remaining_cells;
                let grid_dim = grid.dimension();
                let cell_index = (0..cells_count)
                    .filter(|i| {
                        let coord = grids::index_to_grid_coordinate(grid_dim, *i);
                        !visited.contains(bit_index(coord, &grid)) && !mask.is_masked(coord)
                    })
                    .nth(n)
                    .unwrap();

                Some(grids::index_to_grid_coordinate(grid_dim, cell_index))

            } else {
                None
            }
        }
    }
}

fn random_unmasked_neighbour<GridIndexType, R>(cell: Cartesian2DCoordinate,
                                               grid: &SquareGrid<GridIndexType>,
                                               mask: &BinaryMask2D,
                                               mut rng: &mut R)
                                               -> Option<Cartesian2DCoordinate>
    where GridIndexType: IndexType,
          R: Rng
{

    let unmasked_neighbours: CoordinateSmallVec = grid.neighbours(cell)
        .iter()
        .cloned()
        .filter(|c| !mask.is_masked(*c))
        .collect();
    if !unmasked_neighbours.is_empty() {
        let count = unmasked_neighbours.len();
        let neighbour_cell = match count {
            1 => unmasked_neighbours[0],
            _ => unmasked_neighbours[rng.gen::<usize>() % count],
        };
        Some(neighbour_cell)
    } else {
        None
    }
}

fn unmasked_cells_count<GridIndexType>(grid: &SquareGrid<GridIndexType>,
                                       mask: Option<&BinaryMask2D>)
                                       -> usize
    where GridIndexType: IndexType
{
    if let Some(m) = mask {
        m.count_unmasked_within_dimensions(grid.dimension(), grid.dimension())
    } else {
        grid.size()
    }
}
