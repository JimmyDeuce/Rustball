use regex::Regex;
use std::collections::VecDeque;
use std::str::FromStr;
use super::dice_errors::RollError;
use super::roll::Roll;
use crate::math::calculator;

const DICE_MATCH_STRING: &str = r"(?P<number>\d+)d(?P<sides>\d+)";
const DICE_SPLIT_STRING: &str = r"d";
const CAPACITY: usize = 1;

pub struct Tray {
    dice_match_re: Regex,
    _dice_split_re: Regex,
    rolls: VecDeque<Roll>,
}

impl Tray {
    pub fn new() -> Self {
        Tray {
            dice_match_re: Regex::new(DICE_MATCH_STRING).expect("Failed to compile dice matching regex!"),
            _dice_split_re: Regex::new(DICE_SPLIT_STRING).expect("Failed to compile dice splitting regex!"),
            rolls: VecDeque::new()
        }
    }

    // Take a roll command and return the fully formatted result string (or an error)
    pub fn process_roll_command(&mut self, roll_command: &str) -> Result<String, RollError> {
        // Check if there is a dice expression in the command
        if !self.dice_match_re.is_match(roll_command) {
            // If no dice, treat it as a mathematical expression and toss it to the calculator
            let calc_result = match calculator::evaluate(roll_command) {
                Ok(res) => res,
                Err(why) => format!("☢ I don't know how to calculate that! ☢ {}", why)
            };
            return Ok(calc_result);
        }

        let roll_result;
        match self.add_roll_fom_command(roll_command) {
            Ok(res) => roll_result = res,
            Err(why) => return Err(why)
        };

        Ok(roll_result)
    }

    pub fn get_newest_roll(&self) -> Result<&Roll, RollError> {
        let get_roll_result = match self.rolls.back() {
            Some(roll) => Ok(roll),
            None => Err(RollError::RetrieveError("Error retrieving latest roll from tray: Roll queue is empty".to_owned()))
        };

        get_roll_result
    }

    // Take the command, turn it into a roll, add that to the tray, and return the infix expression that should be passed to the calculator
    fn add_roll_fom_command(&mut self, roll_command: &str) -> Result<String, RollError> {
         // If Rolls queue is full, remove the oldest element
        while self.rolls.len() >= CAPACITY { self.rolls.pop_front(); }

        // Command to be passed to math
        let mut math_command = roll_command.to_owned();

        // Make a new empty roll
        let mut new_roll = Roll::new(roll_command.to_owned());

        // For each capture in the roll command, add a dicepool constructed from that capture to the roll
        for captures in self.dice_match_re.captures_iter(roll_command) {
            let number = &captures["number"].parse::<u8>()?;
            let sides = &captures["sides"].parse::<u8>()?;
            let pool_total = new_roll.add_pool(*number, *sides);

            math_command = self.dice_match_re.replace(&math_command, pool_total.to_string()).to_string();
        }

        // Add new roll to tray
        self.rolls.push_back(new_roll);

        Ok(format!("{}", math_command))
    }

    pub fn add_roll_from_string(&mut self, roll_command: &str) -> Result<(), RollError> {
        while self.rolls.len() >= CAPACITY { self.rolls.pop_front(); } // If Rolls queue is full, remove the oldest element

        let roll_result = Roll::from_str(roll_command);

        let add_to_tray_result = match roll_result {
            Ok(roll) => {
                self.rolls.push_back(roll);
                Ok(())
            },
            Err(why) => Err(why)
        };

        add_to_tray_result
    }
}