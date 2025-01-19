use std::vec;

use anyhow::Result;
use clap::Parser;
use comfy_table::Table;
use crucible_core::monte_carlo::dice::run_dice_monte_carlo;
use crucible_core::{Action, ActionResult, Character, Combat, HitResult, Team};

#[derive(Parser)]
enum SubCommand {
    #[command(name = "dice-monte-carlo")]
    DiceMonteCarlo(DiceMonteCarloArgs),

    #[command(name = "level-1-kobolds")]
    LevelOneKobolds(LevelOneKoboldsArgs),
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
    }
}

fn level_one_kobolds(args: LevelOneKoboldsArgs) -> Result<()> {
    let fighter =
        Character::new("Fighter", 12, 16, Team::Heroes).with_actions(vec![Action::Attack {
            name: "Greatsword".into(),
            hit_bonus: 5,
            damage: "2d6+3".into(),
        }]);
    let cleric = Character::new("Cleric", 10, 16, Team::Heroes).with_actions(vec![
        Action::Attack {
            name: "Mace".into(),
            hit_bonus: 4,
            damage: "1d6+2".into(),
        },
        Action::Heal {
            name: "Healing Word".into(),
            healing: "1d8+3".into(),
        },
    ]);
    let rogue = Character::new("Rogue", 9, 14, Team::Heroes).with_actions(vec![Action::Attack {
        name: "Rapier".into(),
        hit_bonus: 5,
        damage: "1d8+3".into(),
    }]);
    let heroes = vec![fighter, cleric, rogue];

    let kobold_dagger = Action::Attack {
        name: "Dagger".into(),
        hit_bonus: 4,
        damage: "1d4+2".into(),
    };
    let kobold_sling = Action::Attack {
        name: "Sling".into(),
        hit_bonus: 4,
        damage: "1d4+2".into(),
    };
    let kobold_actions = vec![kobold_dagger, kobold_sling];
    let monsters = (0..args.num_kobolds)
        .into_iter()
        .map(|i| {
            Character::new(format!("Kobold {}", i + 1), 5, 12, Team::Monsters)
                .with_actions(kobold_actions.clone())
        })
        .collect();

    let mut combat = Combat::new(heroes, monsters);
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

    let stats = run_dice_monte_carlo(args.iterations);

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
