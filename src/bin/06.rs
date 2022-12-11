use std::collections::VecDeque;

use bit_set::BitSet;

struct StartMarkerIndexIterator<'a, const MARKER_SIZE: u8> {
    ringbuffer: VecDeque<u8>,
    index: usize,
    input: &'a [u8],
}

impl<'a, const MARKER_SIZE: u8> StartMarkerIndexIterator<'a, MARKER_SIZE> {
    fn iterate(input: &'a [u8]) -> Self {
        assert!(
            input.len() >= MARKER_SIZE as usize,
            "Input is too short to have a Start Of Packet marker"
        );
        let ringbuffer = VecDeque::with_capacity(MARKER_SIZE as usize);
        StartMarkerIndexIterator {
            ringbuffer,
            index: 0,
            input,
        }
    }

    fn advance_by(&mut self, by: u8) {
        assert!(by <= MARKER_SIZE);
        assert!(self.index + (by as usize) < self.input.len());

        // Remove items from the ring as necessary
        let removal_amount = if self.ringbuffer.len() + by as usize > MARKER_SIZE as usize {
            self.ringbuffer.len() + by as usize - MARKER_SIZE as usize
        } else {
            0
        };
        self.ringbuffer.rotate_left(removal_amount);
        self.ringbuffer
            .resize(self.ringbuffer.len() - removal_amount, 0);

        // Add new items to the ring
        for _i in 0..by {
            self.index += 1;
            self.ringbuffer.push_back(self.input[self.index])
        }
    }

    // Finds the last character pair in the ring buffer. Returns the index of the first character
    // in the pair. Returns None if every character is unique.
    fn find_last_duplicate_char(&self) -> Option<usize> {
        let mut bit_set = BitSet::with_capacity(256);
        if self.ringbuffer.len() < 2 {
            return None;
        }
        (0..self.ringbuffer.len())
            .rev()
            .find(|&i| !bit_set.insert(self.ringbuffer[i].into()))
    }
}

impl<'a, const MARKER_SIZE: u8> Iterator for StartMarkerIndexIterator<'a, MARKER_SIZE> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.input.len() {
            if let Some(last_duplicate_index) = self.find_last_duplicate_char() {
                self.advance_by(last_duplicate_index as u8 + 1);
            } else if self.ringbuffer.len() == MARKER_SIZE as usize {
                return Some(self.index as u32 + 1);
            } else {
                self.advance_by(1);
            }
        }
        None
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    StartMarkerIndexIterator::<4>::iterate(input.as_bytes()).next()
}

pub fn part_two(input: &str) -> Option<u32> {
    StartMarkerIndexIterator::<14>::iterate(input.as_bytes()).next()
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 6);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}
