mod cli;

use actix_web::{App, HttpServer};
use kvdb::VecDB;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let mut db = VecDB::new();
        cli::run_repl(&mut db);
    } else {
        if args[1] == "serve" {
            HttpServer::new(|| App::new().configure(kvdb::server::config))
                .bind("0.0.0.0:7878")?
                .run()
                .await?;
            
        } else {
            cli::run_single_command();
        }
    }

    Ok(())
}