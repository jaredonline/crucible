use super::{Action, ActionResult, Character};

#[derive(Debug)]
pub struct ActivityLog {
    pub round: usize,
    pub action: Action,
    pub actor: Character,
    pub target: Character,
    pub result: ActionResult,
    pub snapshot_heroes: Vec<Character>,
    pub snapshot_monsters: Vec<Character>,
}
