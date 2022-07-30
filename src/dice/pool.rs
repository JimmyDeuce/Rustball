use super::{
    die::Die,
    dice_errors::RollError,
};
use std::{
    fmt,
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Pool {
    number: u8,
    sides: u8,
    dice: Vec<Die>,
}

impl Pool {
    pub fn new(number: u8, sides: u8) -> Self {
        let mut dice = Vec::<Die>::new();

        for _ in 0..number {
            let die = Die::roll(sides);
            dice.push(die);
        }

        Pool { number, sides, dice }
    }

    #[allow(dead_code)]
    pub fn dice(&self) -> &Vec<Die> {
        &self.dice
    }

    pub fn number(&self) -> u8 {
        self.number
    }

    pub fn sides(&self) -> u8 {
        self.sides
    }

    pub fn total(&self) -> u16 {
        // For now, this just returns the sum. In the future it will decide whether to sum, count successes, something else...
        self.sum_sides()
    }

    fn sum_sides(&self) -> u16 {
        self.dice.iter().fold(0, |sum, die| sum + die.result as u16)
    }

    pub fn keep_highest(&self, argument: u8) -> Self {
        let mut dice_sorted = self.dice.clone();
        dice_sorted.sort_unstable();

        let min_index = if argument > self.number { 0 } else { (self.number - argument) as usize };

        Pool { dice: dice_sorted[min_index..].to_vec(), ..*self }
    }

    pub fn keep_lowest(&self, argument: u8) -> Self {
        let mut dice_sorted = self.dice.clone();
        dice_sorted.sort_unstable();

        let max_index = if argument > self.number { self.number as usize } else { argument as usize };

        Pool { dice: dice_sorted[..max_index].to_vec(), ..*self }
    }

    pub fn reroll_all(&mut self) {
        for die in self.dice.iter_mut() {
            die.reroll();
        }
    }

    pub fn reroll_n(&mut self, n: u8) {
        for die in self.dice.iter_mut().filter(|d| d.equals(n)) {
            die.reroll();
        }
    }

    #[allow(dead_code)]
    fn reroll_n_or_less(&mut self, n: u8) {
        for die in self.dice.iter_mut().filter(|d| d.equal_or_less(n)) {
            die.reroll();
        }
    }

    #[allow(dead_code)]
    fn reroll_n_or_higher(&mut self, n: u8) {
        for die in self.dice.iter_mut().filter(|d| d.equal_or_greater(n)) {
            die.reroll();
        }
    }
}

impl fmt::Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut results = format!("{}", self.dice[0].result);
        for i in 1..self.dice.len() {
            results = format!("{}, {}", results, self.dice[i].result)
        }
        write!(f, "[{}]", results)
    }
}

impl FromStr for Pool {
    type Err = RollError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // TODO: Actually implement this
        Err(RollError::PlaceholderError)
    }
}
