mod action;
mod actor;
mod conditions;
mod log;

use std::{collections::HashMap, vec};

pub use action::{Action, ActionResult, HitResult};
pub use actor::{Character, InitiativeEntry, ResourceType};
pub use conditions::{Condition, Duration, Effect};
pub use log::ActivityLog;

use crate::{Combat, Team};

pub fn build_level_one_combat(num_kobolds: usize) -> Combat {
    let fighter = Character::new("Fighter", 12, 16, Team::Heroes, 1)
        .with_actions(vec![
            Action::Attack {
                name: "Greatsword".into(),
                hit_bonus: 5,
                damage: "2d6+3".into(),
            },
            Action::SecondWind {
                healing: "1d10+1".into(),
                required_resources: vec![(ResourceType::Feature("Second Wind".into()), 1)],
            },
        ])
        .with_resources(HashMap::from([(
            ResourceType::Feature("Second Wind".into()),
            1,
        )]));
    let cleric = Character::new("Cleric", 10, 16, Team::Heroes, 0).with_actions(vec![
        Action::Attack {
            name: "Mace".into(),
            hit_bonus: 4,
            damage: "1d6+2".into(),
        },
        Action::Heal {
            name: "Healing Word".into(),
            healing: "1d8+3".into(),
            required_resources: vec![],
        },
    ]);
    let rogue =
        Character::new("Rogue", 9, 14, Team::Heroes, 3).with_actions(vec![Action::Attack {
            name: "Rapier".into(),
            hit_bonus: 5,
            damage: "1d8+3".into(),
        }]);
    let heroes = vec![fighter, cleric, rogue];

    let kobold_dagger = Action::Attack {
        name: "Dagger".into(),
        hit_bonus: 4,
        damage: "1d4+2".into(),
    };
    let kobold_sling = Action::Attack {
        name: "Sling".into(),
        hit_bonus: 4,
        damage: "1d4+2".into(),
    };
    let kobold_actions = vec![kobold_dagger, kobold_sling];
    let monsters = (0..num_kobolds)
        .into_iter()
        .map(|i| {
            Character::new(format!("Kobold {}", i + 1), 5, 12, Team::Monsters, 2)
                .with_actions(kobold_actions.clone())
        })
        .collect();

    Combat::new(heroes, monsters)
}
