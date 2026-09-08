//! forge-migrate CLI: parse a GlassForge scan artifact and execute code modernization.
//!
//! Usage:
//!   forge-migrate --artifact <path-to-json> --repo <path-to-ios-repo>
//!
//! Reads the GlassForge artifact, plans migration tasks grouped by file,
//! prioritized by severity, and executes them sequentially via Claude Code agents.

use std::path::PathBuf;
use std::process::ExitCode;

use forge_migrate::runner::{MigrationRunner, RunnerConfig, validate_repo};
use forge_migrate::{parse_artifact, plan_migration};
use tracing::{error, info};

/// Parsed CLI arguments for the migrate command.
struct MigrateArgs {
    /// Path to the GlassForge scan artifact JSON.
    artifact: PathBuf,
    /// Path to the iOS repository to modernize.
    repo: PathBuf,
    /// Skip worktree isolation (work directly in repo).
    no_worktree: bool,
    /// Dry run: plan only, do not execute.
    dry_run: bool,
}

fn parse_args() -> Result<MigrateArgs, String> {
    let args: Vec<String> = std::env::args().collect();
    let mut artifact: Option<PathBuf> = None;
    let mut repo: Option<PathBuf> = None;
    let mut no_worktree = false;
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--artifact" | "-a" => {
                i += 1;
                if i >= args.len() {
                    return Err("--artifact requires a path argument".into());
                }
                artifact = Some(PathBuf::from(&args[i]));
            }
            "--repo" | "-r" => {
                i += 1;
                if i >= args.len() {
                    return Err("--repo requires a path argument".into());
                }
                repo = Some(PathBuf::from(&args[i]));
            }
            "--no-worktree" => {
                no_worktree = true;
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {}", other));
            }
        }
        i += 1;
    }

    let artifact = artifact.ok_or("--artifact <path> is required")?;
    let repo = repo.ok_or("--repo <path> is required")?;

    Ok(MigrateArgs {
        artifact,
        repo,
        no_worktree,
        dry_run,
    })
}

fn print_usage() {
    eprintln!(
        "forge-migrate - GlassForge migration pipeline\n\
         \n\
         USAGE:\n\
         \x20 forge-migrate --artifact <path> --repo <path> [OPTIONS]\n\
         \n\
         REQUIRED:\n\
         \x20 --artifact, -a <path>   Path to GlassForge scan artifact JSON\n\
         \x20 --repo, -r <path>       Path to the iOS repository to modernize\n\
         \n\
         OPTIONS:\n\
         \x20 --no-worktree           Work directly in the repo (skip worktree isolation)\n\
         \x20 --dry-run               Plan only, do not execute agents\n\
         \x20 --help, -h              Show this help message"
    );
}

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize tracing.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_usage();
            return ExitCode::from(2);
        }
    };

    // Validate inputs.
    if !args.artifact.exists() {
        eprintln!(
            "Error: artifact file does not exist: {}",
            args.artifact.display()
        );
        return ExitCode::from(1);
    }

    if let Err(e) = validate_repo(&args.repo) {
        eprintln!("Error: {}", e);
        return ExitCode::from(1);
    }

    // Parse the artifact.
    info!(artifact = %args.artifact.display(), "parsing GlassForge artifact");
    let artifact = match parse_artifact(&args.artifact) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error parsing artifact: {}", e);
            return ExitCode::from(1);
        }
    };

    // Print scan metadata if available.
    if let Some(ref meta) = artifact.meta {
        info!(
            repo = meta.repo_name.as_deref().unwrap_or("unknown"),
            branch = meta.branch.as_deref().unwrap_or("unknown"),
            commit = meta.commit.as_deref().unwrap_or("unknown"),
            target_sdk = meta.target_sdk.unwrap_or(0),
            "scan metadata"
        );
    }

    // Plan the migration.
    let mut plan = plan_migration(&artifact);
    info!(summary = %plan.summary(), "migration plan ready");

    // Print the plan.
    println!();
    println!("=== Migration Plan ===");
    println!("{}", plan.summary());
    println!();

    for (i, task) in plan.tasks.iter().enumerate() {
        let status_tag = match &task.status {
            forge_migrate::TaskStatus::Skipped { reason } => format!("[SKIP: {}]", reason),
            _ => format!("[{}]", task.priority),
        };
        println!(
            "  {:>3}. {} {} ({} findings)",
            i + 1,
            status_tag,
            task.file_path,
            task.findings.len(),
        );
    }
    println!();

    if args.dry_run {
        println!("Dry run complete. No agents were executed.");
        return ExitCode::SUCCESS;
    }

    // Filter to only actionable tasks.
    let actionable_count = plan
        .tasks
        .iter()
        .filter(|t| matches!(t.status, forge_migrate::TaskStatus::Pending))
        .count();

    if actionable_count == 0 {
        println!("No actionable migration tasks. Nothing to do.");
        return ExitCode::SUCCESS;
    }

    println!(
        "Executing {} actionable migration tasks...",
        actionable_count
    );
    println!();

    // Execute.
    let config = RunnerConfig {
        repo_path: args.repo.clone(),
        use_worktrees: !args.no_worktree,
        spawn_config: None,
        max_output_bytes: 2 * 1024 * 1024,
    };
    let runner = MigrationRunner::new(config);
    let results = runner.execute_all(&mut plan.tasks).await;

    // Print results.
    println!();
    println!("=== Migration Results ===");
    println!();

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    for result in &results {
        match &result.status {
            forge_migrate::TaskStatus::Completed { summary } => {
                succeeded += 1;
                println!("  [OK] {} - {}", result.file_path, summary);
                if let Some(ref wt) = result.worktree_path {
                    println!(
                        "       Changes in worktree: {}",
                        wt.display()
                    );
                }
            }
            forge_migrate::TaskStatus::Failed { error } => {
                failed += 1;
                println!("  [FAIL] {} - {}", result.file_path, error);
            }
            forge_migrate::TaskStatus::Skipped { reason } => {
                skipped += 1;
                println!("  [SKIP] {} - {}", result.file_path, reason);
            }
            other => {
                println!("  [{}] {}", other, result.file_path);
            }
        }
    }

    println!();
    println!(
        "Summary: {} succeeded, {} failed, {} skipped (of {} total)",
        succeeded,
        failed,
        skipped,
        results.len()
    );

    if failed > 0 {
        error!(failed, "some migration tasks failed");
        ExitCode::from(1)
    } else {
        info!(succeeded, skipped, "migration complete");
        ExitCode::SUCCESS
    }
}
