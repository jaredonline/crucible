use crate::DicePool;

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
