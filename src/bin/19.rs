#![feature(portable_simd)]
use advent_of_code::helpers::parsing::{
    iterate_all, text_u8, AocLineParsable, AocParsable, ParsingResult,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{separated_pair, terminated, tuple},
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    ops::{Add, Index, IndexMut},
    simd::{i16x4, SimdInt},
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};

#[derive(Clone, Copy, Debug, PartialEq, FromRepr, EnumIter)]
enum Material {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}

#[derive(Clone, Copy, Default, Debug)]
struct MaterialMap<T: Clone + Copy + Default + Sized>([T; 4]);

impl<T: Clone + Copy + Default + Sized> Index<Material> for MaterialMap<T> {
    type Output = T;

    fn index(&self, index: Material) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl<T: Clone + Copy + Default + Sized> IndexMut<Material> for MaterialMap<T> {
    fn index_mut(&mut self, index: Material) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

// Parsing

#[derive(Clone, Copy, Default, Debug)]
struct MaterialCost(i16x4);

impl Add for MaterialCost {
    type Output = MaterialCost;

    fn add(self, rhs: Self) -> Self::Output {
        MaterialCost(self.0 + rhs.0)
    }
}

impl Index<Material> for MaterialCost {
    type Output = i16;

    fn index(&self, index: Material) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl MaterialCost {
    fn parse_single_cost(input: &[u8]) -> ParsingResult<MaterialCost> {
        alt((
            map(terminated(text_u8(), tag(" ore")), |c| {
                MaterialCost(i16x4::from_array([c as i16, 0, 0, 0]))
            }),
            map(terminated(text_u8(), tag(" clay")), |c| {
                MaterialCost(i16x4::from_array([0, c as i16, 0, 0]))
            }),
            map(terminated(text_u8(), tag(" obsidian")), |c| {
                MaterialCost(i16x4::from_array([0, 0, c as i16, 0]))
            }),
        ))(input)
    }
}

impl AocParsable for MaterialCost {
    fn parse_from_string(input: &[u8]) -> ParsingResult<MaterialCost> {
        alt((
            map(
                separated_pair(
                    MaterialCost::parse_single_cost,
                    tag(" and "),
                    MaterialCost::parse_single_cost,
                ),
                |(a, b)| a + b,
            ),
            MaterialCost::parse_single_cost,
        ))(input)
    }
}

#[derive(Debug)]
struct BlueprintCosts {
    blueprint_number: u8,
    robot_costs: MaterialMap<MaterialCost>,
    max_costs_per_material: MaterialCost,
}

impl AocLineParsable for BlueprintCosts {
    fn parse_from_line(
        input: &[u8],
    ) -> Result<(&[u8], Self), advent_of_code::helpers::parsing::ParsingError> {
        map(
            tuple((
                tag("Blueprint "),
                text_u8(),
                tag(": Each ore robot costs "),
                MaterialCost::parse_from_string,
                tag(". Each clay robot costs "),
                MaterialCost::parse_from_string,
                tag(". Each obsidian robot costs "),
                MaterialCost::parse_from_string,
                tag(". Each geode robot costs "),
                MaterialCost::parse_from_string,
                tag("."),
            )),
            |(
                _,
                blueprint_number,
                _,
                ore_robot,
                _,
                clay_robot,
                _,
                obsidian_robot,
                _,
                geode_robot,
                _,
            )| {
                let raw_robot_costs = [ore_robot, clay_robot, obsidian_robot, geode_robot];
                let max_costs_per_material = MaterialCost(i16x4::from_array([
                    (0..4)
                        .map(|robot_index| raw_robot_costs[robot_index].0[Material::Ore as usize])
                        .max()
                        .unwrap(),
                    (0..4)
                        .map(|robot_index| raw_robot_costs[robot_index].0[Material::Clay as usize])
                        .max()
                        .unwrap(),
                    (0..4)
                        .map(|robot_index| {
                            raw_robot_costs[robot_index].0[Material::Obsidian as usize]
                        })
                        .max()
                        .unwrap(),
                    0i16,
                ]));
                BlueprintCosts {
                    blueprint_number,
                    robot_costs: MaterialMap(raw_robot_costs),
                    max_costs_per_material,
                }
            },
        )(input)
    }
}

// Simulating

#[derive(Clone, Copy, Debug, Default)]
struct Inventory {
    materials: i16x4,
    robots: MaterialMap<i16>,
}

impl Inventory {
    fn new(initial_robots: [i16; 4]) -> Self {
        Inventory {
            materials: Default::default(),
            robots: MaterialMap(initial_robots),
        }
    }

    fn add_robot(&mut self, material: Material, blueprint_costs: &BlueprintCosts) {
        self.robots[material] += 1;
        self.materials -= blueprint_costs.robot_costs[material].0;
    }

    /// Simulates each robot taking one 'turn', producing one
    /// material of the given type for the inventory.
    fn produce_materials(&mut self) {
        self.materials += i16x4::from_array(self.robots.0);
    }

    /// Returns true if there are enough materials to produce
    /// a given robot type with the given blueprint
    fn can_afford_robot(&self, for_material: Material, blueprint_costs: &BlueprintCosts) -> bool {
        let material_cost = blueprint_costs.robot_costs[for_material].0;
        let subtracted = self.materials - material_cost;
        subtracted.reduce_min() >= 0
    }
}

/// Returns the highest
fn robot_choices_dfs(max_steps: u8, blueprint_costs: &BlueprintCosts) -> usize {
    let inventory: Inventory = Inventory::new([1, 0, 0, 0]);
    robot_choices_dfs_recursive(max_steps, inventory, blueprint_costs, None)
}

fn robot_choices_dfs_recursive(
    remaining_steps: u8,
    inventory: Inventory,
    blueprint_costs: &BlueprintCosts,
    maybe_next_robot: Option<Material>,
) -> usize {
    if remaining_steps == 0 {
        return inventory.materials[Material::Geode as usize] as usize;
    }
    let mut next_inventory = inventory;
    next_inventory.produce_materials();
    let next_remaining_steps = remaining_steps - 1;

    // If we're working towards affording a robot, recurse until we can afford it.
    if let Some(next_robot) = maybe_next_robot {
        if inventory.can_afford_robot(next_robot, blueprint_costs) {
            next_inventory.add_robot(next_robot, blueprint_costs);
        } else {
            return robot_choices_dfs_recursive(
                next_remaining_steps,
                next_inventory,
                blueprint_costs,
                maybe_next_robot,
            );
        }
    }

    Material::iter()
        .filter(|m| {
            is_potential_next_robot(*m, remaining_steps as i16, next_inventory, blueprint_costs)
        })
        .map(|m| {
            robot_choices_dfs_recursive(
                next_remaining_steps,
                next_inventory,
                blueprint_costs,
                Some(m),
            )
        })
        .max()
        .unwrap_or_default()
}

#[inline]
fn is_potential_next_robot(
    robot_material: Material,
    remaining_steps: i16,
    inventory: Inventory,
    blueprint_costs: &BlueprintCosts,
) -> bool {
    // Ensure that creating this robot might potentially be useful.
    robot_material == Material::Geode
        || inventory.robots[robot_material] * remaining_steps
            + inventory.materials[robot_material as usize]
            < blueprint_costs.max_costs_per_material[robot_material] * remaining_steps
}

pub fn part_one(input: &str) -> Option<usize> {
    let blueprint_options: Vec<BlueprintCosts> = iterate_all(input.as_bytes()).collect();
    Some(
        blueprint_options
            .par_iter()
            .map(|blueprint| blueprint.blueprint_number as usize * robot_choices_dfs(24, blueprint))
            .sum(),
    )
}

pub fn part_two(input: &str) -> Option<usize> {
    let blueprint_options: Vec<BlueprintCosts> = iterate_all(input.as_bytes()).collect();
    blueprint_options
        .par_iter()
        .take(3)
        .map(|blueprint| robot_choices_dfs(32, blueprint))
        .reduce_with(|a, b| a * b)
}

pub fn main() {
    let input = &advent_of_code::read_file("inputs", 19);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 19);
        assert_eq!(part_one(&input), Some(33));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 19);
        assert_eq!(part_two(&input), Some(3472));
    }
}
