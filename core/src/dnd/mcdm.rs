use phf::phf_map;

use super::DifficultyCalculatorImpl;

static EASY_CR_PER_LEVEL: phf::Map<&'static str, f64> = phf_map! {
    "1" => 0.125,
    "2" => 0.125,
    "3" => 0.25,
    "4" => 0.5,
    "5" => 1.0,
    "6" => 1.5,
    "7" => 2.0,
    "8" => 2.5,
    "9" => 3.0,
    "10" => 3.5,
    "11" => 4.0,
    "12" => 4.5,
    "13" => 5.0,
    "14" => 5.5,
    "15" => 6.0,
    "16" => 6.5,
    "17" => 7.0,
    "18" => 7.5,
    "19" => 8.0,
    "20" => 8.5,
};

static STANDARD_CR_PER_LEVEL: phf::Map<&'static str, f64> = phf_map! {
    "1" => 0.125,
    "2" => 0.25,
    "3" => 0.5,
    "4" => 0.75,
    "5" => 1.5,
    "6" => 2.0,
    "7" => 2.5,
    "8" => 3.0,
    "9" => 3.5,
    "10" => 4.0,
    "11" => 4.5,
    "12" => 5.0,
    "13" => 5.5,
    "14" => 6.0,
    "15" => 6.5,
    "16" => 7.0,
    "17" => 7.5,
    "18" => 8.0,
    "19" => 8.5,
    "20" => 9.0,
};

static HARD_CR_PER_LEVEL: phf::Map<&'static str, f64> = phf_map! {
    "1" => 0.25,
    "2" => 0.5,
    "3" => 0.75,
    "4" => 1.0,
    "5" => 2.5,
    "6" => 3.0,
    "7" => 3.5,
    "8" => 4.0,
    "9" => 4.5,
    "10" => 5.0,
    "11" => 5.5,
    "12" => 6.0,
    "13" => 6.5,
    "14" => 7.0,
    "15" => 7.5,
    "16" => 8.0,
    "17" => 8.5,
    "18" => 9.0,
    "19" => 9.5,
    "20" => 10.0,
};

static CR_CAP_PER_LEVEL: phf::Map<&'static str, usize> = phf_map! {
    "1" => 1,
    "2" => 3,
    "3" => 4,
    "4" => 6,
    "5" => 8,
    "6" => 9,
    "7" => 10,
    "8" => 12,
    "9" => 13,
    "10" => 15,
    "11" => 16,
    "12" => 17,
    "13" => 19,
    "14" => 20,
    "15" => 22,
    "16" => 24,
    "17" => 25,
    "18" => 26,
    "19" => 28,
    "20" => 30,
};

pub enum MCDMDifficultyScale {
    Trivial,
    Easy,
    Standard,
    Hard,
    Extreme,
}

impl From<MCDMDifficultyScale> for String {
    fn from(value: MCDMDifficultyScale) -> Self {
        match value {
            MCDMDifficultyScale::Trivial => "Trivial".into(),
            MCDMDifficultyScale::Easy => "Easy".into(),
            MCDMDifficultyScale::Standard => "Standard".into(),
            MCDMDifficultyScale::Hard => "Hard".into(),
            MCDMDifficultyScale::Extreme => "Extreme".into(),
        }
    }
}

pub struct MCDMDifficultyCalculator;

impl DifficultyCalculatorImpl for MCDMDifficultyCalculator {
    type DifficultyResult = MCDMDifficultyScale;

    fn calculate(pc_levels: &Vec<usize>, monster_crs: &Vec<f64>) -> Self::DifficultyResult {
        let total_cr: f64 = monster_crs.iter().sum();
        let mut easy_budget = 0.0;
        let mut hard_budget = 0.0;
        let mut standard_budget = 0.0;

        for i in pc_levels {
            let i_str = i.to_string();
            easy_budget += EASY_CR_PER_LEVEL.get(&i_str.as_str()).unwrap();
            standard_budget += STANDARD_CR_PER_LEVEL.get(&i_str.as_str()).unwrap();
            hard_budget += HARD_CR_PER_LEVEL.get(&i_str.as_str()).unwrap();
        }

        // under easy is trivial
        // over hard is extreme
        // between easy and standard

        if total_cr < easy_budget {
            MCDMDifficultyScale::Trivial
        } else if total_cr < standard_budget {
            MCDMDifficultyScale::Easy
        } else if total_cr < hard_budget {
            MCDMDifficultyScale::Standard
        } else if total_cr == hard_budget {
            MCDMDifficultyScale::Hard
        } else {
            MCDMDifficultyScale::Extreme
        }
    }
}
