use clap::Parser;
use fasterthefuck::{
    Command, Corrector, RuleRegistry,
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create rule registry and populate with all available rules
    let mut registry = RuleRegistry::new();

    // Add all git rules
    registry.add_rules(git::git_branch_rules());
    registry.add_rules(git::git_push_pull_rules());
    registry.add_rules(git::git_staging_rules());

    // Add all filesystem rules
    registry.add_rules(filesystem::filesystem_rules());

    // Add all permission rules
    registry.add_rules(permissions::permission_rules());

    // Add all package manager rules
    registry.add_rules(package_managers::package_manager_rules());

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
