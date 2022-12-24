use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::ops::Index;

use outils::prelude::*;
use outils::{tree::bst::waatree::WeightedAaTree, types::NodeIndex};

#[derive(Debug)]
pub struct ShuffleTree<T: Debug + Default, const INTERLEAVING_BITS: usize = 48>(
    WeightedAaTree<usize, T>,
);

impl<T: Debug + Default, const INTERLEAVING_BITS: usize> FromIterator<(usize, T)>
    for ShuffleTree<T, INTERLEAVING_BITS>
{
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(into_iter: I) -> Self {
        let iter = into_iter.into_iter();
        let mut weighted_tree = WeightedAaTree::new(0);
        for (key, value) in iter {
            weighted_tree.insert_weighted((key + 1) << INTERLEAVING_BITS, value, 1);
        }
        ShuffleTree(weighted_tree)
    }
}

impl<T: Debug + Default, const INTERLEAVING_BITS: usize> Index<NodeIndex>
    for ShuffleTree<T, INTERLEAVING_BITS>
{
    type Output = T;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Debug + Default, const INTERLEAVING_BITS: usize> ShuffleTree<T, INTERLEAVING_BITS> {
    pub fn node_indexes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.0.keys().map(|(ni, _)| ni)
    }

    pub fn ordered_values(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.values().map(|(_, v)| v)
    }

    pub fn node_at_position(&self, position: usize) -> Option<NodeIndex> {
        if position
            >= *self
                .0
                .subweight(self.0.root(NodeIndex(0)).unwrap())
                .unwrap()
        {
            return None;
        }
        Some(self.node_at_position_wrapping(position))
    }

    pub fn node_at_position_wrapping(&self, position: usize) -> NodeIndex {
        let wrapped_postition = position
            % *self
                .0
                .subweight(self.0.root(NodeIndex(0)).unwrap())
                .unwrap();
        let first_node = self.0.values().next().unwrap().0;
        self.traverse_by_wrapping(first_node, wrapped_postition as isize)
    }

    pub fn move_by_wrapping(&mut self, node: NodeIndex, by: isize) {
        let current_key = *self.0.key(node).expect("Key must exist");
        let current_node_wrapped_successor = self
            .0
            .successor(node)
            .unwrap_or_else(|| self.0.first(self.0.root(node).unwrap()).unwrap());
        let current_value = self.0.remove(current_key).expect("Value must exist");
        let destination_node_index = self.traverse_by_wrapping(current_node_wrapped_successor, by);
        let destination_key = self.0.key(destination_node_index).unwrap();

        // Append to be back instead of the front. This is a circular list so it doesn't
        // matter, but appending to the back doesn't use up any 'interleaving bits'.
        let maybe_predecessor = self.0.predecessor(destination_node_index);
        if maybe_predecessor.is_none() {
            return self.append(current_value);
        }

        let predecessor_node = maybe_predecessor.unwrap();
        let destination_key_predecessor = self.0.key(predecessor_node).unwrap();
        if *destination_key == destination_key_predecessor + 1 {
            panic!("Ran out of interleaving bits. Needs reindexing.");
        }
        let new_key =
            destination_key_predecessor + ((destination_key - destination_key_predecessor) >> 1);
        self.0.insert_weighted(new_key, current_value, 1);
    }

    pub fn append(&mut self, value: T) {
        let last_node = self.0.last(self.0.root(NodeIndex(0)).unwrap()).unwrap();
        let current_max_key = self.0.key(last_node).unwrap();
        let new_max_key = current_max_key + (1 << INTERLEAVING_BITS);
        self.0.insert_weighted(new_max_key, value, 1);
    }

    pub fn prepend(&mut self, value: T) {
        let first_node = self.0.first(self.0.root(NodeIndex(0)).unwrap()).unwrap();
        let current_min_key = self.0.key(first_node).unwrap();
        if *current_min_key == 0 {
            panic!("Ran out of interleaving bits. Needs reindexing.");
        }
        let new_min_key = current_min_key >> 1;
        self.0.insert_weighted(new_min_key, value, 1);
    }

    pub fn traverse_by_wrapping(&self, from: NodeIndex, by: isize) -> NodeIndex {
        let by_wrapped = by
            % *self
                .0
                .subweight(self.0.root(NodeIndex(0)).unwrap())
                .unwrap() as isize;
        match by_wrapped {
            by_wrapped if by_wrapped > 0 => self
                .traverse_forward_wrapping(from, NonZeroUsize::new(by_wrapped as usize).unwrap()),
            by_wrapped if by_wrapped < 0 => self
                .traverse_backward_wrapping(from, NonZeroUsize::new(-by_wrapped as usize).unwrap()),
            _ => from,
        }
    }

    fn traverse_forward_wrapping(&self, from: NodeIndex, by: NonZeroUsize) -> NodeIndex {
        if let Some(right_child) = self.0.child(from, 1) {
            let child_weight = self.0.subweight(right_child).unwrap();
            if *child_weight >= by.get() {
                let child_shifts_by = self.child_distance_to_parent(right_child, 0);
                self.traverse_by_wrapping(right_child, by.get() as isize - child_shifts_by as isize)
            } else {
                self.traverse_up_wrapping(from, by.get() as isize)
            }
        } else {
            self.traverse_up_wrapping(from, by.get() as isize)
        }
    }

    fn traverse_backward_wrapping(&self, from: NodeIndex, by: NonZeroUsize) -> NodeIndex {
        if let Some(left_child) = self.0.child(from, 0) {
            let child_weight = self.0.subweight(left_child).unwrap();
            if *child_weight >= by.get() {
                let child_shifts_by = self.child_distance_to_parent(left_child, 1);
                self.traverse_by_wrapping(
                    left_child,
                    0 - by.get() as isize + child_shifts_by as isize,
                )
            } else {
                self.traverse_up_wrapping(from, 0 - by.get() as isize)
            }
        } else {
            self.traverse_up_wrapping(from, 0 - by.get() as isize)
        }
    }

    fn traverse_up_wrapping(&self, from: NodeIndex, by: isize) -> NodeIndex {
        if let Some(parent) = self.0.parent(from) {
            if self.0.child(parent, 0).map(|c| c == from).unwrap_or(false) {
                // This is the left child of the parent
                let child_to_parent_right_distance = self.child_distance_to_parent(from, 1);
                self.traverse_by_wrapping(parent, by - child_to_parent_right_distance as isize)
            } else if self.0.child(parent, 1).map(|c| c == from).unwrap_or(false) {
                // This is the right child of the parent
                let child_to_parent_left_distance = self.child_distance_to_parent(from, 0);
                self.traverse_by_wrapping(parent, by + child_to_parent_left_distance as isize)
            } else {
                panic!("Child element is not a child of the parent");
            }
        } else {
            // This is already the root node. Wrap around the entire thing.
            let direction = by / by.abs();
            self.traverse_by_wrapping(
                from,
                by - *self.0.subweight(from).unwrap() as isize * direction,
            )
        }
    }

    fn child_distance_to_parent(&self, child: NodeIndex, pos: u8) -> usize {
        if let Some(child_pos_child) = self.0.child(child, pos as usize) {
            *self.0.subweight(child_pos_child).unwrap() + 1
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traverse_forward_slightly() {
        let list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let keys: Vec<NodeIndex> = list.node_indexes().collect();
        let key3 = keys[3];
        assert_eq!(list[key3], 3);
        assert_eq!(list[list.traverse_by_wrapping(key3, -3)], 0);
        assert_eq!(list[list.traverse_by_wrapping(key3, -2)], 1);
        assert_eq!(list[list.traverse_by_wrapping(key3, -1)], 2);
        assert_eq!(list[list.traverse_by_wrapping(key3, 0)], 3);
        assert_eq!(list[list.traverse_by_wrapping(key3, 1)], 4);
        assert_eq!(list[list.traverse_by_wrapping(key3, 2)], 5);
        assert_eq!(list[list.traverse_by_wrapping(key3, 3)], 6);
        assert_eq!(list[list.traverse_by_wrapping(key3, 4)], 7);
    }

    #[test]
    fn test_traverse_forward_wrapping() {
        let list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let keys: Vec<NodeIndex> = list.node_indexes().collect();
        let key7 = keys[7];
        assert_eq!(list[key7], 7);
        assert_eq!(list[list.traverse_by_wrapping(key7, 9)], 6);
        assert_eq!(list[list.traverse_by_wrapping(key7, 10)], 7);
        assert_eq!(list[list.traverse_by_wrapping(key7, 11)], 8);
    }

    #[test]
    fn test_moves_value_nowhere() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let third_node = list.node_at_position(3).unwrap();
        list.move_by_wrapping(third_node, 0);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn test_moves_value_forward() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let fifth_node = list.node_at_position(5).unwrap();
        list.move_by_wrapping(fifth_node, 1);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 3, 4, 6, 5, 7, 8, 9]
        );
    }

    #[test]
    fn test_moves_value_backward_one() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let eighth_node = list.node_at_position(8).unwrap();
        list.move_by_wrapping(eighth_node, -1);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9]
        );
    }

    #[test]
    fn test_moves_value_backward() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let eighth_node = list.node_at_position(8).unwrap();
        list.move_by_wrapping(eighth_node, -6);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 8, 2, 3, 4, 5, 6, 7, 9]
        );
    }

    #[test]
    fn test_moves_value_forward_wrapping() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let fifth_node = list.node_at_position(5).unwrap();
        list.move_by_wrapping(fifth_node, 16);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 5, 3, 4, 6, 7, 8, 9]
        );
    }

    #[test]
    fn test_moves_value_backward_wrapping() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let seventh_node = list.node_at_position(7).unwrap();
        list.move_by_wrapping(seventh_node, -8);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 8, 7, 9]
        );
    }

    #[test]
    fn test_moves_value_wrap_around_front() {
        let mut list: ShuffleTree<u8> = (0..10).enumerate().collect();
        let seventh_node = list.node_at_position(7).unwrap();
        list.move_by_wrapping(seventh_node, -7);
        assert_eq!(
            list.ordered_values().copied().collect::<Vec<u8>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 8, 9, 7]
        );
    }
}
