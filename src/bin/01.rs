use advent_of_code::helpers::parsing::{parse_all, text_u32, AocParsable, line_ending_or_eof};
use nom::{character::complete::line_ending, multi::many1, sequence::terminated};

#[derive(Debug)]
struct ElfBaggage {
    items: Vec<u32>,
}

impl AocParsable for ElfBaggage {
    fn parse_from_string<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let parse_line = terminated(text_u32(), line_ending);
        let (rest, items) = terminated(many1(parse_line), line_ending_or_eof())(input)?;
        return Ok((rest, ElfBaggage { items }));
    }
}

fn elf_totals(elf_baggages: Vec<ElfBaggage>) -> impl Iterator<Item=u32> {
    elf_baggages
    .into_iter()
    .map(|eb| eb.items.into_iter().reduce(|a, b| a + b).unwrap())
}

pub fn part_one(input: &str) -> Option<u32> {
    let elf_baggages: Vec<ElfBaggage> = parse_all(input.as_bytes()).unwrap();
    Some(
        elf_totals(elf_baggages)
            .max()
            .unwrap()
    )
}

pub fn part_two(input: &str) -> Option<u32> {
    let elf_baggages: Vec<ElfBaggage> = parse_all(input.as_bytes()).unwrap();
    let top_3: [u32; 3] = elf_totals(elf_baggages)
            .fold([0u32; 3], |mut acc, elf_total| {
                if elf_total > acc[0] {
                    acc.rotate_right(1);
                    acc[0] = elf_total;
                } else if elf_total > acc[1] {
                    acc.swap(1, 2);
                    acc[1] = elf_total;
                } else if elf_total > acc[2] {
                    acc[2] = elf_total;
                }
                acc
            });
    return Some(top_3[0] + top_3[1] + top_3[2]);
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 1);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 1);
        assert_eq!(part_one(&input), None);
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 1);
        assert_eq!(part_two(&input), None);
    }
}
