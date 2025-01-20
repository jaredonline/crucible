use std::marker::PhantomData;

pub mod mcdm;
pub mod wizards;

pub trait DifficultyCalculatorImpl {
    type DifficultyResult: Into<String>;

    fn calculate(pc_levels: &Vec<usize>, monster_crs: &Vec<f64>) -> Self::DifficultyResult;
}

pub struct DifficultyCalculator<T: DifficultyCalculatorImpl> {
    pc_levels: Vec<usize>,
    monster_crs: Vec<f64>,
    _phantom: PhantomData<T>,
}

impl<T: DifficultyCalculatorImpl> DifficultyCalculator<T> {
    pub fn new(pc_levels: Vec<usize>, monster_crs: Vec<f64>) -> DifficultyCalculator<T> {
        DifficultyCalculator {
            pc_levels,
            monster_crs,
            _phantom: PhantomData,
        }
    }

    pub fn calculate(&self) -> T::DifficultyResult {
        T::calculate(&self.pc_levels, &self.monster_crs)
    }
}
