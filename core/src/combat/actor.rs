use crate::{Action, ActionResult, DicePool, HitResult, Team};

#[derive(Clone, Debug)]
pub struct Character {
    pub name: String,
    pub max_hp: usize,
    pub current_hp: usize,
    pub ac: usize,
    pub actions: Vec<Action>,
    pub team: Team,
}

impl Character {
    pub fn new<T: Into<String>>(name: T, max_hp: usize, ac: usize, team: Team) -> Self {
        Character {
            name: name.into(),
            max_hp,
            current_hp: max_hp,
            ac,
            actions: vec![],
            team,
        }
    }

    pub fn take_action(&self, target: &mut Character, action: &Action) -> ActionResult {
        match action {
            Action::Attack {
                name: _,
                hit_bonus,
                damage,
            } => {
                let attack_roll = DicePool::d20().add_modifier(*hit_bonus).roll();
                if attack_roll == 20 {
                    let damage = damage.critical_hit();
                    let damage = damage.roll();
                    // damage cannot be negative
                    let damage: usize = if damage > 0 { damage as usize } else { 0 };
                    if damage > target.current_hp {
                        target.current_hp = 0;
                    } else {
                        target.current_hp -= damage;
                    }
                    ActionResult::Attack {
                        hit: HitResult::Critical,
                        damage: damage,
                    }
                } else if attack_roll >= target.ac as isize {
                    let damage = damage.roll();
                    // damage cannot be negative
                    let damage: usize = if damage > 0 { damage as usize } else { 0 };
                    if damage > target.current_hp {
                        target.current_hp = 0;
                    } else {
                        target.current_hp -= damage;
                    }
                    ActionResult::Attack {
                        hit: HitResult::Hit,
                        damage: damage,
                    }
                } else {
                    ActionResult::Attack {
                        hit: HitResult::Miss,
                        damage: 0,
                    }
                }
            }
            Action::Heal { name: _, healing } => {
                let healing = healing.roll() as usize;
                target.current_hp += healing;
                if target.current_hp > target.max_hp {
                    target.current_hp = target.max_hp;
                }
                ActionResult::Heal { amount: healing }
            }
        }
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn with_actions(mut self, mut actions: Vec<Action>) -> Self {
        self.actions.append(&mut actions);
        self
    }

    pub fn valid_actions(&self, allies: &Vec<Character>, enemies: &Vec<Character>) -> Vec<Action> {
        self.actions
            .iter()
            .filter(|a| a.is_valid(allies, enemies))
            .cloned()
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InitiativeEntry {
    pub team: Team,
    pub index: usize,
}
