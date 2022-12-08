use advent_of_code::helpers::parsing::{AocParsable, ParsingError, AocLineParsable, text_u8};
use nom::{FindSubstring, bytes::complete::tag, sequence::{tuple, separated_pair}, combinator::all_consuming, multi::many1, character::complete::line_ending};

#[derive(Clone, Debug)]
struct CargoItem(char);

fn parse_cargo_item_or_none(input: &[u8]) -> Option<CargoItem> {
    if input[0] == b'[' {
        Some(CargoItem(input[1] as char))
    } else if input[0] == b' ' {
        None
    } else {
        panic!("Invalid cargo item");
    }
}

#[derive(Clone, Debug)]
struct CargoStack(Vec<CargoItem>);

#[derive(Clone, Debug)]
struct CargoStage(Vec<CargoStack>);

#[inline]
fn input_index_for_row_col(row: u8, col: usize, col_count: usize) -> usize {
    row as usize * (col_count * 4) + (col as usize * 4)
}

#[inline]
fn input_slice_for_row_col(input: &[u8], row: u8, col: usize, col_count: usize) -> &[u8] {
    let start_index = input_index_for_row_col(row, col, col_count);
    &input[start_index..start_index + 4]
}

impl AocParsable for CargoStage {
    fn parse_from_string(input: &[u8]) -> Result<(&[u8], Self), ParsingError> {
        let first_newline = input
            .find_substring("\n")
            .expect("Input must have a newline") + 1;
        assert!(first_newline & 3 == 0, "Columns must divide by 4, was {}", first_newline);
        let column_count = first_newline / 4;

        let mut row_count = 0;
        loop {
            let row_start_index = input_index_for_row_col(row_count, 0, column_count);
            if input[row_start_index..row_start_index+2] == [b' ', b'1'] {
                break;
            }
            row_count += 1;
        }

        let mut columns: Vec<CargoStack> = vec![CargoStack(vec![]); column_count];
        for row in (0..row_count).rev() {
            for col in 0..column_count {
                let input_slice = input_slice_for_row_col(input, row, col, column_count);
                if let Some(item) = parse_cargo_item_or_none(input_slice) {
                    columns[col].0.push(item);
                }
            }
        }

        let cargo_stack = CargoStage(columns);
        Ok((
            &input[input_index_for_row_col(row_count + 1, 0, column_count)..],
            cargo_stack,
        ))
    }
}

#[derive(Debug)]
struct MoveOperation {
    cargo_count: u8,
    from_col: u8,
    to_col: u8
}

impl AocLineParsable for MoveOperation {
    fn parse_from_line(input: &[u8]) -> Result<(&[u8], Self), ParsingError> {
        let (rest, (_, cargo_count, _, from_col, _, to_col)) = tuple((
            tag("move "),
            text_u8(),
            tag(" from "),
            text_u8(),
            tag(" to "),
            text_u8()
        ))(input)?;
        Ok((rest, MoveOperation {cargo_count, from_col, to_col}))
    }
}

impl CargoStage {
    fn apply_move_9000(&mut self, move_op: &MoveOperation) {
        let from_cargo_stack = &mut self.0[move_op.from_col as usize - 1].0;
        let moved = from_cargo_stack.split_off(from_cargo_stack.len() - move_op.cargo_count as usize);
        self.0[move_op.to_col as usize - 1].0.extend(moved.into_iter().rev());
    }

    fn apply_move_9001(&mut self, move_op: &MoveOperation) {
        let from_cargo_stack = &mut self.0[move_op.from_col as usize - 1].0;
        let moved = from_cargo_stack.split_off(from_cargo_stack.len() - move_op.cargo_count as usize);
        self.0[move_op.to_col as usize - 1].0.extend(moved.into_iter());
    }

    fn read_top(&self) -> String {
        self.0.iter()
            .map(|stack| stack.0.last())
            .filter(Option::is_some)
            .map(Option::unwrap)
            .map(|item| item.0)
            .collect()
    }
}

pub fn part_one(input: &str) -> Option<String> {
    let (_, (mut stage, move_ops)) = all_consuming(separated_pair(
        CargoStage::parse_from_string,
        many1(line_ending),
        many1(MoveOperation::parse_from_string)
    ))(input.as_bytes()).expect("Invalid input");
    
    for op in move_ops {
        stage.apply_move_9000(&op);
    }

    Some(stage.read_top())
}

pub fn part_two(input: &str) -> Option<String> {
    let (_, (mut stage, move_ops)) = all_consuming(separated_pair(
        CargoStage::parse_from_string,
        many1(line_ending),
        many1(MoveOperation::parse_from_string)
    ))(input.as_bytes()).expect("Invalid input");
    
    for op in move_ops {
        stage.apply_move_9001(&op);
    }

    Some(stage.read_top())
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 5);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 5);
        assert_eq!(part_one(&input), None);
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 5);
        assert_eq!(part_two(&input), None);
    }
}
