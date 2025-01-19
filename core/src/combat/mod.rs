mod action;
mod actor;
mod log;

pub use action::{Action, ActionResult, HitResult};
pub use actor::{Character, InitiativeEntry};
pub use log::ActivityLog;

use crate::{Combat, Team};

pub fn build_level_one_combat(num_kobolds: usize) -> Combat {
    let fighter =
        Character::new("Fighter", 12, 16, Team::Heroes).with_actions(vec![Action::Attack {
            name: "Greatsword".into(),
            hit_bonus: 5,
            damage: "2d6+3".into(),
        }]);
    let cleric = Character::new("Cleric", 10, 16, Team::Heroes).with_actions(vec![
        Action::Attack {
            name: "Mace".into(),
            hit_bonus: 4,
            damage: "1d6+2".into(),
        },
        Action::Heal {
            name: "Healing Word".into(),
            healing: "1d8+3".into(),
        },
    ]);
    let rogue = Character::new("Rogue", 9, 14, Team::Heroes).with_actions(vec![Action::Attack {
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
            Character::new(format!("Kobold {}", i + 1), 5, 12, Team::Monsters)
                .with_actions(kobold_actions.clone())
        })
        .collect();

    Combat::new(heroes, monsters)
}
