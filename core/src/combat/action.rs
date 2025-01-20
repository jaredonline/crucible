use crate::{Character, DicePool, InitiativeEntry};

#[derive(Clone, Debug)]
pub enum Action {
    Attack {
        name: String,
        hit_bonus: isize,
        damage: DicePool,
    },
    Heal {
        name: String,
        healing: DicePool,
    },
}

impl Action {
    pub fn is_valid(&self, allies: &Vec<Character>, enemies: &Vec<Character>) -> bool {
        self.valid_targets(allies, enemies).len() > 0
    }

    pub fn valid_targets(
        &self,
        allies: &Vec<Character>,
        enemies: &Vec<Character>,
    ) -> Vec<InitiativeEntry> {
        match self {
            Action::Attack { .. } => enemies
                .iter()
                .enumerate()
                .filter(|(_i, c)| c.current_hp > 0)
                .map(|(i, c)| InitiativeEntry {
                    team: c.team,
                    index: i,
                    initiative: 0,
                })
                .collect(),
            Action::Heal { .. } => allies
                .iter()
                .enumerate()
                .filter(|(_i, c)| c.current_hp < c.max_hp)
                .map(|(i, c)| InitiativeEntry {
                    team: c.team,
                    index: i,
                    initiative: 0,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    Attack { hit: HitResult, damage: usize },
    Heal { amount: usize },
    None,
}

#[derive(Debug, Clone)]
pub enum HitResult {
    Hit,
    Miss,
    Critical,
}
