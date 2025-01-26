use std::{collections::HashMap, hash::Hash};

use crate::{Action, ActionResult, DicePool, HitResult, Team};

use super::{
    conditions::{ConditionType, Effect},
    Condition,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub name: String,
    pub max_hp: usize,
    pub current_hp: usize,
    pub ac: usize,
    pub actions: Vec<Action>,
    pub team: Team,
    pub initiative_bonus: isize,

    resources: Resources,
    active_conditions: Vec<Condition>,
}

impl Character {
    pub fn new<T: Into<String>>(
        name: T,
        max_hp: usize,
        ac: usize,
        team: Team,
        initiative_bonus: isize,
    ) -> Self {
        Character {
            name: name.into(),
            max_hp,
            current_hp: max_hp,
            ac,
            actions: vec![],
            team,
            initiative_bonus,
            resources: Resources::new(),
            active_conditions: vec![],
        }
    }

    pub fn take_action(&mut self, target: &mut Character, action: &Action) -> ActionResult {
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
            Action::Heal {
                healing,
                required_resources,
                ..
            }
            | Action::SecondWind {
                healing,
                required_resources,
                ..
            } => {
                let resources_spent = required_resources.iter().all(|(resource, amount)| {
                    self.spend_resource(resource.clone(), *amount).is_ok()
                });

                let healing = if resources_spent {
                    healing.roll() as usize
                } else {
                    0
                };
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
        // If any conditions on the Actor prevent them from taking actions,
        // short-circuit action selection
        if self.active_conditions.iter().any(|con| {
            con.effects
                .iter()
                .any(|eff| eff == &Effect::CantTakeActions)
        }) {
            return vec![];
        }

        self.actions
            .iter()
            .filter(|a| a.is_valid(self, allies, enemies))
            .cloned()
            .collect()
    }

    pub fn roll_initiative(&self) -> isize {
        DicePool::d20().add_modifier(self.initiative_bonus).roll()
    }

    pub fn add_resource(&mut self, resource_type: ResourceType, max: usize) {
        self.resources.add_max(resource_type, max);
    }

    pub fn with_resources(mut self, resources: HashMap<ResourceType, usize>) -> Self {
        for (resource_type, amount) in resources {
            self.add_resource(resource_type, amount);
        }
        self
    }

    pub fn spend_resource(
        &mut self,
        resource_type: ResourceType,
        amount: usize,
    ) -> Result<(), String> {
        self.resources.spend(resource_type, amount)
    }

    pub fn has_resource(&self, resource_type: &ResourceType, amount: usize) -> bool {
        let resource = self.resources.get(resource_type);
        resource >= amount
    }

    pub fn add_condition(&mut self, condition: Condition) {
        self.active_conditions.push(condition);
    }

    pub fn has_condition(&self, condition: ConditionType) -> bool {
        self.active_conditions
            .iter()
            .any(|con| con.condition == condition)
    }

    pub fn end_turn(&mut self) {
        let surviving_conditions: Vec<Condition> = self
            .active_conditions
            .iter()
            .filter_map(|con| {
                if con.duration.remaining() > 1 {
                    Some(Condition::copy_deprecate_duration(con))
                } else {
                    None
                }
            })
            .collect();
        self.active_conditions = surviving_conditions;
    }

    pub fn has_disadvantage_on(&self, action: &Action) -> bool {
        self.active_conditions.iter().any(|condition| match action {
            Action::Attack { .. } => condition
                .effects
                .iter()
                .any(|effect| effect == &Effect::DisadvantageOnAttacks),
            _ => false,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InitiativeEntry {
    pub team: Team,
    pub index: usize,
    pub initiative: isize,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum ResourceType {
    SpellSlot(usize),
    Feature(String),
    Points(String),
}

#[derive(Clone, Debug, PartialEq)]
struct Resources {
    resources: HashMap<ResourceType, usize>,
    max_resources: HashMap<ResourceType, usize>,
}

impl Resources {
    fn new() -> Self {
        Resources {
            resources: HashMap::new(),
            max_resources: HashMap::new(),
        }
    }

    fn add_max(&mut self, resource_type: ResourceType, amount: usize) {
        self.max_resources
            .entry(resource_type.clone())
            .or_insert(amount);
        self.resources.entry(resource_type).or_insert(amount);
    }

    fn get(&self, resource_type: &ResourceType) -> usize {
        match self.resources.get(resource_type) {
            Some(resource) => *resource,
            None => 0,
        }
    }

    fn spend(&mut self, resource_type: ResourceType, amount: usize) -> Result<(), String> {
        let r = self.resources.get_mut(&resource_type);
        match r {
            Some(r) => {
                if *r >= amount {
                    *r -= amount;
                    Ok(())
                } else {
                    Err(format!(
                        "Resource {:?} was had {} and attempted to spend {}",
                        resource_type, r, amount
                    ))
                }
            }
            None => Err(format!("Actor doesn't have resource {:?}", resource_type)),
        }
    }

    fn _recover(&mut self, resource_type: ResourceType, amount: usize) {
        let current = self.resources.get_mut(&resource_type);
        let max = self.max_resources.get(&resource_type).unwrap();

        match current {
            Some(r) => {
                if *r + amount > *max {
                    *r = *max;
                } else {
                    *r += amount;
                }
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;

    #[test]
    fn test_spell_slot_spending() {
        let mut resources = Resources::new();

        // Level 1 slots
        resources.add_max(ResourceType::SpellSlot(1), 4);
        assert_eq!(resources.get(&ResourceType::SpellSlot(1)), 4);

        resources.spend(ResourceType::SpellSlot(1), 1).unwrap();
        assert_eq!(resources.get(&ResourceType::SpellSlot(1)), 3);

        // Can't overspend
        assert!(resources.spend(ResourceType::SpellSlot(1), 4).is_err());
    }

    #[test]
    fn test_feature_usage() {
        let mut resources = Resources::new();
        resources.add_max(ResourceType::Feature("Second Wind".into()), 1);

        resources
            .spend(ResourceType::Feature("Second Wind".into()), 1)
            .unwrap();
        assert_eq!(
            resources.get(&ResourceType::Feature("Second Wind".into())),
            0
        );

        // Can't use when depleted
        assert!(resources
            .spend(ResourceType::Feature("Second Wind".into()), 1)
            .is_err());
    }

    #[test]
    fn test_point_pools() {
        let mut resources = Resources::new();
        resources.add_max(ResourceType::Points("Ki".into()), 5);

        // Partial spending
        resources
            .spend(ResourceType::Points("Ki".into()), 2)
            .unwrap();
        assert_eq!(resources.get(&ResourceType::Points("Ki".into())), 3);

        // Recovery up to max
        resources._recover(ResourceType::Points("Ki".into()), 1);
        assert_eq!(resources.get(&ResourceType::Points("Ki".into())), 4);

        // Can't recover beyond max
        resources._recover(ResourceType::Points("Ki".into()), 2);
        assert_eq!(resources.get(&ResourceType::Points("Ki".into())), 5);
    }

    //   #[test]
    //    fn test_rests() {
    //        let mut resources = Resources::new();
    //        resources.add_max(ResourceType::SpellSlot(1), 4);
    //        resources.add_max(ResourceType::Feature("Second Wind".into()), 1);
    //        resources.add_max(ResourceType::Points("Ki".into()), 5);

    //        // Use some resources
    //        resources.spend(ResourceType::SpellSlot(1), 2).unwrap();
    //        resources.spend(ResourceType::Feature("Second Wind".into()), 1).unwrap();
    //        resources.spend(ResourceType::Points("Ki".into()), 3).unwrap();

    //        // Short rest recovers some
    //        resources.short_rest();
    //        assert_eq!(resources.get(ResourceType::SpellSlot(1)), 2); // Unchanged
    //        assert_eq!(resources.get(ResourceType::Feature("Second Wind".into())), 1); // Recovered
    //        assert_eq!(resources.get(ResourceType::Points("Ki".into())), 5); // Recovered

    //        // Long rest recovers all
    //        resources.spend(ResourceType::SpellSlot(1), 2).unwrap();
    //        resources.long_rest();
    //        assert_eq!(resources.get(ResourceType::SpellSlot(1)), 4);
    //        assert_eq!(resources.get(ResourceType::Feature("Second Wind".into())), 1);
    //        assert_eq!(resources.get(ResourceType::Points("Ki".into())), 5);
    //    }
}
