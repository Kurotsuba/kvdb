use std::env;
use std::io::{self, Write};
use kvdb::VecDB;

pub enum Command {
    Insert { id: String, vec: Vec<f32> },
    Search { vec: Vec<f32>, k_top: usize },
    Get { id: String },
    List,
    Count,
    Delete { id: String },
    Save { path: String },
    Load { path: String },
}

/// Parse a command from a provided argument vector
/// This is used both for command-line args and REPL input
pub fn parse_command_from_args(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Err("No command provided. Use: get, insert, search, list, count, delete, save, load".to_string());
    }

    let command = &args[1];

    match command.as_str() {
        "get" => parse_get(&args),
        "insert" => parse_insert(&args),
        "search" => parse_search(&args),
        "list" => parse_list(&args),
        "count" => parse_count(&args),
        "delete" => parse_delete(&args),
        "save" => parse_save(&args),
        "load" => parse_load(&args),
        _ => Err(format!("Unknown command: {}. Available: get, insert, search, list, count, delete, save, load", command)),
    }
}

/// Parse the 'insert' command
/// Usage: kvdb insert <id> <vector>
fn parse_insert(args: &[String]) -> Result<Command, String> {
    // args[0] = program name
    // args[1] = "insert"
    // args[2] = id (required)
    // args[3..] = vector (required, at least 1)
    if args.len() < 4 {
        return Err("'insert' command requires an ID and a vector. Usage: kvdb insert <id> <vector>".to_string());
    }

    let id = args[2].clone();
    let vec: Result<Vec<f32>, _> = args[3..].iter()
        .map(|s| s.parse::<f32>())
        .collect();

    match vec {
        Ok(v) => Ok(Command::Insert { id, vec: v }),
        Err(_) => Err("Vector parsing error".to_string()),
    }
    
}

/// Parse the 'search' command
/// Usage: kvdb search <v1> <v2> ... [--k_top <number>]
fn parse_search(args: &[String]) -> Result<Command, String> {
    // args[0] = program name
    // args[1] = "search"
    // args[2..] = vector components and optional --k_top flag

    if args.len() < 3 {
        return Err("'search' command requires at least one vector component. Usage: kvdb search <v1> <v2> ... [--k_top <number>]".to_string());
    }

    let mut k_top = 5; // default value
    let mut vector_end = args.len();

    // Check if last two args are --k_top and a number
    if args.len() >= 4 && args[args.len() - 2] == "--k_top" {
        // Try to parse the last argument as k_top
        match args[args.len() - 1].parse::<usize>() {
            Ok(k) => {
                k_top = k;
                vector_end = args.len() - 2; // Exclude --k_top and the number
            }
            Err(_) => {
                return Err(format!("Invalid --k_top value: '{}'. Must be a positive integer.", args[args.len() - 1]));
            }
        }
    }

    // Parse vector components from args[2] to vector_end
    let vec: Result<Vec<f32>, _> = args[2..vector_end].iter()
        .map(|s| s.parse::<f32>())
        .collect();

    match vec {
        Ok(v) => {
            if v.is_empty() {
                return Err("Search vector cannot be empty".to_string());
            }
            Ok(Command::Search { vec: v, k_top })
        }
        Err(_) => Err("Failed to parse vector components as numbers".to_string()),
    }
}

/// Parse the 'get' command
/// Usage: kvdb get <id>
fn parse_get(args: &[String]) -> Result<Command, String> {
    // args[0] = program name
    // args[1] = "get"
    // args[2] = id (required)

    if args.len() < 3 {
        return Err("'get' command requires an ID. Usage: kvdb get <id>".to_string());
    }

    let id = args[2].clone();

    Ok(Command::Get { id })
}

/// Parse the 'list' command
/// Usage: kvdb list
fn parse_list(args: &[String]) -> Result<Command, String> {
    // List takes no arguments
    if args.len() > 2 {
        eprintln!("Warning: 'list' command takes no arguments, ignoring extras");
    }

    Ok(Command::List)
}

/// Parse the 'count' command
/// Usage: kvdb count
fn parse_count(args: &[String]) -> Result<Command, String> {
    // Count takes no arguments
    if args.len() > 2 {
        eprintln!("Warning: 'count' command takes no arguments, ignoring extras");
    }

    Ok(Command::Count)
}

