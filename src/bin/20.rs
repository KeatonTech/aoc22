use advent_of_code::helpers::parsing::{line_ending_or_eof, text_i16};
use advent_of_code::helpers::shuffle_tree::ShuffleTree;
use nom::{combinator::iterator, sequence::terminated};
use outils::types::NodeIndex;

fn parse_to_initial_tree(input: &[u8], decryption_key: isize) -> ShuffleTree<isize> {
    let mut number_iter = iterator(input, terminated(text_i16(), line_ending_or_eof()));
    number_iter
        .enumerate()
        .map(|(i, v)| {
            (i, v as isize * decryption_key)
        })
        .collect()
}

fn calculate_coordinates(from_shuffled_tree: ShuffleTree<isize>) -> isize {
    let index_of_zero = from_shuffled_tree
        .ordered_values()
        .enumerate()
        .find(|(_, v)| **v == 0)
        .unwrap()
        .0;
    let v1000 = from_shuffled_tree[from_shuffled_tree.node_at_position_wrapping(1000 + index_of_zero)];
    let v2000 = from_shuffled_tree[from_shuffled_tree.node_at_position_wrapping(2000 + index_of_zero)];
    let v3000 = from_shuffled_tree[from_shuffled_tree.node_at_position_wrapping(3000 + index_of_zero)];
    v1000 + v2000 + v3000
}

pub fn part_one(input: &str) -> Option<isize> {
    let mut shuffle_tree = parse_to_initial_tree(input.as_bytes(), 1);
    let original_nodes: Vec<NodeIndex> = shuffle_tree.node_indexes().collect();
    for node in original_nodes {
        let node_value = shuffle_tree[node];
        shuffle_tree.move_by_wrapping(node, node_value);
    }
    Some(calculate_coordinates(shuffle_tree))
}

pub fn part_two(input: &str) -> Option<isize> {
    let mut shuffle_tree = parse_to_initial_tree(input.as_bytes(), 811589153);
    let original_nodes: Vec<NodeIndex> = shuffle_tree.node_indexes().collect();
    for _i in 0..10 {
        for node in original_nodes.iter() {
            let node_value = shuffle_tree[*node];
            shuffle_tree.move_by_wrapping(*node, node_value);
        }
    }
    Some(calculate_coordinates(shuffle_tree))
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 20);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 20);
        assert_eq!(part_one(&input), Some(3));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 20);
        assert_eq!(part_two(&input), None);
    }
}
