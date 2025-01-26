#[derive(Clone, Debug, PartialEq)]
pub struct RollResult {
    pub rolls: Vec<DiceRollResult>,
    pub modifier: isize,
}

impl Default for RollResult {
    fn default() -> Self {
        RollResult {
            rolls: vec![],
            modifier: 0,
        }
    }
}

impl RollResult {
    pub fn new() -> Self {
        RollResult::default()
    }

    pub fn add_roll(&mut self, roll: DiceRollResult) {
        self.rolls.push(roll);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiceRollResult {
    pub descriptor: String,
    pub roll: isize,
}

impl DiceRollResult {
    pub fn new<T: Into<String>>(descriptor: T, roll: isize) -> Self {
        let descriptor = descriptor.into();
        DiceRollResult { descriptor, roll }
    }
}
