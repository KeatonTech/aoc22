pub trait AocIteratorExtensions: Iterator {
    fn k_highest<const K: usize>(self) -> [Self::Item; K]
    where
        Self::Item: Ord,
        Self::Item: Default,
        Self::Item: Copy;
}

impl<I: Iterator> AocIteratorExtensions for I {
    fn k_highest<const K: usize>(self) -> [Self::Item; K]
    where
        Self::Item: Ord,
        Self::Item: Default,
        Self::Item: Copy,
    {
        self.fold([Default::default(); K], |mut acc, item| {
            let maybe_insert_at_index = match acc.binary_search(&item) {
                Ok(index) => {
                    if acc[0] == item {
                        None
                    } else {
                        Some(index)
                    }
                }
                Err(index) => {
                    if index == 0 {
                        None
                    } else {
                        Some(index - 1)
                    }
                }
            };
            if let Some(insert_at_index) = maybe_insert_at_index {
                acc[..insert_at_index + 1].rotate_left(1);
                acc[insert_at_index] = item;
            }
            acc
        })
    }
}
