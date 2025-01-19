use std::cell::RefCell;

use rand::Rng;

use crate::AdvantageType;

use super::roll::{DiceRollResult, RollResult};

#[derive(Debug, Clone)]
pub struct DicePool {
    pub dice: Vec<Dice>,
    pub modifier: isize,
    pub last_roll: RefCell<Option<RollResult>>,
    pub advantage_type: AdvantageType,
}

impl Default for DicePool {
    fn default() -> Self {
        DicePool {
            dice: vec![],
            modifier: 0,
            last_roll: RefCell::new(None),
            advantage_type: AdvantageType::None,
        }
    }
}

impl DicePool {
    pub fn new() -> Self {
        DicePool::default()
    }

    pub fn d20() -> Self {
        DicePool::new().add_dice(1, 20)
    }

    pub fn with_advantage(mut self) -> Self {
        self.advantage_type = AdvantageType::Advantage;
        self
    }

    pub fn with_disadvantage(mut self) -> Self {
        self.advantage_type = AdvantageType::Disadvantage;
        self
    }

    pub fn critical_hit(&self) -> Self {
        let mut pool = vec![];
        for d in &self.dice {
            pool.push(Dice::new(d.count * 2, d.sides));
        }

        DicePool {
            dice: pool,
            modifier: self.modifier,
            last_roll: RefCell::new(None),
            advantage_type: AdvantageType::None,
        }
    }

    pub fn add_dice(mut self, count: usize, sides: usize) -> Self {
        self.dice.push(Dice::new(count, sides));
        self
    }

    pub fn add_modifier(mut self, modifier: isize) -> Self {
        self.modifier += modifier;
        self
    }

    pub fn roll(&self) -> isize {
        let mut roll_result = RollResult::new();
        let mut total = 0;

        match self.advantage_type {
            AdvantageType::Advantage => {
                let mut roll1 = 0;
                let mut roll2 = 0;
                for d in &self.dice {
                    let roll = d.roll();
                    roll_result.add_roll(DiceRollResult::new(d.to_string(), roll));
                    roll1 += roll;
                }

                for d in &self.dice {
                    let roll = d.roll();
                    roll_result.add_roll(DiceRollResult::new(d.to_string(), roll));
                    roll2 += roll;
                }

                if roll1 >= roll2 {
                    total += roll1;
                } else {
                    total += roll2;
                }
            }
            AdvantageType::Disadvantage => {
                let mut roll1 = 0;
                let mut roll2 = 0;
                for d in &self.dice {
                    let roll = d.roll();
                    roll_result.add_roll(DiceRollResult::new(d.to_string(), roll));
                    roll1 += roll;
                }

                for d in &self.dice {
                    let roll = d.roll();
                    roll_result.add_roll(DiceRollResult::new(d.to_string(), roll));
                    roll2 += roll;
                }

                if roll1 < roll2 {
                    total += roll1;
                } else {
                    total += roll2;
                }
            }
            AdvantageType::None => {
                for d in &self.dice {
                    let roll = d.roll();
                    roll_result.add_roll(DiceRollResult::new(d.to_string(), roll));
                    total += roll;
                }
            }
        }

        total += self.modifier;
        roll_result.modifier = self.modifier;

        *self.last_roll.borrow_mut() = Some(roll_result);

        total
    }

    pub fn from_str<T: Into<String>>(s: T) -> Result<Self, String> {
        let s = s.into();
        let mut pool = DicePool::new();

        // Replace any "-" with "+-" so we can split on "+" and preserve the negative sign
        let s = s.replace("-", "+-");

        for part in s.split('+') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let is_negative = part.starts_with('-');
            let part = part.trim_start_matches('-');
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.contains('d') {
                let mut parts = part.split('d');
                let count = parts.next().and_then(|n| n.parse().ok()).unwrap_or(1);
                let sides = parts
                    .next()
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| "Invalid dice format".to_string())?;

                let mut dice = Dice::new(count, sides);
                dice.negative = is_negative;
                pool.dice.push(dice);
            } else if let Ok(modifier) = part.parse::<isize>() {
                pool = pool.add_modifier(if is_negative { -modifier } else { modifier });
            }
        }

        Ok(pool)
    }

    pub fn count_dice(&self, sides: usize) -> usize {
        self.dice
            .iter()
            .filter(|d| d.sides == sides)
            .map(|d| d.count)
            .sum()
    }

    pub fn debug_last_roll(&self) -> Option<RollResult> {
        let last_roll = self.last_roll.borrow();
        match *last_roll {
            None => None,
            Some(ref last_roll) => Some(last_roll.clone()),
        }
    }
}

impl From<&str> for DicePool {
    fn from(value: &str) -> Self {
        DicePool::from_str(value).unwrap()
    }
}

impl ToString for DicePool {
    fn to_string(&self) -> String {
        let mut parts = Vec::new();

        // Group dice by sides and sign
        let mut dice_map: std::collections::HashMap<(usize, bool), usize> =
            std::collections::HashMap::new();
        for die in &self.dice {
            *dice_map.entry((die.sides, die.negative)).or_default() += die.count;
        }

        // Sort by dice sides for consistent output
        let mut sides: Vec<_> = dice_map.keys().collect();
        sides.sort_by(|a, b| b.0.cmp(&a.0));

        for &(side, negative) in sides {
            if let Some(&count) = dice_map.get(&(side, negative)) {
                let prefix = if negative {
                    "-"
                } else if !parts.is_empty() {
                    "+"
                } else {
                    ""
                };
                parts.push(format!("{}{:}d{}", prefix, count, side));
            }
        }

        if self.modifier != 0 {
            let prefix = if self.modifier < 0 {
                ""
            } else if !parts.is_empty() {
                "+"
            } else {
                ""
            };
            parts.push(format!("{}{}", prefix, self.modifier));
        }

        parts.join("")
    }
}

impl From<String> for DicePool {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Dice {
    pub sides: usize,
    pub count: usize,
    pub negative: bool,
}

impl Dice {
    fn new(count: usize, sides: usize) -> Self {
        Dice {
            sides,
            count,
            negative: false,
        }
    }

    fn roll(&self) -> isize {
        let mut rng = rand::thread_rng();
        let sum: usize = (0..self.count).map(|_| rng.gen_range(1..=self.sides)).sum();
        if self.negative {
            sum as isize * -1
        } else {
            sum as isize
        }
    }

    fn to_string(&self) -> String {
        let mut parts = vec![];
        if self.negative {
            parts.push("-");
        }

        let count = self.count.to_string();
        let sides = self.sides.to_string();
        parts.push(count.as_str());
        parts.push("d");
        parts.push(sides.as_str());

        parts.join("")
    }
}

pub fn roll_dice<T: Into<String>>(roll: T) -> isize {
    DicePool::from_str(roll.into()).unwrap().roll()
}
