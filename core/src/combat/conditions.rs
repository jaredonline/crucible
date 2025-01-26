#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ConditionType {
    Poisoned,
    Stunned,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub condition: ConditionType,
    pub duration: Duration,
    pub effects: Vec<Effect>,
}

impl Condition {
    pub fn poisoned(duration: Duration) -> Self {
        Condition {
            condition: ConditionType::Poisoned,
            duration,
            effects: vec![Effect::DisadvantageOnAttacks],
        }
    }

    pub fn stunned(duration: Duration) -> Self {
        Condition {
            condition: ConditionType::Stunned,
            duration,
            effects: vec![Effect::CantTakeActions],
        }
    }

    pub fn copy_deprecate_duration(condition: &Condition) -> Self {
        let mut duration = condition.duration.clone();
        duration.deprecate();
        Condition {
            condition: condition.condition.clone(),
            duration,
            effects: condition.effects.clone(),
        }
    }

    pub fn deprecate_duration(&mut self) {
        self.duration.deprecate();
    }

    pub fn still_active(&self) -> bool {
        self.duration.ongoing()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Duration {
    Rounds(usize),
}

impl Duration {
    pub fn deprecate(&mut self) {
        match self {
            Self::Rounds(length) => {
                if *length > 1 {
                    *length -= 1;
                } else {
                    *length = 0;
                }
            }
        }
    }

    pub fn remaining(&self) -> usize {
        match self {
            Self::Rounds(length) => *length,
        }
    }

    pub fn ongoing(&self) -> bool {
        match self {
            Self::Rounds(length) => *length > 0,
        }
    }
}

impl Into<Duration> for usize {
    fn into(self) -> Duration {
        Duration::Rounds(self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Effect {
    DisadvantageOnAttacks,
    CantTakeActions,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Action, Character, Team};

    #[test]
    fn test_basic_conditions() {
        let mut fighter = Character::new("Fighter", 10, 15, Team::Heroes, 1);

        fighter.add_condition(Condition::poisoned(Duration::Rounds(3)));

        // Test duration tracking
        assert!(fighter.has_condition(ConditionType::Poisoned));
        fighter.end_turn(); // Round 1
        assert!(fighter.has_condition(ConditionType::Poisoned));
        fighter.end_turn(); // Round 2
        fighter.end_turn(); // Round 3
        assert!(!fighter.has_condition(ConditionType::Poisoned));
    }

    #[test]
    fn test_condition_effects() {
        let mut fighter = Character::new("Fighter", 10, 15, Team::Heroes, 1);
        let attack = Action::Attack {
            name: "Attack".into(),
            hit_bonus: 1,
            damage: "1d4".into(),
        };
        fighter.add_action(attack.clone());
        let monster = fighter.clone();
        let monsters = vec![monster];

        // Test disadvantage from being poisoned
        fighter.add_condition(Condition::poisoned(1.into()));
        assert!(fighter.has_disadvantage_on(&attack));

        // Test being stunned prevents actions
        fighter.add_condition(Condition::stunned(1.into()));
        assert!(fighter.valid_actions(&vec![], &monsters).is_empty());

        fighter.end_turn();
        assert!(!fighter.valid_actions(&vec![], &monsters).is_empty());
    }

    // #[test]
    // fn test_condition_durations() {
    //     let mut fighter = Character::new("Fighter", 10, 15, Team::Heroes, 1);

    //     // Test various duration types
    //     fighter.add_condition(Condition::Blessed {
    //         duration: Duration::UntilDispelled,
    //         effects: vec![Effect::BonusToAttacks(2)],
    //     });

    //     fighter.add_condition(Condition::Frightened {
    //         duration: Duration::SaveEnd {
    //             dc: 15,
    //             ability: Ability::Wisdom,
    //         },
    //         effects: vec![Effect::DisadvantageOnAttacks],
    //     });

    //     // Test save-based removal
    //     fighter.make_save(Ability::Wisdom, 16);
    //     assert!(!fighter.has_condition(Condition::Frightened));

    //     // Test dispel
    //     fighter.remove_condition(Condition::Blessed);
    //     assert!(!fighter.has_condition(Condition::Blessed));
    // }
}
