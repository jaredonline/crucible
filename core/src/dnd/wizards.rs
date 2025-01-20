use std::usize;

use phf::phf_map;

use super::DifficultyCalculatorImpl;

static MONSTER_CR_TO_XP_2014: phf::Map<&'static str, usize> = phf_map! {
    "0.125" => 25,
    "0.25" => 50,
    "0.5" => 100,
    "1" => 200,
    "2" => 450,
    "3" => 700,
    "4" => 1100,
    "5" => 1800,
    "6" => 2300,
    "7" => 2900,
    "8" => 3900,
    "9" => 5000,
    "10" => 5900,
    "11" => 7200,
    "12" => 8400,
    "13" => 10000,
    "14" => 11500,
    "15" => 13000,
    "16" => 15000,
    "17" => 18000,
    "18" => 20000,
    "19" => 22000,
    "20" => 25000,
    "21" => 33000,
    "22" => 41000,
    "23" => 50000,
    "24" => 62000,
    "25" => 75000,
    "26" => 90000,
    "27" => 105000,
    "28" => 120000,
    "29" => 135000,
    "30" => 155000
};

static EASY_XP_PER_LEVEL_2014: phf::Map<&'static str, usize> = phf_map! {
    "1" => 25,
    "2" => 50,
    "3" => 75,
    "4" => 125,
    "5" => 250,
    "6" => 300,
    "7" => 350,
    "8" => 450,
    "9" => 550,
    "10" => 600,
    "11" => 800,
    "12" => 1000,
    "13" => 1100,
    "14" => 1250,
    "15" => 1400,
    "16" => 1600,
    "17" => 2000,
    "18" => 2100,
    "19" => 2400,
    "20" => 2800,
};

static MEDIUM_XP_PER_LEVEL_2014: phf::Map<&'static str, usize> = phf_map! {
    "1" => 50,
    "2" => 100,
    "3" => 150,
    "4" => 250,
    "5" => 500,
    "6" => 600,
    "7" => 750,
    "8" => 900,
    "9" => 1100,
    "10" => 1200,
    "11" => 1600,
    "12" => 2000,
    "13" => 2200,
    "14" => 2500,
    "15" => 2800,
    "16" => 3200,
    "17" => 3900,
    "18" => 4200,
    "19" => 4900,
    "20" => 5700,
};

static HARD_XP_PER_LEVEL_2014: phf::Map<&'static str, usize> = phf_map! {
    "1" => 75,
    "2" => 150,
    "3" => 225,
    "4" => 375,
    "5" => 750,
    "6" => 900,
    "7" => 1100,
    "8" => 1400,
    "9" => 1600,
    "10" => 1900,
    "11" => 2400,
    "12" => 3000,
    "13" => 3400,
    "14" => 3800,
    "15" => 4300,
    "16" => 4800,
    "17" => 5900,
    "18" => 6300,
    "19" => 7300,
    "20" => 8500,
};

static DEADLY_XP_PER_LEVEL_2014: phf::Map<&'static str, usize> = phf_map! {
    "1" => 100,
    "2" => 200,
    "3" => 400,
    "4" => 500,
    "5" => 1100,
    "6" => 1400,
    "7" => 1700,
    "8" => 2100,
    "9" => 2400,
    "10" => 2800,
    "11" => 3600,
    "12" => 4500,
    "13" => 5100,
    "14" => 5700,
    "15" => 6400,
    "16" => 7200,
    "17" => 8800,
    "18" => 9500,
    "19" => 10900,
    "20" => 12700,
};

static XP_PER_LEVEL_2024: phf::Map<&'static str, (usize, usize, usize)> = phf_map! {
    "1" => (50, 75, 100),
    "2" => (100, 150, 200),
    "3" => (150, 225, 400),
    "4" => (250, 375, 500),
    "5" => (500, 750, 1100),
    "6" => (600, 1000, 1400),
    "7" => (750, 1300, 1700),
    "8" => (1000, 1700, 2100),
    "9" => (1300, 2000, 2600),
    "10" => (1600, 2300, 3100),
    "11" => (1900, 2900, 4100),
    "12" => (2200, 3700, 4700),
    "13" => (2600, 4200, 5400),
    "14" => (2900, 4900, 6200),
    "15" => (3300, 5400, 7800),
    "16" => (3800, 6100, 9800),
    "17" => (4500, 7200, 11700),
    "18" => (5000, 8700, 14200),
    "19" => (5500, 10700, 17200),
    "20" => (6400, 13200, 22000),
};

