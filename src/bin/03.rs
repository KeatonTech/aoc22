use advent_of_code::helpers::parsing::{
    iterate_all, line_ending_or_eof, AocLineParsable, AocParsable, generic_error_for_input,
};
use fixedbitset::FixedBitSet;
use nom::{
    multi::{fold_many1, many1},
    sequence::{terminated, tuple},
};

#[derive(Debug)]
struct Rucksack {
    left: FixedBitSet,
    right: FixedBitSet,
}

fn parse_rucksack_item<'a>(
    input: &'a [u8],
) -> Result<(&'a [u8], u8), nom::Err<nom::error::Error<&'a [u8]>>> {
    if input.len() == 0 {
        return generic_error_for_input(input);
    }

    let c = input[0] as char;
    if c >= 'a' && c <= 'z' {
        Ok((&input[1..], c as u8 - 'a' as u8 + 1))
    } else if c >= 'A' && c <= 'Z' {
        Ok((&input[1..], c as u8 - 'A' as u8 + 27))
    } else {
        generic_error_for_input(input)
    }
}

fn bit_set_from_items<'a>(rucksack_items: &'a [u8]) -> FixedBitSet {
    let mut bitset = FixedBitSet::with_capacity(53);
    for item in rucksack_items {
        bitset.insert((*item).into())
    }
    return bitset;
}

impl AocLineParsable for Rucksack {
    fn parse_from_line<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let (rest, items) = many1(parse_rucksack_item)(input)?;
        let (first_half, second_half) = items.split_at(items.len() >> 1);
        Ok((
            rest,
            Rucksack {
                left: bit_set_from_items(first_half),
                right: bit_set_from_items(second_half),
            },
        ))
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    Some(
        iterate_all(input.as_bytes())
            .map(|rucksack: Rucksack| {
                let mut intersection = rucksack.left.intersection(&rucksack.right);
                let item = intersection
                    .next()
                    .expect("Sides must have one intersection");
                assert_eq!(intersection.next(), None);
                item as u32
            })
            .sum(),
    )
}

#[derive(Debug)]
struct ElfPocket(FixedBitSet);

impl AocLineParsable for ElfPocket {
    fn parse_from_line<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let (rest, bitset) = fold_many1(
            parse_rucksack_item,
            || FixedBitSet::with_capacity(53),
            |mut set, item| {
                set.insert(item as usize);
                set
            },
        )(input)?;
        Ok((rest, ElfPocket(bitset)))
    }
}

#[derive(Debug)]
struct ElfGroup([ElfPocket; 3]);

impl AocParsable for ElfGroup {
    fn parse_from_string<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let (rest, (elf1, elf2, elf3)) = tuple((
            terminated(ElfPocket::parse_from_line, line_ending_or_eof()),
            terminated(ElfPocket::parse_from_line, line_ending_or_eof()),
            terminated(ElfPocket::parse_from_line, line_ending_or_eof()),
        ))(input)?;
        return Ok((rest, ElfGroup([elf1, elf2, elf3])));
    }
}

pub fn part_two(input: &str) -> Option<u32> {
    Some(
        iterate_all(input.as_bytes())
            .map(|elf_group: ElfGroup| {
                let mut intersection_set = elf_group.0[0].0.clone();
                intersection_set.intersect_with(&elf_group.0[1].0);
                intersection_set.intersect_with(&elf_group.0[2].0);
                let mut intersection_iter = intersection_set.ones();
                let priority = intersection_iter
                    .next()
                    .expect("Must have at least one intersection");
                assert_eq!(intersection_iter.next(), None);
                priority as u32
            })
            .sum(),
    )
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 3);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 3);
        assert_eq!(part_one(&input), Some(157));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 3);
        assert_eq!(part_two(&input), None);
    }
}
