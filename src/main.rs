mod cli;

use kvdb::VecDB;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let mut db = VecDB::new();
        cli::run_repl(&mut db);
    } else {
        cli::run_single_command();
    }
}