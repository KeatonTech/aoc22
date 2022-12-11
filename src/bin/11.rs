use num::integer;
use std::num::NonZeroU8;
use std::{cell::RefCell, mem::swap};

use advent_of_code::helpers::iter::AocIteratorExtensions;
use advent_of_code::helpers::parsing::{
    iterate_all, line_ending_or_eof, text_u8, text_usize, AocLineParsable, ParsingResult,
};
use nom::{
    bytes::streaming::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    Parser,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Old,
    Primitive(u8),
}

impl Value {
    fn parse<'a>(input: &'a [u8]) -> ParsingResult<Value> {
        nom::branch::alt((
            tag("old").map(|_| Value::Old),
            text_u8().map(|num| Value::Primitive(num)),
        ))(input)
    }

    fn resolve(&self, old_value: usize) -> usize {
        match self {
            Value::Old => old_value,
            Value::Primitive(val) => *val as usize,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Operation {
    Plus((Value, Value)),
    Times((Value, Value)),
}

impl Operation {
    fn parse<'a>(input: &'a [u8]) -> ParsingResult<Operation> {
        nom::branch::alt((
            separated_pair(Value::parse, tag(" + "), Value::parse)
                .map(|(first, second)| Operation::Plus((first, second))),
            separated_pair(Value::parse, tag(" * "), Value::parse)
                .map(|(first, second)| Operation::Times((first, second))),
        ))(input)
    }

    fn evaluate(&self, old_value: usize) -> usize {
        match self {
            Operation::Plus((v1, v2)) => {
                v1.resolve(old_value) as usize + v2.resolve(old_value) as usize
            }
            Operation::Times((v1, v2)) => {
                v1.resolve(old_value) as usize * v2.resolve(old_value) as usize
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Monkey {
    items: Vec<usize>,
    operation: Operation,
    test_divisibility: u8,
    if_true: u8,
    if_false: u8,
    inspection_count: u32,
}

impl AocLineParsable for Monkey {
    fn parse_from_line<'a>(input: &'a [u8]) -> ParsingResult<Monkey> {
        map(
            tuple((
                tag("Monkey "),
                text_u8(),
                tag(":\n  Starting items: "),
                separated_list1(tag(", "), text_usize()),
                tag("\n  Operation: new = "),
                Operation::parse,
                tag("\n  Test: divisible by "),
                text_u8(),
                tag("\n    If true: throw to monkey "),
                text_u8(),
                tag("\n    If false: throw to monkey "),
                text_u8(),
                line_ending_or_eof(),
            )),
            |(_, _, _, starting_items, _, operation, _, test, _, t, _, f, _)| Monkey {
                items: starting_items,
                operation,
                test_divisibility: test,
                if_true: t,
                if_false: f,
                inspection_count: 0,
            },
        )(input)
    }
}

#[derive(Debug)]
struct MonkeyGaggle {
    monkeys: Vec<RefCell<Monkey>>,
    lcm: u32,
}

impl MonkeyGaggle {
    fn new<I: Iterator<Item = Monkey>>(monkey_iter: I) -> MonkeyGaggle {
        let monkeys: Vec<RefCell<Monkey>> = monkey_iter.map(RefCell::new).collect();
        let lcm = monkeys
            .iter()
            .map(|m| m.borrow().test_divisibility as u32)
            .reduce(integer::lcm)
            .unwrap();
        MonkeyGaggle { monkeys, lcm }
    }

    fn simulate_round(&self, worry_reduction_factor: NonZeroU8) {
        let items_before: usize = self.monkeys.iter().map(|m| m.borrow().items.len()).sum();
        for i in 0..self.monkeys.len() {
            self.take_turn(i, worry_reduction_factor)
        }
        let items_after: usize = self.monkeys.iter().map(|m| m.borrow().items.len()).sum();
        assert_eq!(items_before, items_after);
    }

    fn take_turn(&self, monkey_id: usize, worry_reduction_factor: NonZeroU8) {
        let mut me = self.monkeys[monkey_id].borrow_mut();
        me.inspection_count += me.items.len() as u32;
        let mut items: Vec<usize> = vec![];
        swap(&mut me.items, &mut items);
        for item in items {
            let new_item_value = (me.operation.evaluate(item)
                / worry_reduction_factor.get() as usize)
                % self.lcm as usize;
            let toss_to = if new_item_value % me.test_divisibility as usize == 0 {
                me.if_true
            } else {
                me.if_false
            };
            self.monkeys[toss_to as usize]
                .borrow_mut()
                .items
                .push(new_item_value);
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let gaggle = MonkeyGaggle::new(&mut iterate_all(input.as_bytes()));
    for _i in 0..20 {
        gaggle.simulate_round(NonZeroU8::new(3).unwrap());
    }
    let highest2 = gaggle
        .monkeys
        .iter()
        .map(|m| m.borrow().inspection_count)
        .k_highest::<2>();
    Some(highest2[0] * highest2[1])
}

pub fn part_two(input: &str) -> Option<usize> {
    let gaggle = MonkeyGaggle::new(&mut iterate_all(input.as_bytes()));
    for _i in 0..10000 {
        gaggle.simulate_round(NonZeroU8::new(1).unwrap());
    }
    let highest2 = gaggle
        .monkeys
        .iter()
        .map(|m| m.borrow().inspection_count as usize)
        .k_highest::<2>();
    Some(highest2[0] * highest2[1])
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 11);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_parse() {
        assert_eq!(
            Operation::parse("old * 11".as_bytes()).unwrap().1,
            Operation::Times((Value::Old, Value::Primitive(11)))
        );
    }
}
