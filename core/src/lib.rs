pub mod combat;
mod dice;
pub mod monte_carlo;
mod team;

pub use combat::{Action, ActionResult, ActivityLog, Character, HitResult, InitiativeEntry};
pub use dice::{roll_dice, DicePool};
use rand::{seq::SliceRandom, thread_rng};
pub use team::Team;

#[derive(Debug)]
pub struct Combat {
    heroes: Vec<Character>,
    monsters: Vec<Character>,
    initiative_order: Vec<InitiativeEntry>,
    pub round: usize,
    debug_mode: bool,
    pub debug_log: Vec<ActivityLog>,
}

impl Combat {
    pub fn new(heroes: Vec<Character>, monsters: Vec<Character>) -> Self {
        Combat {
            heroes,
            monsters,
            initiative_order: vec![],
            debug_mode: false,
            debug_log: vec![],
            round: 1,
        }
    }

    pub fn debug(&mut self, val: bool) {
        self.debug_mode = val;
    }

    pub fn roll_initiative(&mut self) {
        let mut entries: Vec<InitiativeEntry> = (0..self.heroes.len())
            .map(|i| InitiativeEntry {
                team: Team::Heroes,
                index: i,
            })
            .chain((0..self.monsters.len()).map(|i| InitiativeEntry {
                team: Team::Monsters,
                index: i,
            }))
            .collect();
        entries.shuffle(&mut thread_rng());
        self.initiative_order = entries;
    }

    pub fn execute_round(&mut self) {
        for i in self.initiative_order.clone() {
            if self.lookup_character(i).current_hp == 0 {
                continue;
            }
            let valid_actions = self.valid_actions_for(i.team, i.index);
            let action = valid_actions.choose(&mut rand::thread_rng());
            if action.is_none() {
                continue;
            }
            let action = action.unwrap();
            let valid_targets = match i.team {
                Team::Heroes => {
                    let hero = &self.heroes[i.index];
                    action.valid_targets(self.teammates_for(hero), self.valid_targets_for(hero))
                }
                Team::Monsters => {
                    let monster = &self.monsters[i.index];
                    action
                        .valid_targets(self.teammates_for(monster), self.valid_targets_for(monster))
                }
            };
            let target = valid_targets
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();
            let result = self.execute_action(i, action, target);
            if self.debug_mode {
                self.debug_log.push(ActivityLog {
                    round: self.round,
                    action: action.clone(),
                    actor: self.lookup_character(i).clone(),
                    target: self.lookup_character(target).clone(),
                    result: result.clone(),
                    snapshot_heroes: self.heroes.clone(),
                    snapshot_monsters: self.monsters.clone(),
                })
            };
        }

        self.round += 1;
    }

    pub fn is_ongoing(&self) -> bool {
        !(self.heroes.iter().all(|c| c.current_hp == 0)
            || self.monsters.iter().all(|c| c.current_hp == 0))
    }

    fn valid_targets_for(&self, character: &Character) -> &Vec<Character> {
        match character.team {
            Team::Heroes => &self.monsters,
            Team::Monsters => &self.heroes,
        }
    }

    fn lookup_character(&self, init: InitiativeEntry) -> &Character {
        match init.team {
            Team::Heroes => &self.heroes[init.index],
            Team::Monsters => &self.monsters[init.index],
        }
    }

    fn teammates_for(&self, character: &Character) -> &Vec<Character> {
        match character.team {
            Team::Heroes => &self.heroes,
            Team::Monsters => &self.monsters,
        }
    }

    fn valid_actions_for(&self, team: Team, character_index: usize) -> Vec<Action> {
        match team {
            Team::Heroes => {
                let hero = &self.heroes[character_index];
                hero.valid_actions(self.teammates_for(hero), self.valid_targets_for(hero))
            }
            Team::Monsters => {
                let monster = &self.monsters[character_index];
                monster.valid_actions(self.teammates_for(monster), self.valid_targets_for(monster))
            }
        }
    }

    fn execute_action(
        &mut self,
        actor_entry: InitiativeEntry,
        action: &Action,
        target_entry: InitiativeEntry,
    ) -> ActionResult {
        match actor_entry.team {
            Team::Heroes => {
                let actor = &self.heroes[actor_entry.index].clone();
                match target_entry.team {
                    Team::Heroes => {
                        actor.take_action(self.heroes.get_mut(target_entry.index).unwrap(), action)
                    }
                    Team::Monsters => actor
                        .take_action(self.monsters.get_mut(target_entry.index).unwrap(), action),
                }
            }
            Team::Monsters => {
                let actor = &self.monsters[actor_entry.index].clone();
                match target_entry.team {
                    Team::Heroes => {
                        actor.take_action(self.heroes.get_mut(target_entry.index).unwrap(), action)
                    }
                    Team::Monsters => actor
                        .take_action(self.monsters.get_mut(target_entry.index).unwrap(), action),
                }
            }
        }
    }

