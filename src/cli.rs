use std::env;

pub enum Command {
    Insert { id: String, vec: Vec<f32> },
    Search { vec: Vec<f32>, k_top: usize },
    Get { id: String },
    List,
    Count,
    Delete { id: String },
}

/// Parse command-line arguments into a Command
pub fn parse_args() -> Result<Command, String> {
    let args: Vec<String> = env::args().collect();
    parse_command_from_args(&args)
}

/// Parse a command from a provided argument vector
/// This is used both for command-line args and REPL input
pub fn parse_command_from_args(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Err("No command provided. Use: get, insert, search, list, count, delete".to_string());
    }

    let command = &args[1];

    match command.as_str() {
        "get" => parse_get(&args),
        "insert" => parse_insert(&args),
        "search" => parse_search(&args),
        "list" => parse_list(&args),
        "count" => parse_count(&args),
        "delete" => parse_delete(&args),
        _ => Err(format!("Unknown command: {}. Available: get, insert, search, list, count, delete", command)),
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