pub enum WizardDifficulty2014 {
    Easy,
    Medium,
    Hard,
    Deadly,
}

impl From<WizardDifficulty2014> for String {
    fn from(value: WizardDifficulty2014) -> Self {
        match value {
            WizardDifficulty2014::Easy => "Easy".into(),
            WizardDifficulty2014::Medium => "Medium".into(),
            WizardDifficulty2014::Hard => "Hard".into(),
            WizardDifficulty2014::Deadly => "Deadly".into(),
        }
    }
}

pub struct WizardDifficultyCalculator2014;

impl DifficultyCalculatorImpl for WizardDifficultyCalculator2014 {
    type DifficultyResult = WizardDifficulty2014;

    fn calculate(pc_levels: &Vec<usize>, monster_crs: &Vec<f64>) -> Self::DifficultyResult {
        let mut total_monster_xp: usize = monster_crs
            .iter()
            .map(|cr| MONSTER_CR_TO_XP_2014.get(cr.to_string().as_str()).unwrap())
            .sum();
        let mut easy_threshold = 0;
        let mut medium_threshold = 0;
        let mut hard_threshold = 0;
        let mut deadly_threshold = 0;

        for i in pc_levels {
            let i_str = i.to_string();
            easy_threshold += EASY_XP_PER_LEVEL_2014.get(&i_str.as_str()).unwrap();
            medium_threshold += MEDIUM_XP_PER_LEVEL_2014.get(&i_str.as_str()).unwrap();
            hard_threshold += HARD_XP_PER_LEVEL_2014.get(&i_str.as_str()).unwrap();
            deadly_threshold += DEADLY_XP_PER_LEVEL_2014.get(&i_str.as_str()).unwrap();
        }

        match monster_crs.len() {
            0..=1 => {}
            2 => total_monster_xp = (total_monster_xp as f64 * 1.5) as usize,
            3..=6 => total_monster_xp = total_monster_xp * 2,
            7..=10 => total_monster_xp = (total_monster_xp as f64 * 2.5) as usize,
            11..=14 => total_monster_xp = total_monster_xp * 3,
            15.. => total_monster_xp = total_monster_xp * 4,
        }

        if total_monster_xp >= deadly_threshold {
            WizardDifficulty2014::Deadly
        } else if total_monster_xp >= hard_threshold {
            WizardDifficulty2014::Hard
        } else if total_monster_xp >= medium_threshold {
            WizardDifficulty2014::Medium
        } else {
            WizardDifficulty2014::Easy
        }
    }
}

pub enum WizardDifficulty2024 {
    Low,
    Moderate,
    High,
}

impl From<WizardDifficulty2024> for String {
    fn from(value: WizardDifficulty2024) -> Self {
        match value {
            WizardDifficulty2024::Low => "Low".into(),
            WizardDifficulty2024::Moderate => "Moderate".into(),
            WizardDifficulty2024::High => "High".into(),
        }
    }
}

pub struct WizardDifficultyCalculator2024;

impl DifficultyCalculatorImpl for WizardDifficultyCalculator2024 {
    type DifficultyResult = WizardDifficulty2024;

    fn calculate(pc_levels: &Vec<usize>, monster_crs: &Vec<f64>) -> Self::DifficultyResult {
        let total_monster_xp: usize = monster_crs
            .iter()
            .map(|cr| MONSTER_CR_TO_XP_2014.get(cr.to_string().as_str()).unwrap())
            .sum();
        let mut low_threshold = 0;
        let mut moderate_threshold = 0;
        let mut _high_threshold = 0;

        for i in pc_levels {
            let xp = XP_PER_LEVEL_2024.get(i.to_string().as_str()).unwrap();
            low_threshold += xp.0;
            moderate_threshold += xp.1;
            _high_threshold += xp.2;
        }

        if total_monster_xp <= low_threshold {
            WizardDifficulty2024::Low
        } else if total_monster_xp <= moderate_threshold {
            WizardDifficulty2024::Moderate
        } else {
            WizardDifficulty2024::High
        }
    }
}
