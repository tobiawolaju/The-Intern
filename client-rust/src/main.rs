use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

mod command;
mod logger;
mod runner;
mod server;
mod tray;
mod windows;

use command::CommandItem;
use logger::Logger;
use runner::run_command;
use windows::LocalBody;

#[derive(Parser, Debug)]
#[command(name = "intern-local")]
#[command(about = "Windows Local PC Body (CLI stub for command flow)")]
struct Cli {
    /// Write JSONL logs to a file (one event per command)
    #[arg(long)]
    log_file: Option<PathBuf>,

    /// Emit JSON events to stdout instead of plain text
    #[arg(long, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a single instruction provided on the command line
    Exec {
        /// Numeric index for the command (useful for ordering)
        #[arg(long)]
        index: u32,

        /// Instruction text
        #[arg(long)]
        instruction: String,

        /// Optional tag for grouping/filtering
        #[arg(long, default_value = "manual")]
        tag: String,
    },

    /// Run a command sheet (JSON array) from a file
    Run {
        /// Path to JSON file
        #[arg(long)]
        file: PathBuf,

        /// Optional tag filter (only run commands matching this tag)
        #[arg(long)]
        tag: Option<String>,
    },

    /// List commands from a command sheet (JSON array)
    List {
        /// Path to JSON file
        #[arg(long)]
        file: PathBuf,
    },

    /// Start a simple interactive prompt
    Repl,

    /// Start a WebSocket server (default if no command is provided)
    Serve {
        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,

        /// Port to listen on
        #[arg(long, default_value_t = 8765)]
        port: u16,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let mut logger = Logger::new(cli.log_file)?;
    let mut local_body = LocalBody::new();

    match cli.command.unwrap_or(Commands::Serve {
        bind: "127.0.0.1".to_string(),
        port: 8765,
    }) {
        Commands::Exec {
            index,
            instruction,
            tag,
        } => {
            let cmd = CommandItem {
                index,
                instruction,
                tag,
            };
            execute_command(&mut local_body, &mut logger, cli.json, &cmd)?;
        }
        Commands::Run { file, tag } => {
            let mut commands = load_commands(&file)?;
            commands.sort_by_key(|c| c.index);
            for cmd in commands {
                if let Some(ref t) = tag {
                    if cmd.tag != *t {
                        continue;
                    }
                }
                execute_command(&mut local_body, &mut logger, cli.json, &cmd)?;
            }
        }
        Commands::List { file } => {
            let mut commands = load_commands(&file)?;
            commands.sort_by_key(|c| c.index);
            for cmd in commands {
                println!("[{}] ({}) {}", cmd.index, cmd.tag, cmd.instruction);
            }
        }
        Commands::Repl => repl_loop(&mut local_body, &mut logger, cli.json)?,
        Commands::Serve { bind, port } => {
            let _tray = tray::init_tray()?;
            server::serve(&bind, port, &mut local_body, &mut logger)?;
        }
    }

    Ok(())
}

fn load_commands(path: &PathBuf) -> Result<Vec<CommandItem>, String> {
    let data = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let commands: Vec<CommandItem> = serde_json::from_str(&data)
        .map_err(|e| format!("invalid JSON in {}: {e}", path.display()))?;
    Ok(commands)
}

fn execute_command(
    local_body: &mut LocalBody,
    logger: &mut Logger,
    json_out: bool,
    cmd: &CommandItem,
) -> Result<(), String> {
    let event = run_command(local_body, logger, cmd)?;

    if json_out {
        let line = serde_json::to_string(&event).map_err(|e| e.to_string())?;
        println!("{line}");
    } else {
        println!(
            "EXEC -> index={} tag={} instruction=\"{}\" status={} detail=\"{}\" duration_ms={}",
            event.index, event.tag, event.instruction, event.status, event.detail, event.duration_ms
        );
    }

    Ok(())
}

fn repl_loop(local_body: &mut LocalBody, logger: &mut Logger, json_out: bool) -> Result<(), String> {
    println!("Intern Local REPL");
    println!("Enter commands as JSON: {{\"index\":1,\"instruction\":\"...\",\"tag\":\"...\"}}");
    println!("Type 'exit' to quit.");

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            return Err("failed to read input".to_string());
        }

        let line = line.trim();
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        if line.is_empty() {
            continue;
        }

        let cmd: CommandItem = match serde_json::from_str(line) {
            Ok(c) => c,
            Err(e) => {
                println!("Invalid JSON: {e}");
                continue;
            }
        };
        execute_command(local_body, logger, json_out, &cmd)?;
    }

    Ok(())
}
