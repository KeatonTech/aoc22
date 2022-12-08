use advent_of_code::helpers::parsing::{
    iterate_all, AocLineParsable, AocParsable, generic_error_for_input, ParsingError,
};
use nom::{
    sequence::tuple,
};

#[derive(Debug)]
struct RucksackBitSet(u64);

impl RucksackBitSet {
    fn new() -> Self {
        RucksackBitSet(0)
    }

    fn add(&mut self, value: u8) {
        self.0 |= 1 << value;
    }

    fn intersect(&mut self, other: Self) -> &mut Self {
        self.0 &= other.0;
        self
    }

    fn get_first(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }
}

fn parse_rucksack(
    input: &[u8],
) -> Result<(&[u8], RucksackBitSet), ParsingError> {
    if input.is_empty() {
        return generic_error_for_input(input);
    }

    let mut rucksack_bit_set = RucksackBitSet::new();
    let mut i = 0;
    while input[i] != b'\n' {
        let c = input[i];
        if c >= b'a' && c <= b'z' {
            rucksack_bit_set.add(c - b'a' + 1);
        } else if c >= b'A' && c <= b'Z' {
            rucksack_bit_set.add(c - b'A' + 27);
        } else {
            return generic_error_for_input(input)
        }
        i += 1;
    }

    return Ok((&input[i + 1..], rucksack_bit_set));
}

#[derive(Debug)]
struct ElfPocket(RucksackBitSet);

impl AocLineParsable for ElfPocket {
    fn parse_from_line(
        input: &[u8],
    ) -> Result<(&[u8], Self), ParsingError> {
        let (rest, bitset) = parse_rucksack(input)?;
        Ok((rest, ElfPocket(bitset)))
    }
}

#[derive(Debug)]
struct ElfGroup([ElfPocket; 3]);

impl AocParsable for ElfGroup {
    fn parse_from_string(
        input: &[u8],
    ) -> Result<(&[u8], Self), nom::Err<nom::error::Error<&[u8]>>> {
        let (rest, (elf1, elf2, elf3)) = tuple((
            ElfPocket::parse_from_line,
            ElfPocket::parse_from_line,
            ElfPocket::parse_from_line,
        ))(input)?;
        Ok((rest, ElfGroup([elf1, elf2, elf3])))
    }
}

pub fn part_two(input: &str) -> Option<u32> {
    Some(
        iterate_all(input.as_bytes())
            .map(|elf_group: ElfGroup| {
                let [mut elf_group_1, elf_group_2, elf_group_3] = elf_group.0;
                elf_group_1.0.intersect(elf_group_2.0);
                elf_group_1.0.intersect(elf_group_3.0);
                elf_group_1.0.get_first() as u32
            })
            .sum(),
    )
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 3);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 3);
        assert_eq!(part_two(&input), None);
    }
}