/// Parse the 'delete' command
/// Usage: kvdb delete
fn parse_delete(args: &[String]) -> Result<Command, String> {
    // args[0] = program name
    // args[1] = "delete"
    // args[2] = id (required)
    if args.len() < 3 {
        return Err("'delete' command requires an ID. Usage: kvdb delete <id>".to_string());
    }
    let id = args[2].clone();
    Ok(Command::Delete { id })
}

/// Parse the 'save' command
/// Usage: kvdb save <path>
fn parse_save(args: &[String]) -> Result<Command, String> {
    if args.len() < 3 {
        return Err("'save' command requires a file path. Usage: save <path>".to_string());
    }
    let path = args[2].clone();
    Ok(Command::Save { path })
}

/// Parse the 'load' command
/// Usage: kvdb load <path>
fn parse_load(args: &[String]) -> Result<Command, String> {
    if args.len() < 3 {
        return Err("'load' command requires a file path. Usage: load <path>".to_string());
    }
    let path = args[2].clone();
    Ok(Command::Load { path })
}

/// REPL mode - interactive session with persistent database
pub fn run_repl(db: &mut VecDB) {
    println!("KVDB - Vector Database");
    println!("Type 'help' for commands, 'exit' or 'quit' to quit\n");

    loop {
        print!("kvdb> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        if input == "help" {
            print_help();
            continue;
        }

        let mut args: Vec<String> = vec!["kvdb".to_string()];
        args.extend(input.split_whitespace().map(|s| s.to_string()));

        let command = match parse_command_from_args(&args) {
            Ok(cmd) => cmd,
            Err(error) => {
                eprintln!("Error: {}", error);
                continue;
            }
        };

        execute_command(db, command);
    }
}

/// Single-command mode - load db from path, execute command, save back
/// Usage: kvdb <db_path> <command> [args...]
pub fn run_single_command() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: kvdb <db_path> <command> [args...]");
        std::process::exit(1);
    }

    let db_path = &args[1];

    // Load existing db or create new
    let mut db = if std::path::Path::new(db_path).exists() {
        match VecDB::load(db_path) {
            Ok(loaded) => loaded,
            Err(e) => {
                eprintln!("Error loading '{}': {}", db_path, e);
                std::process::exit(1);
            }
        }
    } else {
        VecDB::new()
    };

    // Rebuild args: shift so args[1] becomes the command
    let shifted_args: Vec<String> = std::iter::once(args[0].clone())
        .chain(args[2..].iter().cloned())
        .collect();

    let command = match parse_command_from_args(&shifted_args) {
        Ok(cmd) => cmd,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

    execute_command(&mut db, command);

    // Save db back to path
    if let Err(e) = db.save(db_path) {
        eprintln!("Error saving '{}': {}", db_path, e);
        std::process::exit(1);
    }
}

fn execute_command(db: &mut VecDB, command: Command) {
    match command {
        Command::Get { id } => {
            match db.get(&id) {
                Some(vector) => println!("Vector '{}': {:?}", id, vector),
                None => eprintln!("Error: Vector '{}' not found", id),
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

        Command::Count => println!("{}", db.count()),

        Command::Insert { id, vec } => {
            match db.insert(id.clone(), vec) {
                Ok(message) => println!("{}", message),
                Err(error) => eprintln!("Error: {}", error),
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
                Err(error) => eprintln!("Error: {}", error),
            }
        }

        Command::Delete { id } => {
            match db.delete(&id) {
                Ok(message) => println!("{}", message),
                Err(error) => eprintln!("Error: {}", error),
            }
        }

        Command::Save { path } => {
            match db.save(&path) {
                Ok(()) => println!("Database saved to '{}'", path),
                Err(error) => eprintln!("Error: {}", error),
            }
        }

        Command::Load { path } => {
            match VecDB::load(&path) {
                Ok(loaded_db) => {
                    let count = loaded_db.count();
                    *db = loaded_db;
                    println!("Database loaded from '{}' ({} vectors)", path, count);
                }
                Err(error) => eprintln!("Error: {}", error),
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
    println!("  save <path>                      - Save database to file");
    println!("  load <path>                      - Load database from file");
    println!("  help                             - Show this help");
    println!("  exit, quit                       - Exit the program");
}
