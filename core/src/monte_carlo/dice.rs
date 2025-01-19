use crate::DicePool;

#[derive(Clone)]
pub struct DieStats {
    pub sides: usize,
    pub counts: Vec<usize>,
    pub mean: f64,
    pub expected_mean: f64,
    pub chi_square: f64,
    pub distribution: String,
}

pub fn run_dice_monte_carlo(iterations: usize) -> Vec<DieStats> {
    let die_types = vec![4, 6, 8, 10, 12, 20];
    let mut results = vec![];

    for sides in die_types {
        let mut counts = vec![0; sides];
        let mut sum = 0;
        for _ in 0..iterations {
            let roll = DicePool::new().add_dice(1, sides).roll();
            counts[(roll - 1) as usize] += 1;
            sum += roll;
        }

        let mean = sum as f64 / iterations as f64;
        let expected_mean = (sides + 1) as f64 / 2.0;

        // Chi-square test
        let expected = iterations as f64 / sides as f64;
        let chi_square: f64 = counts
            .iter()
            .map(|&count| {
                let diff = count as f64 - expected;
                (diff * diff) / expected
            })
            .sum();

        // Distribution visualization
        let distribution = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let percentage = (count as f64 / iterations as f64 * 100.0).round();
                format!("{}:{:.1}%", i + 1, percentage)
            })
            .collect::<Vec<_>>()
            .join("\n");

        results.push(DieStats {
            sides: sides,
            counts: counts,
            mean,
            expected_mean,
            chi_square,
            distribution,
        });
    }

    results
}

#[derive(Clone)]
pub struct DiceMonteCarloCollection {
    sides: Vec<usize>,
    iterations: usize,
}

#[derive(Clone)]
pub struct DiceMonteCarloIterator {
    collection: DiceMonteCarloCollection,
    sides_index: usize,
    iteration_index: usize,

    current_counts: Vec<usize>,
    current_sum: isize,

    pub results: Vec<DieStats>,
}

impl DiceMonteCarloIterator {
    fn not_done(&self) -> bool {
        self.sides_index < self.collection.sides.len()
            && self.iteration_index < self.collection.iterations
    }
}

impl Iterator for DiceMonteCarloIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.not_done() {
            let sides = self.collection.sides[self.sides_index];

            if self.iteration_index == 0 {
                // reset counters
                self.current_counts = vec![0; sides];
                self.current_sum = 0;
            }

            let roll = DicePool::new().add_dice(1, sides).roll();
            self.current_counts[(roll - 1) as usize] += 1;
            self.current_sum += roll;

            let ret = Some((self.sides_index, self.iteration_index));

            self.iteration_index += 1;
            if self.iteration_index >= self.collection.iterations {
                // we've finished with this particular die
                self.iteration_index = 0;
                self.sides_index += 1;

                // add all the shit to our results
                let mean = self.current_sum as f64 / self.collection.iterations as f64;
                let expected_mean = (sides + 1) as f64 / 2.0;

                // Chi-square test
                let expected = self.collection.iterations as f64 / sides as f64;
                let chi_square: f64 = self
                    .current_counts
                    .iter()
                    .map(|&count| {
                        let diff = count as f64 - expected;
                        (diff * diff) / expected
                    })
                    .sum();

                // Distribution visualization
                let distribution = self
                    .current_counts
                    .iter()
                    .enumerate()
                    .map(|(i, &count)| {
                        let percentage =
                            (count as f64 / self.collection.iterations as f64 * 100.0).round();
                        format!("{}:{:.1}%", i + 1, percentage)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                self.results.push(DieStats {
                    sides: sides,
                    counts: self.current_counts.clone(),
                    mean,
                    expected_mean,
                    chi_square,
                    distribution,
                });
            }

            return ret;
        }

        None
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.collection.sides.len() * self.collection.iterations
    }
}

pub fn dice_monte_carlo_iterator(iterations: usize) -> DiceMonteCarloIterator {
    let collection = DiceMonteCarloCollection {
        sides: vec![4, 6, 8, 10, 12, 20],
        iterations,
    };

    DiceMonteCarloIterator {
        collection,
        sides_index: 0,
        iteration_index: 0,

        current_counts: vec![],
        current_sum: 0,

        results: vec![],
    }
}
