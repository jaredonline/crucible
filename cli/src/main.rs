use std::collections::HashMap;
use std::fmt::Write;
use std::vec;

use anyhow::Result;
use clap::Parser;
use comfy_table::Table;
use crucible_core::combat::build_level_one_combat;
use crucible_core::dnd::mcdm::MCDMDifficultyCalculator;
use crucible_core::dnd::wizards::{WizardDifficultyCalculator2014, WizardDifficultyCalculator2024};
use crucible_core::dnd::DifficultyCalculator;
use crucible_core::monte_carlo::combat::combat_monte_carlo_iterator;
use crucible_core::monte_carlo::dice::dice_monte_carlo_iterator;
use crucible_core::{Action, ActionResult, Character, HitResult};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

#[derive(Parser)]
enum SubCommand {
    #[command(name = "dice-monte-carlo")]
    DiceMonteCarlo(DiceMonteCarloArgs),

    #[command(name = "level-1-kobolds")]
    LevelOneKobolds(LevelOneKoboldsArgs),

    #[command(name = "level-1-monte-carlo")]
    LevelOneMonteCarlo(LevelOneMonteCarloArgs),
}

#[derive(Parser)]
#[command(name = "dice-monte-carlo")]
pub struct DiceMonteCarloArgs {
    /// Number of iterations for each die type
    #[arg(short, long, default_value = "100000")]
    iterations: usize,
}

#[derive(Parser)]
#[command(name = "dice-monte-carlo")]
pub struct LevelOneKoboldsArgs {
    /// Number of iterations for each die type
    #[arg(short, long, default_value = "6")]
    num_kobolds: usize,
}

#[derive(Parser)]
#[command(name = "level-1-monte-carlo")]
pub struct LevelOneMonteCarloArgs {
    /// Number of iterations for combat
    #[arg(short, long, default_value = "10000")]
    iterations: usize,

    /// Turn on/off verbose logging
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Number of iterations for each die type
    #[arg(short, long, default_value = "6")]
    num_kobolds: usize,
}

#[derive(Parser)]
#[command(
    name = "cru",
    author,
    version,
    about = "Crucible CLI",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
    // /// Optional name to operate on
    // #[arg(short, long)]
    // name: Option<String>,

    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // // Use cli arguments and core functionality
    // if let Some(name) = cli.name.as_deref() {
    //     println!("Name: {name}");
    // }

    // if let Some(config_path) = cli.config.as_deref() {
    //     println!("Config: {}", config_path.display());
    // }

    match cli.command {
        SubCommand::DiceMonteCarlo(args) => dice_monte_carlo(args),
        SubCommand::LevelOneKobolds(args) => level_one_kobolds(args),
        SubCommand::LevelOneMonteCarlo(args) => level_one_monte_carlo(args),
    }
}

