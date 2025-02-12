use std::{str::FromStr, fmt};
use super::{
    dice_errors::RollError,
    pool::Pool,
    roll_token::RollToken,
    roll_value::RollValue,
    value_kinds::*,
    genesymbols::GeneSymbol,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    Single(u8),
    Array(Vec<u8>),
}

impl FromStr for Argument {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(array_string) = s.trim().strip_prefix('[').unwrap_or("").strip_suffix(']') {
            let mut args_array = Vec::<u8>::new();
            for number_str in array_string.split_terminator(',') {
                args_array.push(number_str.trim().parse()?);
            }
            Ok(Argument::Array(args_array))
        } else {
            Ok(Argument::Single(s.parse()?))
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argument::Array(array) => write!(f, "{:?}", array),
            Argument::Single(single) => write!(f, "{}", single),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dice{pub pool: Option<Pool>}

impl Dice {
    pub fn apply(&self, left: Argument, right: Argument) -> Result<Self, RollError> {
        let pool = match left {
            Argument::Single(number) => match right {
                Argument::Single(sides) => Some(Pool::new(number, sides)),
                Argument::Array(sides) => Some(Pool::new_dice_array(number, &sides)),
            },
            Argument::Array(number) => match right {
                Argument::Single(sides) => Some(Pool::new_numbers_array(&number, sides)),
                Argument::Array(sides) => Some(Pool::new_from_arrays(&number, &sides)),
            },
        };

        Ok(Dice{ pool })
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        self.pool.ok_or(RollError::MissingPoolError)
    }

    pub fn value(self) -> Result<RollValue, RollError> {
        Ok(self.pool()?.total().into())
    }

    pub fn description(&self) -> String {
        let pool = self.pool.as_ref().expect("Tried to print a dice operation that wasn't resolved yet!");

        let (numbers, sides) = (pool.numbers(), pool.sides());

        match numbers.len() {
            1 => match sides.len() {
                1 => format!("Rolled {}d{}", numbers[0], sides[0]),
                _ => format!("Rolled {}d{:?}", numbers[0], sides)
            },
            _ => match sides.len() {
                1 => format!("Rolled {:?}d{}", numbers, sides[0]),
                _ => format!("Rolled {:?}d{:?}", numbers, sides)
            }
        }
    }

    pub fn verbose(&self) -> String {
        format!("{}", self)
    }
}

impl FromStr for Dice {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "d" {                               // If just the dice operator, return an uninitialized pool
            Ok(Dice{ pool: None })
        } else if let Ok(pool) = s.parse() {  // If it can be parsed into a pool, return that pool
            Ok(Dice{ pool: Some(pool) })
        } else {                                    // Otherwise error
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pool = self.pool.as_ref().expect("Tried to print a dice operation that wasn't resolved yet!");

        let (numbers, sides) = (pool.numbers(), pool.sides());

        let dice_code = match numbers.len() {
            1 => match sides.len() {
                1 => format!("{}d{}", numbers[0], sides[0]),
                _ => format!("{}d{:?}", numbers[0], sides)
            },
            _ => match sides.len() {
                1 => format!("{:?}d{}", numbers, sides[0]),
                _ => format!("{:?}d{:?}", numbers, sides)
            }
        };

        write!(f, "{} -> {}", dice_code, pool)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Combination {
    Merge(Merge),
}

impl Combination {
    pub fn apply(&self, left: RollToken, right: RollToken) -> Result<Self, RollError> {
        match self {
            Combination::Merge(merge) => Ok(Combination::Merge(merge.apply(left, right)?)),
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Combination::Merge(merge) => merge.pool(),
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            Combination::Merge(merge) => merge.value(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Combination::Merge(merge) => merge.description(),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Combination::Merge(merge) => merge.verbose(),
        }
    }
}

impl FromStr for Combination {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(merge) = s.parse() {
            Ok(Combination::Merge(merge))
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Combination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combination::Merge(merge) => write!(f, "{}", merge),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Merge {
    pub left: Option<Box<RollToken>>,
    pub right: Option<Box<RollToken>>,
}

impl Merge {
    pub fn apply(&self, left: RollToken, right: RollToken) -> Result<Self, RollError> {
        Ok(Merge { left: Some(Box::new(left)), right: Some(Box::new(right)) })
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        let left_pool = self.left.ok_or(RollError::NotResolvedError)?.pool()?;
        let right_pool = self.right.ok_or(RollError::NotResolvedError)?.pool()?;
        Ok(left_pool.add(&right_pool))
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        let left_value = self.left.as_ref().ok_or(RollError::NotResolvedError)?.value()?;
        let right_value = self.right.as_ref().ok_or(RollError::NotResolvedError)?.value()?;
        left_value.add(right_value)
    }

    pub fn description(&self) -> String {
        String::from("Oops, you're not supposed to see this! Please let the boss know! (Tried to get description for merge operator)")
    }

    pub fn verbose(&self) -> String {
        String::from("Oops, you're not supposed to see this! Please let the boss know! (Tried to get verbose string for merge operator)")
    }
}

impl FromStr for Merge {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() == "&" {
            Ok(Merge { left: None, right: None })
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Merge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not implemented yet")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Conversion {
    Genesys(GenesysDice),
}

impl Conversion {
    pub fn apply(&self, token: RollToken) -> Result<Self, RollError> {
        match self {
            Conversion::Genesys(g_dice) => Ok(Conversion::Genesys(g_dice.apply(token.pool()?)?)),
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Conversion::Genesys(g_dice) => g_dice.pool()
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            Conversion::Genesys(g_dice) => g_dice.value(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Conversion::Genesys(g_dice) => g_dice.description(),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Conversion::Genesys(g_dice) => g_dice.verbose(),
        }
    }
}

impl FromStr for Conversion {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(g_dice) = s.parse() {
            Ok(Conversion::Genesys(g_dice))
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Conversion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conversion::Genesys(g_dice) => write!(f, "{}", g_dice),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GenesysDice {
    Boost{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
    Setback{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
    Ability{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
    Difficulty{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
    Proficiency{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
    Challenge{base: Option<Pool>, res: Vec<Vec<GeneSymbol>>},
}

impl GenesysDice {
    pub fn apply(&self, pool: Pool) -> Result<Self, RollError> {
        let mut res = vec![];

        match self {
            GenesysDice::Boost { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::boost(*die));
                }
                Ok(GenesysDice::Boost { base: Some(pool), res })
            },
            GenesysDice::Setback { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::setback(*die));
                }
                Ok(GenesysDice::Setback { base: Some(pool), res })
            },
            GenesysDice::Ability { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::ability(*die));
                }
                Ok(GenesysDice::Ability { base: Some(pool), res })
            },
            GenesysDice::Difficulty { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::difficulty(*die));
                }
                Ok(GenesysDice::Difficulty { base: Some(pool), res })
            },
            GenesysDice::Proficiency { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::proficiency(*die));
                }
                Ok(GenesysDice::Proficiency { base: Some(pool), res })
            },
            GenesysDice::Challenge { base: _, res: _ } => {
                for die in pool.dice() {
                    res.push(GeneSymbol::challenge(*die));
                }
                Ok(GenesysDice::Challenge { base: Some(pool), res })
            },
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            GenesysDice::Boost { base, res: _ } => base.ok_or(RollError::NotResolvedError),
            GenesysDice::Setback { base, res: _ } => base.ok_or(RollError::NotResolvedError),
            GenesysDice::Ability { base, res: _ } => base.ok_or(RollError::NotResolvedError),
            GenesysDice::Difficulty { base, res: _ } => base.ok_or(RollError::NotResolvedError),
            GenesysDice::Proficiency { base, res: _ } => base.ok_or(RollError::NotResolvedError),
            GenesysDice::Challenge { base, res: _ } => base.ok_or(RollError::NotResolvedError),
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            GenesysDice::Boost { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
            GenesysDice::Setback { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
            GenesysDice::Ability { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
            GenesysDice::Difficulty { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
            GenesysDice::Proficiency { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
            GenesysDice::Challenge { base: _, res } => Ok(RollValue::Genesys(GenesysValue::new(res))),
        }
    }

    pub fn description(&self) -> String {
        match self {
            GenesysDice::Boost { base: _, res: _ } => String::from("Convert numeric results to boost die values"),
            GenesysDice::Setback { base: _, res: _ } => String::from("Convert numeric results to setback die values"),
            GenesysDice::Ability { base: _, res: _ } => String::from("Convert numeric results to ability die values"),
            GenesysDice::Difficulty { base: _, res: _ } => String::from("Convert numeric results to difficulty die values"),
            GenesysDice::Proficiency { base: _, res: _ } => String::from("Convert numeric results to proficiency die values"),
            GenesysDice::Challenge { base: _, res: _ } => String::from("Convert numeric results to challenge die values"),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            GenesysDice::Boost { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
            GenesysDice::Setback { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
            GenesysDice::Ability { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
            GenesysDice::Difficulty { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
            GenesysDice::Proficiency { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
            GenesysDice::Challenge { base, res } => format!("{} -> {:?}", base.as_ref().unwrap_or(&Pool::new(0, 0)), res),
        }
    }
}

impl FromStr for GenesysDice {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(kind) = s.trim().strip_prefix('g') {
            match kind {
                "b" => Ok(GenesysDice::Boost { base: None, res: vec![] }),
                "s" => Ok(GenesysDice::Setback { base: None, res: vec![] }),
                "a" => Ok(GenesysDice::Ability { base: None, res: vec![] }),
                "d" => Ok(GenesysDice::Difficulty { base: None, res: vec![] }),
                "p" => Ok(GenesysDice::Proficiency { base: None, res: vec![] }),
                "c" => Ok(GenesysDice::Challenge { base: None, res: vec![] }),
                _ => Err(RollError::SymbolError(s.into())),
            }
        } else {
                Err(RollError::SymbolError(s.into()))
            }
    }
}

impl fmt::Display for GenesysDice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenesysDice::Boost { base: _, res } => write!(f, "Boost: {:?}", res),
            GenesysDice::Setback { base: _, res } => write!(f, "Setback: {:?}", res),
            GenesysDice::Ability { base: _, res } => write!(f, "Ability: {:?}", res),
            GenesysDice::Difficulty { base: _, res } => write!(f, "Difficulty: {:?}", res),
            GenesysDice::Proficiency { base: _, res } => write!(f, "Proficiency: {:?}", res),
            GenesysDice::Challenge { base: _, res } => write!(f, "Challenge: {:?}", res),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Cap(Cap),
    Explode(Explode),
    Keep(Keep),
    Reroll(Reroll),
    Target(Target),
}

impl Operator {
    pub fn apply(&self, token: RollToken, argument: Argument) -> Result<Self, RollError> {
        match self {
            Operator::Cap(cap) => Ok(Operator::Cap(cap.apply(token, argument)?)),
            Operator::Explode(explode) => Ok(Operator::Explode(explode.apply(token.pool()?, argument)?)),
            Operator::Keep(keep) => Ok(Operator::Keep(keep.apply(token.pool()?, argument)?)),
            Operator::Reroll(reroll) => Ok(Operator::Reroll(reroll.apply(token.pool()?, argument)?)),
            Operator::Target(target) => Ok(Operator::Target(target.apply(token, argument)?)),
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Operator::Cap(_) => todo!(),
            Operator::Explode(explode) => explode.pool(),
            Operator::Keep(keep) => keep.pool(),
            Operator::Reroll(reroll) => reroll.pool(),
            Operator::Target(target) => target.pool(),
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            Operator::Cap(_) => todo!(),
            Operator::Explode(explode) => explode.value(),
            Operator::Keep(keep) => keep.value(),
            Operator::Reroll(reroll) => reroll.value(),
            Operator::Target(target) => Ok(target.value()),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Operator::Cap(_) => todo!(),
            Operator::Explode(explode) => explode.description(),
            Operator::Keep(keep) => keep.description(),
            Operator::Reroll(reroll) => reroll.description(),
            Operator::Target(target) => target.description(),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Operator::Cap(_) => todo!(),
            Operator::Explode(explode) => explode.verbose(),
            Operator::Keep(keep) => keep.verbose(),
            Operator::Reroll(reroll) => reroll.verbose(),
            Operator::Target(target) => target.verbose(),
        }
    }
}

impl FromStr for Operator {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(cap) = s.parse() {          // Attempt to parse into cap token
            Ok(Operator::Cap(cap))
        } else if let Ok(explode) = s.parse() {          // Attempt to parse into explode token
            Ok(Operator::Explode(explode))
        } else if let Ok(keep) = s.parse() {                // Attempt to parse into keep token
            Ok(Operator::Keep(keep))
        } else if let Ok(reroll) = s.parse() {            // Attempt to parse into reroll token
            Ok(Operator::Reroll(reroll))
        } else if let Ok(target) = s.parse() {
            Ok(Operator::Target(target))
        } else {                                                  // If all these fail, error out
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Cap(_) => todo!(),
            Operator::Explode(explode) => write!(f, "{}", explode),
            Operator::Keep(keep) => write!(f, "{}", keep),
            Operator::Reroll(reroll) => write!(f, "{}", reroll),
            Operator::Target(target) => write!(f, "{}", target),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cap {
    Max{arg: Option<Argument>, res: Option<Pool>},
    Min{arg: Option<Argument>, res: Option<Pool>},
}

impl Cap {
    pub fn apply(&self, token: RollToken, argument: Argument) -> Result<Self, RollError> {
        todo!()
    }
}

impl FromStr for Cap {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(mode) = s.trim().strip_prefix('c') {
            match mode {
                "" | "h" | "max"    => Ok(Cap::Max { arg: None, res: None }),
                "l" | "min"         => Ok(Cap::Min { arg: None, res: None }),
                _           => Err(RollError::SymbolError(s.into()))
            }
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Explode {
    Additive{arg: Option<Argument>, res: Vec<Pool>},
    Once{arg: Option<Argument>, res: Vec<Pool>},
    Recursive{arg: Option<Argument>, res: Vec<Pool>},
}

impl Explode {
    pub fn apply(&self, pool: Pool, argument: Argument) -> Result<Self, RollError> {
        let arg = Some(argument.clone());

        match self {
            Explode::Additive { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Single(explode_number) => pool.explode_n_additive(explode_number, true)?,
                    Argument::Array(explode_array) => pool.explode_specific_additive(&explode_array, true)?,
                };
                Ok(Explode::Additive { arg, res })
            },
            Explode::Once { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Single(explode_number) => pool.explode_n(explode_number, false)?,
                    Argument::Array(explode_array) => pool.explode_specific(&explode_array, false)?,
                };
                Ok(Explode::Once { arg, res })
            },
            Explode::Recursive { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Single(explode_number) => pool.explode_n(explode_number, true)?,
                    Argument::Array(explode_array) => pool.explode_specific(&explode_array, true)?,
                };
                Ok(Explode::Recursive { arg, res })
            },
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Explode::Additive { arg: _, res } => {
                match res.len() {
                    0 => Err(RollError::MissingPoolError),
                    _ => Ok(res.last().unwrap_or(&Pool::new(0, 0)).clone()),
                }
            },
            Explode::Once { arg: _, res } => {
                match res.len() {
                    0 => Err(RollError::MissingPoolError),
                    1 => Ok(res[0].clone()),
                    2 => Ok(res[0].add(&res[1])),
                    _ => Err(RollError::FBomb),
                }
            },
            Explode::Recursive { arg: _, res } => {
                match res.len() {
                    0 => Err(RollError::MissingPoolError),
                    _ => Ok(res.iter().fold(Pool::new(0, 0), |final_pool, pool| final_pool.add(pool)))
                }
            },
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        Ok(self.clone().pool()?.total().into())
    }

    pub fn description(&self) -> String {
        match self {
            Explode::Additive { arg, res: _ } => format!("For all dice showing {}, roll another one and add results", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Explode::Once { arg, res: _ } => format!("Explode dice showing {} once", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Explode::Recursive { arg, res: _ } => format!("Explode dice showing {} indefinitely", arg.as_ref().unwrap_or(&Argument::Single(0))),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Explode::Additive { arg: _, res } => {
                let mut summary = String::new();
                let mut results = res.clone();
                let total = match results.pop() {
                    None => return "Something went wrong! Please let the boss know!".into(),
                    Some(t) => t
                };
                // Skip the first value, which is the base pool
                for pool in results.iter().skip(1) {
                    if pool.total_number() == 0 {continue;}
                    summary = format!("{}Explode {} di(c)e -> {}\n", summary, pool.total_number(), pool);
                }
                summary = match summary.len() {
                    0 => format!("No exploded dice -> {}", total),
                    _ => format!("{}Total: {}", summary, total)
                };
                summary
            },
            Explode::Once { arg: _, res } => {
                match res.len() {
                    1 => format!("No exploded dice -> {}", res[0]),
                    2 => format!("Explode {} di(c)e -> {}, total: {}", res[1].total_number(), res[1], res[0].add(&res[1])),
                    _ => "Something went wrong! Please let the boss know!".into()
                }
            },
            Explode::Recursive { arg: _, res } => {
                let mut summary = String::new();
                // Skip the first value, which is the base pool
                for pool in res.iter().skip(1) {
                    if pool.total_number() == 0 {continue;}
                    summary = format!("{}Explode {} di(c)e -> {}\n", summary, pool.total_number(), pool);
                }
                summary = match summary.len() {
                    0 => format!("No exploded dice -> {}", res[0]),
                    _ => format!("{}Total: {}", summary, res.iter().fold(Pool::new(0, 0), |final_pool, pool| final_pool.add(pool)))
                };
                summary
            },
        }
    }
}

impl FromStr for Explode {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(mode) = s.trim().strip_prefix('e') {
            match mode {
                "" | "o"    => Ok(Explode::Once { arg: None, res: vec![] }),
                "r"         => Ok(Explode::Recursive { arg: None, res: vec![] }),
                "a"         => Ok(Explode::Additive { arg: None, res: vec![] }),
                _           => Err(RollError::SymbolError(s.into()))
            }
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Explode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Explode::Additive { arg, res: _ } => write!(f, "explode additive {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), self.clone().pool().unwrap_or(Pool::new(0, 0))),
            Explode::Once { arg, res: _ } => write!(f, "explode once {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), self.clone().pool().unwrap_or(Pool::new(0, 0))),
            Explode::Recursive { arg, res: _ } => write!(f, "explode recursive {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), self.clone().pool().unwrap_or(Pool::new(0, 0))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keep {
    Exact{arg: Option<Argument>, res: Option<Pool>},
    Low{arg: Option<Argument>, res: Option<Pool>},
    High{arg: Option<Argument>, res: Option<Pool>},
}

impl Keep {
    pub fn apply(&self, pool: Pool, argument: Argument) -> Result<Self, RollError> {
        let arg = Some(argument.clone());

        match self {
            Keep::Exact { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Array(keep_array) => Some(pool.keep_exact(&keep_array)),
                    Argument::Single(keep_number) => Some(pool.keep_exact(&[keep_number]))
                };
                Ok(Keep::Exact { arg, res })
            },
            Keep::High { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Array(keep_array) if keep_array.len() == 1 => Some(pool.keep_highest(keep_array[0])),
                    Argument::Array(_) => return Err(RollError::ArgumentError),
                    Argument::Single(keep_amount) => Some(pool.keep_highest(keep_amount))
                };
                Ok(Keep::High { arg, res })
            },
            Keep::Low { arg: _, res: _ } => {
                let res = match argument {
                    Argument::Array(keep_array) if keep_array.len() == 1 => Some(pool.keep_lowest(keep_array[0])),
                    Argument::Array(_) => return Err(RollError::ArgumentError),
                    Argument::Single(keep_amount) => Some(pool.keep_lowest(keep_amount))
                };
                Ok(Keep::Low { arg, res })
            },
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Keep::Exact { arg: _, res: pool } => pool.ok_or(RollError::NotResolvedError),
            Keep::High { arg: _, res: pool } => pool.ok_or(RollError::NotResolvedError),
            Keep::Low { arg: _, res: pool } => pool.ok_or(RollError::NotResolvedError),
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            Keep::Exact { arg: _, res: pool } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
            Keep::High { arg: _, res: pool } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
            Keep::Low { arg: _, res: pool } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Keep::Exact { arg, res: _ } => format!("Keep all dice showing {}", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Keep::High { arg, res: _ } => format!("Keep highest {} di(c)e", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Keep::Low { arg, res: _ } => format!("Keep lowest {} di(c)e", arg.as_ref().unwrap_or(&Argument::Single(0))),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Keep::Exact { arg: _, res } => {
                let default = Pool::new(0, 0);
                let result = res.as_ref().unwrap_or(&default);
                format!("Keep {} dice -> {}", result.dice().len(), result)
            },
            Keep::Low { arg: _, res } => {
                let default = Pool::new(0, 0);
                let result = res.as_ref().unwrap_or(&default);
                format!("Keep {} lowest -> {}", result.dice().len(), result)
            },
            Keep::High { arg: _, res } => {
                let default = Pool::new(0, 0);
                let result = res.as_ref().unwrap_or(&default);
                format!("Keep {} highest -> {}", result.dice().len(), result)
            },
        }
    }
}

impl FromStr for Keep {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(mode) = s.trim().strip_prefix('k') {
            match mode {
                "" | "h"    => Ok(Keep::High { arg: None, res: None }),
                "l"         => Ok(Keep::Low { arg: None, res: None }),
                "e"         => Ok(Keep::Exact { arg: None, res: None }),
                _           => Err(RollError::SymbolError(s.into()))
            }
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Keep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keep::Exact { arg, res } => write!(f, "keep exactly {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
            Keep::High { arg, res } => write!(f, "keep highest {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
            Keep::Low { arg, res } => write!(f, "keep lowest {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Reroll {
    Better{arg: Option<Argument>, res: Option<Pool>, rerolls: Option<Pool>},
    Once{arg: Option<Argument>, res: Option<Pool>, rerolls: Option<Pool>},
    Recursive{arg: Option<Argument>, res: Option<Pool>, rerolls: Option<Pool>},
    Worse{arg: Option<Argument>, res: Option<Pool>, rerolls: Option<Pool>},
}

impl Reroll {
    pub fn apply(&self, pool: Pool, argument: Argument) -> Result<Self, RollError> {
        let arg = Some(argument.clone());
        let mut rerolled_pool = pool.clone();

        match self {
            Reroll::Better { arg: _, res: _, rerolls: _ } => {
                match argument {
                    Argument::Array(array) => {
                        let new_dice = rerolled_pool.reroll_specific_better(&array);
                        Ok(Reroll::Better { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    },
                    Argument::Single(reroll_number) => {
                        let new_dice = rerolled_pool.reroll_n_better(reroll_number);
                        Ok(Reroll::Better { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    }
                }
            },
            Reroll::Once { arg: _, res: _, rerolls: _ } => {
                match argument {
                    Argument::Array(array) => {
                        let new_dice = rerolled_pool.reroll_specific(&array);
                        Ok(Reroll::Once { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    },
                    Argument::Single(reroll_number) => {
                        let new_dice = rerolled_pool.reroll_n(reroll_number);
                        Ok(Reroll::Once { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    }
                }
            },
            Reroll::Recursive { arg: _, res: _, rerolls: _ } => {
                match argument {
                    Argument::Array(array) => {
                        let new_dice = rerolled_pool.reroll_specific_recursive(&array);
                        Ok(Reroll::Recursive { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    },
                    Argument::Single(reroll_number) => {
                        let new_dice = rerolled_pool.reroll_n_recursive(reroll_number);
                        Ok(Reroll::Recursive { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    },
                }
            },
            Reroll::Worse { arg: _, res: _, rerolls: _ } => {
                match argument {
                    Argument::Array(array) => {
                        let new_dice = rerolled_pool.reroll_specific_worse(&array);
                        Ok(Reroll::Worse { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    },
                    Argument::Single(reroll_number) => {
                        let new_dice = rerolled_pool.reroll_n_worse(reroll_number);
                        Ok(Reroll::Worse { arg, res: Some(rerolled_pool), rerolls: Some(new_dice) })
                    }
                }
            },
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Reroll::Better { arg: _, res: pool, rerolls: _ } => pool.ok_or(RollError::NotResolvedError),
            Reroll::Once { arg: _, res: pool, rerolls: _ } => pool.ok_or(RollError::NotResolvedError),
            Reroll::Recursive { arg: _, res: pool, rerolls: _ } => pool.ok_or(RollError::NotResolvedError),
            Reroll::Worse { arg: _, res: pool, rerolls: _ } => pool.ok_or(RollError::NotResolvedError),
        }
    }

    pub fn value(&self) -> Result<RollValue, RollError> {
        match self {
            Reroll::Better { arg: _, res: pool, rerolls: _ } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
            Reroll::Once { arg: _, res: pool, rerolls: _ } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
            Reroll::Recursive { arg: _, res: pool, rerolls: _ } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
            Reroll::Worse { arg: _, res: pool, rerolls: _ } => Ok(pool.as_ref().ok_or(RollError::NotResolvedError)?.total().into()),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Reroll::Better { arg, res: _, rerolls: _ } => format!("Reroll all dice showing {} and keep the better result", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Reroll::Once { arg, res: _, rerolls: _ } => format!("Reroll all dice showing {} once", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Reroll::Recursive { arg, res: _, rerolls: _ } => format!("Reroll all dice showing {} until none appear", arg.as_ref().unwrap_or(&Argument::Single(0))),
            Reroll::Worse { arg, res: _, rerolls: _ } => format!("Reroll all dice showing {} and keep the worse result", arg.as_ref().unwrap_or(&Argument::Single(0))),
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Reroll::Better { arg: _, res, rerolls } => {
                format!(
                    "Reroll {} di(c)e -> {}, result: {}", 
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)).total_number(),
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)),
                    res.as_ref().unwrap_or(&Pool::new(0, 0))
                )
            },
            Reroll::Once { arg: _, res, rerolls } => {
                format!(
                    "Reroll {} di(c)e -> {}, result: {}", 
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)).total_number(),
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)),
                    res.as_ref().unwrap_or(&Pool::new(0, 0))
                )
            },
            Reroll::Recursive { arg: _, res, rerolls } => {
                format!(
                    "Reroll {} di(c)e -> {}, result: {}", 
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)).total_number(),
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)),
                    res.as_ref().unwrap_or(&Pool::new(0, 0))
                )
            },
            Reroll::Worse { arg: _, res, rerolls } => {
                format!(
                    "Reroll {} di(c)e -> {}, result: {}", 
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)).total_number(),
                    rerolls.as_ref().unwrap_or(&Pool::new(0, 0)),
                    res.as_ref().unwrap_or(&Pool::new(0, 0))
                )
            },
        }
    }
}

impl FromStr for Reroll {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(mode) = s.trim().strip_prefix('r') {
            match mode {
                "" | "o"    => Ok(Reroll::Once { arg: None , res: None, rerolls: None }),
                "r"         => Ok(Reroll::Recursive { arg: None , res: None, rerolls: None }),
                "b"         => Ok(Reroll::Better { arg: None , res: None, rerolls: None }),
                "w"         => Ok(Reroll::Worse { arg: None , res: None, rerolls: None }),
                _           => Err(RollError::SymbolError(s.into()))
            }
        } else {
            Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Reroll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reroll::Better { arg, res, rerolls: _ } => write!(f, "reroll keep better {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
            Reroll::Once { arg, res, rerolls: _ } => write!(f, "reroll once {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
            Reroll::Recursive { arg, res, rerolls: _ } => write!(f, "reroll recursively {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
            Reroll::Worse { arg, res, rerolls: _ } => write!(f, "reroll keep worse {} -> {}", arg.as_ref().unwrap_or(&Argument::Single(0)), res.as_ref().unwrap_or(&Pool::new(0, 0))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Success{arg: Option<Argument>, pool: Option<Pool>, sux: i16},
    Botch{arg: Option<Argument>, pool: Option<Pool>, sux: i16},
}

impl Target {
    pub fn apply(&self, token: RollToken, argument: Argument) -> Result<Self, RollError> {
        let arg = Some(argument.clone());
        let pool = Some(token.clone().pool()?);
        let base_sux = match &token {
            RollToken::Operator(Operator::Target(target)) => target.value().to_decimal()? as i16,
            _ => 0,
        };
        match argument {
            Argument::Single(threshold) => {
                match self {
                    Target::Success { arg: _, pool: _, sux: _ } => {
                        let sux = base_sux + token.pool()?.count_dice_over(threshold) as i16;
                        Ok(Target::Success { arg, pool, sux })
                    },
                    Target::Botch { arg: _, pool: _, sux: _ } => {
                        let sux = base_sux - (token.pool()?.count_dice_under(threshold) as i16);
                        Ok(Target::Botch { arg, pool, sux })
                    },
                }
            },
            Argument::Array(threshold_array) => {
                match self {
                    Target::Success { arg: _, pool: _, sux: _ } => {
                        let max_sides = token.clone().pool()?.sides_max() as usize;
                        let mut tns = vec![0; max_sides];
                        if tns.len() >= threshold_array.len() {
                            tns[max_sides - threshold_array.len()..].copy_from_slice(&threshold_array);
                        } else {
                            tns.copy_from_slice(&threshold_array[..max_sides]);
                        }

                        let sux = token.pool()?.count_successes(&tns) as i16;
                        let arg = Some(Argument::Array(tns));
                        Ok(Target::Success { arg, pool, sux })
                    },
                    Target::Botch { arg: _, pool: _, sux: _ } => {
                        let max_sides = token.clone().pool()?.sides_max() as usize;
                        let mut tns = vec![0; max_sides];
                        if tns.len() >= threshold_array.len() {
                            tns[..threshold_array.len()].copy_from_slice(&threshold_array);
                        } else {
                            tns.copy_from_slice(&threshold_array[..max_sides]);
                        }

                        let sux = - (token.pool()?.count_successes(&threshold_array) as i16);
                        let arg = Some(Argument::Array(tns));
                        Ok(Target::Botch { arg, pool, sux })
                    },
                }
            },
        }
    }

    pub fn pool(self) -> Result<Pool, RollError> {
        match self {
            Target::Success { arg: _, pool, sux: _ } => pool.ok_or(RollError::MissingPoolError),
            Target::Botch { arg: _, pool, sux: _ } => pool.ok_or(RollError::MissingPoolError),
        }
    }

    pub fn value(&self) -> RollValue {
        match self {
            Target::Success { arg: _, pool: _, sux } => RollValue::Successes(*sux),
            Target::Botch { arg: _, pool: _, sux } => RollValue::Successes(*sux),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Target::Success { arg, pool: _, sux: _ } => match arg {
                Some(argument) => match argument {
                    Argument::Single(n) => format!("Count one success per die showing {} or higher", n),
                    Argument::Array(a) => format!("Count successes: {:?}", a),
                },
                None => "Something went wrong! Pleasse let the boss know!".into(),
            },
            Target::Botch { arg, pool: _, sux: _ } => match arg {
                Some(argument) => match argument {
                    Argument::Single(n) => format!("Subtract one success per die showing {} or lower", n),
                    Argument::Array(a) => format!("Subtract successes: {:?}", a),
                },
                None => "Something went wrong! Pleasse let the boss know!".into(),
            },
        }
    }

    pub fn verbose(&self) -> String {
        match self {
            Target::Success { arg, pool, sux } => match arg {
                Some(argument) => match argument {
                    Argument::Single(n) => format!(
                        "{} -> {:?} = {} success(es)",
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)),
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)).dice().iter().map(|d| if d.equal_or_greater(*n) {1} else {0}).collect::<Vec<i16>>(),
                        sux
                    ),
                    Argument::Array(a) => format!(
                        "{} -> {:?} = {} success(es)",
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)),
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)).dice().iter().map(|d| d.count_successes(a)).collect::<Vec<u8>>(),
                        sux
                    ),
                },
                None => "Something went wrong! Pleasse let the boss know!".into(),
            },
            Target::Botch { arg, pool, sux } => match arg {
                Some(argument) => match argument {
                    Argument::Single(n) => format!(
                        "{} -> {:?} = {} success(es)",
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)),
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)).dice().iter().map(|d| if d.equal_or_less(*n) {-1} else {0}).collect::<Vec<i16>>(),
                        sux
                    ),
                    Argument::Array(a) => format!(
                        "{} -> {:?} = {} success(es)",
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)),
                        pool.as_ref().unwrap_or(&Pool::new(0, 0)).dice().iter().map(|d| - (d.count_successes(a) as i16)).collect::<Vec<i16>>(),
                        sux
                    ),
                },
                None => "Something went wrong! Pleasse let the boss know!".into(),
            },
        }
    }
}

impl FromStr for Target {
    type Err = RollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" => Ok(Target::Success { arg: None, pool: None, sux: 0 }),
            "b" => Ok(Target::Botch { arg: None, pool: None, sux: 0 }),
            _   => Err(RollError::SymbolError(s.into()))
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Success { arg, pool: _, sux } => {
                match arg.as_ref().unwrap_or(&Argument::Single(0)) {
                    Argument::Single(threshold) => {
                        write!(f, "success on {} or higher -> {} success(es)", threshold, sux)
                    },
                    Argument::Array(thresh_array) => {
                        let t_values = thresh_array.iter().enumerate();
                        let t_string = t_values.fold(String::new(), |acc, (index, &value)| {
                            if value == 0 {
                                acc
                            } else {
                                format!("{}{}: {} sux, ", acc, index + 1, value)
                            }
                        });
                        match t_string.strip_suffix(", ") {
                            Some(output) => write!(f, "count successes: {} -> {} success(es)", output, sux),
                            None => write!(f, "no success counting rule given -> {} success(es)", sux),
                        }
                    },
                }
            },
            Target::Botch { arg, pool: _, sux } => {
                match arg.as_ref().unwrap_or(&Argument::Single(0)) {
                    Argument::Single(threshold) => {
                        write!(f, "subtract success on {} or lower -> {} success(es)", threshold, sux)
                    },
                    Argument::Array(thresh_array) => {
                        let t_values = thresh_array.iter().enumerate();
                        let t_string = t_values.fold(String::new(), |acc, (index, &value)| {
                            if value == 0 {
                                acc
                            } else {
                                format!("{}{}: -{} sux, ", acc, index + 1, value)
                            }
                        });
                        match t_string.strip_suffix(", ") {
                            Some(output) => write!(f, "subtract successes: {} -> {} success(es)", output, sux),
                            None => write!(f, "no success counting rule given -> {} success(es)", sux),
                        }
                    },
                }
            },
        }
    }
}
