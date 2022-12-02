use advent_of_code::helpers::parsing::{line_ending_or_eof, parse_all, AocParsable, generic_error_for_input};
use nom::{
    character::complete::{one_of, space1},
    sequence::{separated_pair, terminated},
};
use num_enum::TryFromPrimitive;

#[derive(PartialEq, Eq, Debug)]
enum Outcome {
    LOSS = 0,
    TIE = 3,
    WIN = 6,
}

impl Outcome {
    fn parse_from_char(char: &char) -> Option<Outcome> {
        match char {
            'X' => Some(Outcome::LOSS),
            'Y' => Some(Outcome::TIE),
            'Z' => Some(Outcome::WIN),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug, TryFromPrimitive)]
enum HandMove {
    ROCK = 1,
    PAPER = 2,
    SCISSORS = 3,
}

impl HandMove {
    fn is_victorious(&self, versus: &HandMove) -> Outcome {
        if self == versus {
            return Outcome::TIE;
        }
        let self_beats = (*self as u8 + 1) % 3 + 1;
        if *versus as u8 == self_beats {
            Outcome::WIN
        } else {
            Outcome::LOSS
        }
    }

    fn move_for_outcome(&self, outcome: &Outcome) -> HandMove {
        match &outcome {
            Outcome::TIE => self.clone(),
            Outcome::WIN => HandMove::try_from(*self as u8 % 3 + 1).unwrap(),
            Outcome::LOSS => HandMove::try_from((*self as u8 + 1) % 3 + 1).unwrap(),
        }
    }

    fn parse_from_char(char: &char) -> Option<HandMove> {
        match char {
            'A' | 'X' => Some(HandMove::ROCK),
            'B' | 'Y' => Some(HandMove::PAPER),
            'C' | 'Z' => Some(HandMove::SCISSORS),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct PartOneRound {
    opponent: HandMove,
    me: HandMove,
}

impl PartOneRound {
    fn my_score(&self) -> u8 {
        self.me.is_victorious(&self.opponent) as u8 + self.me as u8
    }
}

impl AocParsable for PartOneRound {
    fn parse_from_string<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let mut parser = terminated(
            separated_pair(one_of("ABC"), space1::<&'a [u8], ()>, one_of("XYZ")),
            line_ending_or_eof(),
        );
        if let Ok((rest, (opponent_char, me_char))) = parser(input) {
            let opponent_move = HandMove::parse_from_char(&opponent_char).unwrap();
            let me_move = HandMove::parse_from_char(&me_char).unwrap();
            let round = PartOneRound {
                opponent: opponent_move,
                me: me_move,
            };
            return Ok((rest, round));
        } else {
            generic_error_for_input(input)
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let rounds: Vec<PartOneRound> = parse_all(input.as_bytes()).unwrap();
    rounds
        .into_iter()
        .map(|round| round.my_score() as u32)
        .reduce(|a, b| a + b)
}

#[derive(Debug)]
struct PartTwoRound {
    opponent: HandMove,
    desired_outcome: Outcome,
}

impl PartTwoRound {
    fn my_score(&self) -> u8 {
        let my_move = self.opponent.move_for_outcome(&self.desired_outcome);
        my_move.is_victorious(&self.opponent) as u8 + my_move as u8
    }
}

impl AocParsable for PartTwoRound {
    fn parse_from_string<'a>(
        input: &'a [u8],
    ) -> Result<(&'a [u8], Self), nom::Err<nom::error::Error<&'a [u8]>>> {
        let mut parser = terminated(
            separated_pair(one_of("ABC"), space1::<&'a [u8], ()>, one_of("XYZ")),
            line_ending_or_eof(),
        );
        if let Ok((rest, (opponent_char, me_char))) = parser(input) {
            let opponent_move = HandMove::parse_from_char(&opponent_char).unwrap();
            let desired_outcome = Outcome::parse_from_char(&me_char).unwrap();
            let round = PartTwoRound {
                opponent: opponent_move,
                desired_outcome,
            };
            return Ok((rest, round));
        } else {
            generic_error_for_input(input)
        }
    }
}

pub fn part_two(input: &str) -> Option<u32> {
    let rounds: Vec<PartTwoRound> = parse_all(input.as_bytes()).unwrap();
    rounds
        .into_iter()
        .map(|round| round.my_score() as u32)
        .reduce(|a, b| a + b)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 2);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 2);
        assert_eq!(part_one(&input), None);
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 2);
        assert_eq!(part_two(&input), None);
    }
}