fn level_one_kobolds(args: LevelOneKoboldsArgs) -> Result<()> {
    let mut combat = build_level_one_combat(args.num_kobolds);
    combat.debug(true);
    let mut table = Table::new();
    table.set_header(vec![
        "Round", "Actor", "Action", "Target", "Result", "All Hps", "All ACs",
    ]);

    combat.roll_initiative();

    while combat.is_ongoing() {
        combat.execute_round();
    }

    for log in combat.debug_log {
        table.add_row(vec![
            format!("{}", log.round),
            actor_name(log.actor),
            action_name(log.action),
            actor_name(log.target),
            result_debug(log.result),
            all_hps(&log.snapshot_heroes, &log.snapshot_monsters),
            all_acs(&log.snapshot_heroes, &log.snapshot_monsters),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn dice_monte_carlo(args: DiceMonteCarloArgs) -> Result<()> {
    let mut table = Table::new();
    table.set_header(vec![
        "Die",
        "Mean",
        "Expected Mean",
        "Chi-Square",
        "Distribution",
    ]);

    let mut iterator = dice_monte_carlo_iterator(args.iterations);
    let bar = ProgressBar::new(iterator.clone().count() as u64);

    while let Some(_) = iterator.next() {
        bar.inc(1);
    }
    bar.finish_and_clear();

    let stats = iterator.results;

    for stat in stats {
        table.add_row(vec![
            format!("d{}", stat.sides),
            format!("{:.3}", stat.mean),
            format!("{:.3}", stat.expected_mean),
            format!("{:.3}", stat.chi_square),
            stat.distribution,
        ]);
    }

    println!("{table}");

    Ok(())
}

fn level_one_monte_carlo(args: LevelOneMonteCarloArgs) -> Result<()> {
    let mut table = Table::new();
    table.set_header(vec![
        "Player Victories",
        "Monster Victories",
        "Average Rounds",
        "Hero K/O Counts",
        "Monster K/O Counts",
        "Decisive Victories",
        //"Pyrrhic Victories",
        "MCDM Difficulty",
        "Wizards Difficulty",
    ]);

    let mut iterator = combat_monte_carlo_iterator(args.iterations, args.verbose, args.num_kobolds);
    let bar = ProgressBar::new(args.iterations as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    while let Some(_) = iterator.next() {
        bar.inc(1);
    }
    bar.finish_and_clear();

    let combat = build_level_one_combat(args.num_kobolds);
    let mcdm: DifficultyCalculator<MCDMDifficultyCalculator> =
        DifficultyCalculator::new(combat.hero_levels(), combat.monster_crs());
    let wizards2014: DifficultyCalculator<WizardDifficultyCalculator2014> =
        DifficultyCalculator::new(combat.hero_levels(), combat.monster_crs());
    let wizards2024: DifficultyCalculator<WizardDifficultyCalculator2024> =
        DifficultyCalculator::new(combat.hero_levels(), combat.monster_crs());

    let stats = iterator.stats;
    table.add_row(vec![
        format!(
            "{} ({:.3}%)",
            stats.hero_victories,
            stats.hero_victories_perc * 100.0
        ),
        format!(
            "{} ({:.3}%)",
            stats.monster_victories,
            stats.monster_victories_perc * 100.0
        ),
        format!("{:.3}", stats.average_rounds),
        actor_ko_counts_formatted(&stats.hero_ko_counts),
        actor_ko_counts_formatted(&stats.monster_ko_counts),
        format!(
            "{} ({:.3}%)",
            stats.decisive_victories,
            stats.decisive_victories_perc * 100.0
        ),
        format!("{}", String::from(mcdm.calculate())),
        format!(
            "2014: {}\n2024: {}",
            String::from(wizards2014.calculate()),
            String::from(wizards2024.calculate())
        ),
    ]);

    println!("{table}");

    Ok(())
}

fn actor_ko_counts_formatted(counts: &HashMap<String, usize>) -> String {
    let mut parts = vec![];
    for (name, kos) in counts {
        parts.push(format!("{}: {}", name, kos));
    }
    parts.join("\n")
}

fn action_name(action: Action) -> String {
    match action {
        Action::Attack { name, .. } => format!("Attack with {}", name).into(),
        Action::Heal { name, .. } => format!("Heal with {}", name).into(),
    }
}

fn actor_name(actor: Character) -> String {
    actor.name
}

fn result_debug(result: ActionResult) -> String {
    match result {
        ActionResult::Attack { hit, damage } => match hit {
            HitResult::Critical => format!("Critical for {}", damage),
            HitResult::Hit => format!("Hit for {}", damage),
            HitResult::Miss => format!("Miss"),
        },
        ActionResult::Heal { amount } => format!("Healed for {}", amount),
        ActionResult::None => "No result".into(),
    }
}

fn all_hps(heroes: &Vec<Character>, villains: &Vec<Character>) -> String {
    let mut result = vec![];
    for hero in heroes {
        result.push(format!(
            "{}: {}/{}",
            hero.name, hero.current_hp, hero.max_hp
        ));
    }
    for villain in villains {
        result.push(format!(
            "{}: {}/{}",
            villain.name, villain.current_hp, villain.max_hp
        ));
    }
    result.join("\n")
}

fn all_acs(heroes: &Vec<Character>, villains: &Vec<Character>) -> String {
    let mut result = vec![];
    for hero in heroes {
        result.push(format!("{}: {}", hero.name, hero.ac));
    }
    for villain in villains {
        result.push(format!("{}: {}", villain.name, villain.ac));
    }
    result.join("\n")
}
