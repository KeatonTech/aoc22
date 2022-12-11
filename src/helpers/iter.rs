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
                    if acc[K - 1] == item {
                        None
                    } else {
                        Some(index)
                    }
                }
                Err(index) => {
                    if index == K {
                        None
                    } else {
                        Some(index)
                    }
                }
            };
            if let Some(insert_at_index) = maybe_insert_at_index {
                acc[insert_at_index..].rotate_right(1);
                acc[insert_at_index] = item;
            }
            acc
        })
    }
}
