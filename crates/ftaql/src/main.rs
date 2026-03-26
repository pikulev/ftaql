use clap::Parser;
use ftaql::analyze_project;
use ftaql::config::read_config;
use ftaql::sqlite::{persist_run, PersistRunOptions};
use std::fs;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true, help = "Path to the project to analyze")]
    project: String,

    #[arg(long, short, help = "Path to config file")]
    config_path: Option<String>,

    #[arg(long, short = 'd', help = "Path to the sqlite database file")]
    db: String,

    #[arg(long, help = "Revision identifier to store with this snapshot")]
    revision: Option<String>,

    #[arg(long = "ref", help = "Human-readable branch, tag or channel label")]
    ref_label: Option<String>,
}

pub fn main() {
    // Start tracking execution time
    let start = Instant::now();

    let cli = Cli::parse();

    // Resolve the ftaql.json path, which can optionally be used-supplied
    let (config_path, path_specified_by_user) = match cli.config_path {
        Some(config_path_arg) => (config_path_arg, true),
        None => (format!("{}/ftaql.json", cli.project), false),
    };

    // Resolve the input config. Optionally adds ftaql.json values to the default config.
    let config = match read_config(config_path, path_specified_by_user) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let project_root = match fs::canonicalize(&cli.project) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            eprintln!("Failed to resolve project path '{}': {}", cli.project, err);
            std::process::exit(1);
        }
    };

    // Execute the analysis
    let analysis_result = analyze_project(&project_root, Some(config.clone()));

    // Execution finished, capture elapsed time
    let elapsed = start.elapsed().as_secs_f64();

    let summary = match persist_run(
        &cli.db,
        &analysis_result,
        &PersistRunOptions {
            project_root: &project_root,
            revision: cli.revision.as_deref(),
            ref_label: cli.ref_label.as_deref(),
            elapsed_seconds: elapsed,
            config: &config,
        },
    ) {
        Ok(summary) => summary,
        Err(err) => {
            eprintln!("Failed to persist analysis into sqlite: {err:#}");
            std::process::exit(1);
        }
    };

    println!("Persisted analysis run {} to {}", summary.run_id, cli.db);
    println!(
        "Files: {}, cycles: {}, runtime cycles: {}, elapsed: {:.4}s",
        summary.file_count, summary.cycle_count, summary.runtime_cycle_count, elapsed
    );
    if let Some(revision) = cli.revision {
        println!("Revision: {}", revision);
    }
}
