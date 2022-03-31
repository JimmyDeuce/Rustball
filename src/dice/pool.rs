use super::die::Die;
use std::fmt;

#[derive(Debug)]
pub struct Pool {
    dice: Vec<Die>,
}

impl Pool {
    pub fn new(number: u8, sides: u8) -> Self {
        let mut dice = Vec::<Die>::new();

        for _ in 0..number {
            let die = Die::roll(sides);
            dice.push(die);
        }

        Pool { dice }
    }

    fn sum_sides(&self) -> u16 {
        let mut total = 0;
        for die in &self.dice {
            total += die.result as u16;
        }
        total
    }
}

impl fmt::Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut results = format!("{}", self.dice[0].result);
        for i in 1..self.dice.len() {
            results = format!("{}, {}", results, self.dice[i].result)
        }
        write!(f, "**{}**: [{}]", self.sum_sides(), results)
    }
}