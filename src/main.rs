mod cli;

use kvdb::VecDB;
use cli::Command;
use std::io::{self, Write};

fn main() {
    // Create database instance once - persists across commands
    let mut db = VecDB::new();

    // Check if running in REPL mode (no command-line args) or single-command mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // REPL mode - no arguments provided
        run_repl(&mut db);
    } else {
        // Single-command mode
        run_single_command(&mut db);
    }
}

/// REPL mode - interactive session with persistent database
fn run_repl(db: &mut VecDB) {
    println!("KVDB - Vector Database");
    println!("Type 'help' for commands, 'exit' or 'quit' to quit\n");

    loop {
        // Print prompt
        print!("kvdb> ");
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                continue;
            }
        }

        // Parse input into arguments
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Check for special commands
        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        if input == "help" {
            print_help();
            continue;
        }

        // Parse the command from the input string
        let mut args: Vec<String> = vec!["kvdb".to_string()];
        args.extend(input.split_whitespace().map(|s| s.to_string()));

        // Build a mock env::args for parsing
        let command = match cli::parse_command_from_args(&args) {
            Ok(cmd) => cmd,
            Err(error) => {
                eprintln!("Error: {}", error);
                continue; // Don't exit, just continue to next command
            }
        };

        // Execute command - errors don't exit in REPL mode
        execute_command(db, command);
    }
}

/// Single-command mode - execute one command and exit
fn run_single_command(db: &mut VecDB) {
    let command = match cli::parse_args() {
        Ok(cmd) => cmd,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

    execute_command(db, command);
}

/// Execute a command against the database
fn execute_command(db: &mut VecDB, command: Command) {
    match command {
        Command::Get { id } => {
            match db.get(&id) {
                Some(vector) => {
                    println!("Vector '{}': {:?}", id, vector);
                }
                None => {
                    eprintln!("Error: Vector '{}' not found", id);
                }
            }
        }

        Command::List => {
            let vectors = db.list();
            if vectors.is_empty() {
                println!("Database is empty");
            } else {
                println!("Stored vectors:");
                for (id, vec) in vectors {
                    println!("  {}: {:?}", id, vec);
                }
                println!("Total: {} vectors", db.count());
            }
        }

        Command::Count => {
            println!("{}", db.count());
        }

        Command::Insert { id, vec } => {
            match db.insert(id.clone(), vec) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
        }

        Command::Search { vec, k_top } => {
            match db.search(vec, k_top) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No results found");
                    } else {
                        println!("Top {} results:", results.len());
                        for (rank, (id, vector, score)) in results.iter().enumerate() {
                            println!("{}. ID: {}, Score: {:.4}, Vector: {:?}",
                                rank + 1, id, score, vector);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
        }

        Command::Delete { id } => {
            match db.delete(&id) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  insert <id> <v1> <v2> ...        - Insert a vector");
    println!("  search <v1> <v2> ... [--k_top N] - Search for similar vectors (default k=5)");
    println!("  get <id>                         - Retrieve a vector by ID");
    println!("  list                             - List all vectors");
    println!("  count                            - Show vector count");
    println!("  delete <id>                      - Delete a vector");
    println!("  help                             - Show this help");
    println!("  exit, quit                       - Exit the program");
}
