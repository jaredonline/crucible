use crate::{Character, DicePool, InitiativeEntry};

use super::ResourceType;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Attack {
        name: String,
        hit_bonus: isize,
        damage: DicePool,
    },
    Heal {
        name: String,
        healing: DicePool,
        required_resources: Vec<(ResourceType, usize)>,
    },
    SecondWind {
        healing: DicePool,
        required_resources: Vec<(ResourceType, usize)>,
    },
}

impl Action {
    pub fn is_valid(
        &self,
        actor: &Character,
        allies: &Vec<Character>,
        enemies: &Vec<Character>,
    ) -> bool {
        self.resources_available(actor) && self.valid_targets(actor, allies, enemies).len() > 0
    }

    pub fn valid_targets(
        &self,
        actor: &Character,
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
            Action::SecondWind { .. } => allies
                .iter()
                .enumerate()
                .filter(|(_i, c)| *c == actor)
                .filter(|(_i, c)| c.current_hp < c.max_hp)
                .map(|(i, c)| InitiativeEntry {
                    team: c.team,
                    index: i,
                    initiative: 0,
                })
                .collect(),
        }
    }

    fn resources_available(&self, actor: &Character) -> bool {
        match self {
            Action::Heal {
                required_resources, ..
            } => required_resources
                .iter()
                .all(|(r_type, amount)| actor.has_resource(r_type, *amount)),
            _ => true,
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

#[cfg(test)]
mod action_tests {
    use crate::{combat::ResourceType, Action, Character, Team};

    #[test]
    fn test_action_resource_requirements() {
        let mut fighter = Character::new("Fighter", 10, 15, Team::Heroes, 2);
        let mut ally = Character::new("Ally", 10, 10, Team::Heroes, 0);
        ally.current_hp = 1;
        let allies = vec![ally];

        // Action that requires Second Wind
        let second_wind = Action::Heal {
            name: "Second Wind".into(),
            healing: "1d10+1".into(),
            required_resources: vec![(ResourceType::Feature("Second Wind".into()), 1)],
        };

        fighter.add_action(second_wind.clone());
        fighter.add_resource(ResourceType::Feature("Second Wind".into()), 1);

        // Action is valid when resource available
        assert!(second_wind.is_valid(&fighter, &allies, &vec![]));

        // Use the resource
        fighter
            .spend_resource(ResourceType::Feature("Second Wind".into()), 1)
            .unwrap();

        // Action invalid when resource depleted
        assert!(!second_wind.is_valid(&fighter, &allies, &vec![]));
    }
}
