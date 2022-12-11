use advent_of_code::helpers::{
    grids::{LineIterator, SquareGrid},
    parsing::{generic_error_for_input, ParsingResult},
};

fn parse_tree_heights<'a, const S: usize>(input: &'a [u8]) -> ParsingResult<SquareGrid<u8, S>> {
    let mut heights = [[0; S]; S];
    let mut row = 0;
    let mut col = 0;
    for c in input {
        match c {
            b'\n' => {
                col = 0;
                row += 1;
            }
            (b'0'..=b'9') => {
                heights[row][col] = c - b'0';
                col += 1;
            }
            _ => {
                return generic_error_for_input(input);
            }
        }
    }
    return Ok((&input[(S * S)..], SquareGrid::from_array(heights)));
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct LocationAndHeight {
    row: usize,
    col: usize,
    height: Option<u8>,
}

impl LocationAndHeight {
    fn initial_state() -> LocationAndHeight {
        LocationAndHeight {
            row: 0,
            col: 0,
            height: None,
        }
    }

    fn lookup<const S: usize>(
        row: usize,
        col: usize,
        tree_grid: &SquareGrid<u8, S>,
    ) -> LocationAndHeight {
        LocationAndHeight {
            row,
            col,
            height: Some(tree_grid[row][col]),
        }
    }
}

fn find_visible_trees_along_line<'a, const S: usize>(
    line_iterator: LineIterator<S>,
    tree_grid: &'a SquareGrid<u8, S>,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    line_iterator
        .map(|(row, col)| LocationAndHeight::lookup(row, col, tree_grid))
        .scan(
            LocationAndHeight::initial_state(),
            |current_highest, item| {
                if let Some(current_highest_height) = current_highest.height {
                    if current_highest_height >= item.height.unwrap() {
                        Some(None)
                    } else {
                        *current_highest = item;
                        Some(Some(item))
                    }
                } else {
                    *current_highest = item;
                    Some(Some(item))
                }
            },
        )
        .filter_map(|t| t)
        .map(|location_and_height| (location_and_height.row, location_and_height.col))
}

pub fn part_one(input: &str) -> Option<u32> {
    let tree_grid = parse_tree_heights::<99>(input.as_bytes()).unwrap().1;
    let edge_iterators = (1..98).flat_map(|i| {
        let line_iterators = [
            LineIterator::get_line_from_top(i),
            LineIterator::get_line_from_bottom(i),
            LineIterator::get_line_from_left(i),
            LineIterator::get_line_from_right(i),
        ];
        line_iterators
            .into_iter()
            .flat_map(|line_iterator| find_visible_trees_along_line(line_iterator, &tree_grid))
    });

    let mut visibilities = SquareGrid::from_array([[false; 99]; 99]);
    edge_iterators.for_each(|(row, col)| {
        visibilities[row][col] = true;
    });
    let exterior_count: u32 = 4;
    let inner_count: u32 = visibilities
        .iter()
        .flat_map(|r| r.iter())
        .map(|v| if *v { 1 } else { 0 })
        .sum();
    Some(inner_count + exterior_count)
}

#[derive(Debug)]
struct TreeView {
    latest_index_of_at_least_height: [usize; 10],
}

impl TreeView {
    fn new() -> Self {
        TreeView {
            latest_index_of_at_least_height: [0; 10],
        }
    }

    fn add_tree_of_height(&mut self, height: u8, at_index: usize) {
        assert!((0..=9).contains(&height));
        for i in 0..=height {
            self.latest_index_of_at_least_height[i as usize] = at_index;
        }
    }

    fn view_distance_from_height(&self, height: u8, view_index: usize) -> usize {
        view_index - self.latest_index_of_at_least_height[height as usize]
    }
}

fn view_distances_in_direction<const S: usize>(
    tree_grid: &SquareGrid<u8, S>,
    line_iter_fn: &fn(usize) -> LineIterator<S>,
) -> SquareGrid<u32, S> {
    let mut view_distances = SquareGrid::new_with(0u32);
    for i in 0..S {
        line_iter_fn(i)
            .enumerate()
            .scan(
                TreeView::new(),
                |current_view, (current_index, (row, col))| {
                    let height = tree_grid[row][col];
                    let view_distance =
                        current_view.view_distance_from_height(height, current_index) as u32;
                    current_view.add_tree_of_height(height, current_index);
                    Some((view_distance, row, col))
                },
            )
            .for_each(|(distance, row, col)| view_distances[row][col] = distance)
    }
    view_distances
}

pub fn part_two<const S: usize>(input: &str) -> Option<u32> {
    let tree_grid = parse_tree_heights::<S>(input.as_bytes()).unwrap().1;
    let line_iterator_fns = [
        |i| LineIterator::<S>::get_line_from_top(i),
        |i| LineIterator::<S>::get_line_from_bottom(i),
        |i| LineIterator::<S>::get_line_from_left(i),
        |i| LineIterator::<S>::get_line_from_right(i),
    ];
    Some(
        *line_iterator_fns
            .iter()
            .map(|iter_fn| view_distances_in_direction(&tree_grid, iter_fn))
            .reduce(|a, b| a * b)
            .expect("No view grids generated")
            .iter()
            .flat_map(|row| row.iter())
            .max()
            .expect("View must have a maximum"),
    )
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 8);
    advent_of_code::solve!(1, part_one, input);
    let part_two_sized = part_two::<99>;
    advent_of_code::solve!(2, part_two_sized, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 8);
        assert_eq!(part_two::<5>(&input).unwrap(), 8);
    }
}

