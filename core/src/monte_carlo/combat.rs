use anyhow::Result;
use std::collections::HashMap;

use crate::{combat::build_level_one_combat, Combat};

pub struct CombatStats {
    pub hero_victories: usize,
    pub hero_victories_perc: f64,
    pub monster_victories: usize,
    pub monster_victories_perc: f64,
    pub average_rounds: f64,
    pub hero_ko_counts: HashMap<String, usize>, // How often each hero went down
    pub monster_ko_counts: HashMap<String, usize>,
    pub decisive_victories: usize, // All enemies dead, all heroes alive
    pub pyrrhic_victories: usize,  // Heroes win but most are down

    round_counts: Vec<usize>,
}

impl Default for CombatStats {
    fn default() -> Self {
        CombatStats {
            hero_victories: 0,
            hero_victories_perc: 0.0,
            monster_victories: 0,
            monster_victories_perc: 0.0,
            average_rounds: 0.0,
            hero_ko_counts: HashMap::new(),
            monster_ko_counts: HashMap::new(),
            decisive_victories: 0,
            pyrrhic_victories: 0,

            round_counts: vec![],
        }
    }
}

impl CombatStats {
    pub fn new() -> Self {
        CombatStats::default()
    }

    fn finalize(&mut self) {
        let len = self.round_counts.len() as f64;
        let average_rounds: f64 = self
            .round_counts
            .clone()
            .into_iter()
            .map(|i| i as f64)
            .sum();

        self.average_rounds = average_rounds / len;

        self.hero_victories_perc = self.hero_victories as f64 / len;
        self.monster_victories_perc = self.monster_victories as f64 / len;
    }
}

pub fn run_combat_monte_carlo(iterations: usize, verbose: bool, num_kobolds: usize) -> CombatStats {
    let mut stats = CombatStats::new();

    for i in 0..iterations {
        // Set up fresh combat with same initial state
        let mut combat = build_level_one_combat(num_kobolds);
        combat.debug(verbose);

        // Run until completion
        while combat.is_ongoing() {
            combat.execute_round();
        }

        // Collect stats
        //update_stats(&mut stats, &combat);
    }

    // Print results
    //print_monte_carlo_results(&stats, args.iterations);
    stats
}

pub fn combat_monte_carlo_iterator(
    iterations: usize,
    verbose: bool,
    num_kobolds: usize,
) -> CombatMonteCarloIterator {
    let collection = CombatMonteCarloCollection {
        iterations,
        num_kobolds,
    };

    CombatMonteCarloIterator {
        collection,
        index: 0,
        stats: CombatStats::default(),
    }
}

pub struct CombatMonteCarloCollection {
    iterations: usize,
    num_kobolds: usize,
}

pub struct CombatMonteCarloIterator {
    collection: CombatMonteCarloCollection,
    index: usize,

    pub stats: CombatStats,
}

impl CombatMonteCarloIterator {
    fn update_stats(&mut self, combat: &Combat) {
        if combat.heroes_won() {
            self.stats.hero_victories += 1;
        } else {
            self.stats.monster_victories += 1;
        }

        for hero in &combat.heroes {
            if hero.current_hp == 0 {
                let stat = self
                    .stats
                    .hero_ko_counts
                    .entry(hero.name.clone())
                    .or_insert(0);
                *stat += 1;
            }
        }

        for monster in &combat.monsters {
            if monster.current_hp == 0 {
                let stat = self
                    .stats
                    .monster_ko_counts
                    .entry(monster.name.clone())
                    .or_insert(0);
                *stat += 1;
            }
        }

        self.stats.round_counts.push(combat.round);

        if combat.heroes.iter().all(|c| c.current_hp > 0) {
            self.stats.decisive_victories += 1;
        }
    }

    fn finalize_stats(&mut self) {
        self.stats.finalize();
    }
}

impl Iterator for CombatMonteCarloIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.collection.iterations {
            let mut combat = build_level_one_combat(self.collection.num_kobolds);
            combat.roll_initiative();

            while combat.is_ongoing() {
                combat.execute_round();
            }

            self.update_stats(&combat);
            let ret = Some(self.index);
            self.index += 1;

            return ret;
        }

        self.finalize_stats();
        None
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.collection.iterations
    }
}