    pub fn heroes_won(&self) -> bool {
        if self.is_ongoing() {
            false
        } else if self.heroes.iter().all(|c| c.current_hp == 0) {
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_fighter() -> Character {
        let mut fighter = Character::new("Fighter", 10, 15, Team::Heroes);
        fighter.add_action(Action::Attack {
            name: "Shortsword".into(),
            hit_bonus: 4,
            damage: "1d6+2".into(),
        });
        fighter
    }

    fn create_kobold() -> Character {
        let mut kobold = Character::new("Kobold", 5, 12, Team::Monsters);
        kobold.add_action(Action::Attack {
            name: "Dagger".into(),
            hit_bonus: 2,
            damage: "1d4+1".into(),
        });
        kobold
    }

    #[test]
    fn test_single_round() {
        let fighter = create_fighter();
        let kobold = create_kobold();

        let mut combat = Combat::new(vec![fighter], vec![kobold]);
        combat.execute_round();

        // After one round:
        // Both teams should still have participants
        assert!(!combat.heroes.is_empty());
        assert!(!combat.monsters.is_empty());

        // Each participant should have HP between 0 and their starting value
        for hero in combat.heroes {
            assert!(hero.current_hp <= 10);
        }

        for monster in combat.monsters {
            assert!(monster.current_hp <= 5);
        }
    }

    #[test]
    fn test_targeting() {
        let fighter1 = create_fighter();
        let fighter2 = Character::new("Fighter 2", 10, 15, Team::Heroes);
        let kobold1 = create_kobold();
        let kobold2 = Character::new("Kobold 2", 5, 12, Team::Monsters);

        let combat = Combat::new(vec![fighter1, fighter2], vec![kobold1, kobold2]);

        // When a character needs a target, they should only get enemies
        let hero = &combat.heroes[0];
        let valid_targets = combat.valid_targets_for(hero);

        assert_eq!(valid_targets.len(), 2); // Both kobolds
        assert!(valid_targets.iter().all(|t| t.team == Team::Monsters));
    }

    #[test]
    fn test_basic_combat_setup() {
        let pc = create_fighter();
        let kobold = create_kobold();

        let combat = Combat::new(
            vec![pc],     // Team Heroes
            vec![kobold], // Team Monsters
        );

        assert_eq!(combat.heroes.len(), 1);
        assert_eq!(combat.monsters.len(), 1);
    }

    #[test]
    fn test_character_attacks() {
        let fighter = create_fighter();
        assert!(!fighter.actions.is_empty());

        let attack = &fighter.actions[0];
        match attack {
            Action::Attack {
                name, hit_bonus, ..
            } => {
                assert_eq!(name, "Shortsword");
                assert_eq!(hit_bonus, &4);
            }
            _ => assert!(false),
        }

        assert_eq!(fighter.team, Team::Heroes);
    }

    #[test]
    fn test_basic_attack() {
        let fighter = create_fighter();
        let mut kobold = create_kobold();

        let result = fighter.take_action(&mut kobold, &fighter.actions[0]); // Use first attack

        match result {
            ActionResult::Attack { hit, damage } => {
                match hit {
                    HitResult::Hit => {
                        assert!(damage >= 3 && damage <= 8); // 1d6+2
                        assert!(kobold.current_hp <= 5);
                    }
                    HitResult::Miss => {
                        assert_eq!(kobold.current_hp, 5);
                    }
                    HitResult::Critical => {
                        assert!(damage >= 4 && damage <= 14); // 1d6+2
                        assert!(kobold.current_hp <= 5);
                    }
                }
            }
            _ => {}
        }
    }

    #[test]
    fn test_attack_guaranteed_hit() {
        let mut pc = Character::new("Fighter", 10, 15, Team::Heroes);
        pc.add_action(Action::Attack {
            name: "Magic Sword".into(),
            hit_bonus: 19,
            damage: "1d8+4".into(),
        });
        let mut kobold = create_kobold();

        // Even with a roll of 4 it will hit (4 + 8 >= 12)
        // This should hit unless we roll a natural 1
        let result = pc.take_action(&mut kobold, &pc.actions[0]);

        assert!(matches!(result, ActionResult::Attack { .. }));
        //  assert!(result.damage >= 5); // Damage should be 5-12 (1d8+4)
        assert!(kobold.current_hp <= 5);
    }

    #[test]
    fn test_attack_guaranteed_miss() {
        let pc = create_fighter();
        let mut kobold = Character::new("Kobold", 5, 30, Team::Monsters); // Very high AC

        // Even with a roll of 17 it will miss (17 + 2 < 30)
        let result = pc.take_action(&mut kobold, &pc.actions[0]);

        match result {
            ActionResult::Attack { hit, .. } => {
                match hit {
                    HitResult::Hit => assert!(false, "a normal hit is mathematically impossible!"),
                    HitResult::Miss => assert_eq!(kobold.current_hp, 5),
                    _ => {} // a crit is fine
                }
            }
            _ => assert!(false), // should never get here
        }
    }

    #[test]
    fn test_dice_rolling() {
        // Basic rolls
        let roll1 = roll_dice("1d6");
        assert!(roll1 >= 1 && roll1 <= 6);

        // Multiple dice
        let roll2 = roll_dice("2d20");
        assert!(roll2 >= 2 && roll2 <= 40);

        // With modifier
        let roll3 = roll_dice("1d6 + 4");
        assert!(roll3 >= 5 && roll3 <= 10);

        // Multiple dice types
        let roll4 = roll_dice("2d10 + 3d6");
        assert!(roll4 >= 5 && roll4 <= 38);

        // Negative modifier
        let roll5 = roll_dice("1d4 - 1");
        assert!(roll5 <= 3, "roll was {:?} which is not less than 3", roll5);
    }

    #[test]
    fn busted_ass_shit() {
        // Negative modifier
        let roll5 = roll_dice("1d4 - 1");
        assert!(
            roll5 >= 0 && roll5 <= 3,
            "roll was {:?} which is not less than 3",
            roll5
        );
    }

    #[test]
    fn test_action_validation() {
        // Set up a party: Fighter and Cleric vs two Kobolds
        let fighter =
            Character::new("Fighter", 10, 15, Team::Heroes).with_actions(vec![Action::Attack {
                name: "Shortsword".into(),
                hit_bonus: 4,
                damage: "1d6+2".into(),
            }]);

        let cleric = Character::new("Cleric", 8, 14, Team::Heroes).with_actions(vec![
            Action::Attack {
                name: "Mace".into(),
                hit_bonus: 2,
                damage: "1d6".into(),
            },
            Action::Heal {
                name: "Cure Wounds".into(),
                healing: "1d8+3".into(),
            },
        ]);

        let kobold1 = Character::new("Kobold 1", 5, 12, Team::Monsters);
        let kobold2 = Character::new("Kobold 2", 5, 12, Team::Monsters);

        let mut combat = Combat::new(vec![fighter, cleric], vec![kobold1, kobold2]);

        // Test 1: When everyone is at full HP
        {
            let cleric_index = 1; // Cleric is second hero
            let valid_actions = combat.valid_actions_for(Team::Heroes, cleric_index);
            assert_eq!(valid_actions.len(), 1); // Only attack should be valid, no need to heal
            assert!(matches!(valid_actions[0], Action::Attack { .. }));
        }

        // Test 2: When Fighter is damaged
        {
            let fighter_index = 0;
            combat.heroes[fighter_index].current_hp = 5; // Fighter took damage

            let cleric_index = 1;
            let valid_actions = combat.valid_actions_for(Team::Heroes, cleric_index);
            assert_eq!(valid_actions.len(), 2); // Both attack and heal should be valid
            assert!(valid_actions
                .iter()
                .any(|a| matches!(a, Action::Heal { .. })));
        }

        // Test 3: When all enemies are dead
        {
            combat.monsters[0].current_hp = 0;
            combat.monsters[1].current_hp = 0;

            let fighter_index = 0;
            let valid_actions = combat.valid_actions_for(Team::Heroes, fighter_index);
            assert!(valid_actions.is_empty()); // No valid attacks when enemies are dead
        }

        // Test 4: Cleric can still heal even when enemies are dead
        {
            let cleric_index = 1;
            let valid_actions = combat.valid_actions_for(Team::Heroes, cleric_index);
            assert_eq!(valid_actions.len(), 1);
            assert!(matches!(valid_actions[0], Action::Heal { .. }));
        }
    }
}

#[derive(Debug, Clone)]
pub enum AdvantageType {
    Advantage,
    Disadvantage,
    None,
}

#[cfg(test)]
mod dice_tests {
    use super::*;

    #[test]
    fn test_basic_dice_pool() {
        let pool = DicePool::new()
            .add_dice(1, 6) // 1d6
            .add_modifier(2); // +2

        let result = pool.roll();
        assert!(result >= 3 && result <= 8); // 1d6 + 2

        let pool = DicePool::from_str("1d4-1").unwrap();
        assert_eq!(pool.modifier, -1);
        assert_eq!(pool.to_string(), "1d4-1");
        let result = pool.roll();
        assert!(result >= 0 && result <= 3); // 1d4 - 1
    }

    #[test]
    fn test_dice_string_spacing() {
        // Test various space patterns in basic expressions
        let pool = DicePool::from_str("1d4-1").unwrap();
        assert_eq!(pool.to_string(), "1d4-1");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -1);

        let pool = DicePool::from_str("1d4 -1").unwrap();
        assert_eq!(pool.to_string(), "1d4-1");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -1);

        let pool = DicePool::from_str("1d4- 1").unwrap();
        assert_eq!(pool.to_string(), "1d4-1");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -1);

