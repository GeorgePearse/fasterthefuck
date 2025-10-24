use clap::Parser;
use fasterthefuck::{
    Command, Config, Corrector, RuleRegistry,
    rules::{
        git, filesystem, permissions, package_managers,
    },
};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "ftf")]
#[command(about = "Faster version of 'thefuck' - automatic command correction")]
#[command(version)]
struct Args {
    /// The command that failed
    #[arg(long)]
    command: String,

    /// The output/error message from the failed command
    #[arg(long)]
    output: String,

    /// The exit code from the failed command
    #[arg(long)]
    exit_code: i32,

    /// Skip interactive selection and just print first correction
    #[arg(long)]
    no_interaction: bool,

    /// Path to config file (defaults to ~/.config/fasterthefuck/config.toml)
    #[arg(long)]
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration
    let config = if let Some(config_path) = &args.config {
        Config::load_from_file(std::path::Path::new(config_path))?
    } else {
        Config::load_default().unwrap_or_default()
    };

    // Create rule registry and populate with all available rules
    let mut registry = RuleRegistry::new();

    // Add all git rules
    let mut git_rules = git::git_branch_rules();
    git_rules.extend(git::git_push_pull_rules());
    git_rules.extend(git::git_staging_rules());
    registry.add_rules(filter_rules_by_config(git_rules, &config));

    // Add all filesystem rules
    registry.add_rules(filter_rules_by_config(
        filesystem::filesystem_rules(),
        &config,
    ));

    // Add all permission rules
    registry.add_rules(filter_rules_by_config(
        permissions::permission_rules(),
        &config,
    ));

    // Add all package manager rules
    registry.add_rules(filter_rules_by_config(
        package_managers::package_manager_rules(),
        &config,
    ));

    // Create corrector and evaluate
    let corrector: Corrector = registry.into();
    let cmd = Command {
        script: args.command,
        output: args.output,
        exit_code: args.exit_code,
    };

    let corrections = corrector.get_corrections(&cmd);

    // Handle different correction scenarios
    match corrections.len() {
        0 => {
            // No corrections found
            std::process::exit(1);
        }
        1 => {
            // Single correction - print and exit
            println!("{}", corrections[0].script);
            std::process::exit(0);
        }
        _ => {
            // Multiple corrections - interactive selection or print first
            if args.no_interaction {
                println!("{}", corrections[0].script);
                std::process::exit(0);
            } else {
                if let Some(selected) = select_correction_interactive(&corrections) {
                    println!("{}", selected);
                    std::process::exit(0);
                } else {
                    // User cancelled or no selection
                    std::process::exit(1);
                }
            }
        }
    }
}

fn select_correction_interactive(corrections: &[fasterthefuck::CorrectedCommand]) -> Option<String> {
    eprintln!("\nMultiple corrections available:");
    for (i, correction) in corrections.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, correction.script);
    }

    eprint!("\nSelect correction (1-{}): ", corrections.len());
    let _ = io::stderr().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None;
    }

    if let Ok(idx) = input.trim().parse::<usize>() {
        if idx > 0 && idx <= corrections.len() {
            return Some(corrections[idx - 1].script.clone());
        }
    }

    None
}

/// Filters rules based on configuration.
/// Removes rules that are disabled in the config.
fn filter_rules_by_config(
    rules: Vec<Box<dyn fasterthefuck::Rule>>,
    config: &Config,
) -> Vec<Box<dyn fasterthefuck::Rule>> {
    rules
        .into_iter()
        .filter(|rule| config.is_rule_enabled(rule.name()))
        .collect()
}