        let pool = DicePool::from_str("1d4 - 1").unwrap();
        assert_eq!(pool.to_string(), "1d4-1");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -1);

        // Test spacing in more complex expressions
        let pool = DicePool::from_str("2d6 + 1d4 - 3").unwrap();
        assert_eq!(pool.to_string(), "2d6+1d4-3");
        assert_eq!(pool.count_dice(6), 2);
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -3);

        // Test extreme spacing
        let pool = DicePool::from_str("   1d4    -    1   ").unwrap();
        assert_eq!(pool.to_string(), "1d4-1");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.modifier, -1);

        // Test no spaces
        let pool = DicePool::from_str("1d4+2d6-3+1").unwrap();
        assert_eq!(pool.to_string(), "2d6+1d4-2");
        assert_eq!(pool.count_dice(4), 1);
        assert_eq!(pool.count_dice(6), 2);
        assert_eq!(pool.modifier, -2);
    }

    #[test]
    fn test_multiple_dice_types() {
        let pool = DicePool::new()
            .add_dice(2, 6) // 2d6
            .add_dice(1, 4) // +1d4
            .add_modifier(3); // +3

        let result = pool.roll();
        assert!(result >= 6 && result <= 19); // (2d6 + 1d4 + 3)
    }

    #[test]
    fn add_the_same_type_of_dice() {
        let pool = DicePool::new().add_dice(2, 6).add_dice(1, 6);

        assert_eq!(pool.to_string(), "3d6");
        let result = pool.roll();
        assert!(result >= 3 && result <= 18); // (3d6)
    }

    #[test]
    fn test_critical_hit() {
        let base_pool = DicePool::new()
            .add_dice(2, 6) // 2d6
            .add_modifier(4); // +4

        // On a crit, we double the dice but not the modifier
        let crit_pool = base_pool.critical_hit();

        // Normal roll should be 2d6+4 (6-16)
        let normal_result = base_pool.roll();
        assert!(normal_result >= 6 && normal_result <= 16);

        // Crit roll should be 4d6+4 (8-28)
        let crit_result = crit_pool.roll();
        assert!(crit_result >= 8 && crit_result <= 28);
    }

    #[test]
    fn test_parse_dice_string() {
        let pool = DicePool::from_str("2d6+1d4+3").unwrap();

        assert_eq!(pool.count_dice(6), 2); // Should have 2 d6
        assert_eq!(pool.count_dice(4), 1); // Should have 1 d4
        assert_eq!(pool.modifier, 3); // Should have +3 modifier

        let pool = DicePool::from_str("2d6-1d4-3").unwrap();
        assert_eq!(pool.count_dice(6), 2); // Should have 2 d6
        assert_eq!(pool.count_dice(4), 1); // Should have 1 d4
        assert_eq!(pool.modifier, -3); // Should have +3 modifier
        assert_eq!(pool.to_string(), "2d6-1d4-3");

        let result = pool.roll();
        assert!(result >= -5 && result <= 8); // (2d6-1d4-3)
    }

    #[test]
    fn test_dice_string_output() {
        let pool = DicePool::new()
            .add_dice(2, 6)
            .add_dice(1, 4)
            .add_modifier(3);

        assert_eq!(pool.to_string(), "2d6+1d4+3");
    }

    #[test]
    fn test_advantage_roll_selection() {
        let pool = DicePool::new()
            .add_dice(1, 20)
            .add_modifier(-2) // Negative modifier
            .with_advantage();

        let result = pool.roll();
        let roll_result = pool.debug_last_roll().unwrap();

        // Even with a negative modifier, we should still pick the higher roll
        let selected_roll = roll_result.rolls.iter().map(|r| r.roll).max().unwrap();
        assert_eq!(result, selected_roll - 2);

        // Verify that the modifier wasn't applied before choosing the roll
        assert!(roll_result
            .rolls
            .iter()
            .all(|r| r.roll >= 1 && r.roll <= 20));
    }

    #[test]
    fn test_disadvantage_roll_selection() {
        let pool = DicePool::new()
            .add_dice(1, 20)
            .add_modifier(10) // High positive modifier
            .with_disadvantage();

        let result = pool.roll();
        let roll_result = pool.debug_last_roll().unwrap();

        // Even with a high modifier, we should still pick the lower roll
        let selected_roll = roll_result.rolls.iter().map(|r| r.roll).min().unwrap();
        assert_eq!(result, selected_roll + 10);

        assert!(roll_result
            .rolls
            .iter()
            .all(|r| r.roll >= 1 && r.roll <= 20));
    }
}
