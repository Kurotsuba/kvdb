//! REST API for kvdb.
//!
//! Provides a stateless HTTP server with JSON endpoints for vector operations.
//! Each request includes a `db` field specifying the database file path.
//! The server loads the database from disk per request and saves after mutations.
//!
//! ## Endpoints
//!
//! - `POST /insert` - Insert or update vectors
//! - `POST /search` - Search for similar vectors
//! - `POST /get` - Retrieve vectors by ID
//! - `POST /delete` - Delete vectors by ID
//!
//! ## Usage
//!
//! ```rust,no_run
//! use actix_web::{App, HttpServer};
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| App::new().configure(kvdb::server::config))
//!         .bind("0.0.0.0:7878")?
//!         .run()
//!         .await
//! }
//! ```

use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use crate::VecDB;
use std::path::Path;


// --- Request structs ---

#[derive(Deserialize)]
struct VectorEntry {
    id: String,
    values: Vec<f32>,
}

#[derive(Deserialize)]
struct Query {
    value: Vec<f32>,
    top_k: usize,
}

#[derive(Deserialize)]
struct InsertRequest {
    db: String,
    vectors: Vec<VectorEntry>,
}

#[derive(Deserialize)]
struct SearchRequest {
    db: String,
    queries: Vec<Query>,
}

#[derive(Deserialize)]
struct GetRequest {
    db: String,
    ids: Vec<String>,
}

#[derive(Deserialize)]
struct DeleteRequest {
    db: String,
    ids: Vec<String>,
}

// --- Response structs ---

#[derive(Serialize)]
struct InsertResponse {
    inserted: usize,
    results: Vec<InsertResult>,
}

#[derive(Serialize)]
struct InsertResult {
    id: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResultGroup>,
}

#[derive(Serialize)]
struct SearchResultGroup {
    matches: Vec<MatchResult>,
    message: String,
}

#[derive(Serialize)]
struct MatchResult {
    id: String,
    score: f32,
    values: Vec<f32>,
}

#[derive(Serialize)]
struct GetResponse {
    results: Vec<GetResult>,
}

#[derive(Serialize)]
struct GetResult {
    id: String,
    values: Option<Vec<f32>>,
}

#[derive(Serialize)]
struct DeleteResponse {
    deleted: usize,
    results: Vec<DeleteResult>,
}

#[derive(Serialize)]
struct DeleteResult {
    id: String,
    status: String,
    message: String,
}


/// Helper function for load or create database
fn load_or_create(path: &str) -> Result<VecDB, String> {
    if Path::new(path).exists() {
        return VecDB::load(path);
    }

    Ok(VecDB::new())
}

// --- Handlers ---

async fn insert_handler(body: web::Json<InsertRequest>) -> impl Responder {
    let mut db = match load_or_create(&body.db) {
        Ok(db) => db,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    };

    let mut results = Vec::new();
    let mut inserted = 0;

    for entry in &body.vectors {
        match db.insert(entry.id.clone(), entry.values.clone()) {
            Ok(msg) => {
                inserted += 1;
                results.push(InsertResult {
                    id: entry.id.clone(),
                    status: "ok".to_string(),
                    message: msg,
                });
            }
            Err(e) => {
                results.push(InsertResult {
                    id: entry.id.clone(),
                    status: "error".to_string(),
                    message: e,
                });
            }
        }
    }

    if let Err(e) = db.save(&body.db) {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e}));
    }
    
    HttpResponse::Ok().json(InsertResponse { inserted, results })
}

async fn search_handler(body: web::Json<SearchRequest>) -> impl Responder {
    // load the db
    let db = match load_or_create(&body.db) {
        Ok(db) => db,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    };
    
    let mut results = Vec::new();
    
    for entry in &body.queries {
        match db.search(entry.value.clone(), entry.top_k) {
            Ok(res) => {
                results.push(SearchResultGroup { 
                    matches: res.iter()
                    .map(|(id, vec, score)| MatchResult {
                        id: id.clone(),
                        score: *score,
                        values: vec.clone(),
                    })
                    .collect(),
                    message: "Search Success".to_string(), 
                });
            }
            Err(e ) => {
                results.push(SearchResultGroup {
                    matches: Vec::new(),
                    message: e, 
                });
            }
        }
    }
    
    HttpResponse::Ok().json(SearchResponse { results })
}

async fn get_handler(body: web::Json<GetRequest>) -> impl Responder {
    let db = match load_or_create(&body.db) {
        Ok(db) => db,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    };
    
    let mut results = Vec::new();
    
    for entry in &body.ids {
        results.push(GetResult {
            id: entry.clone(),
            values: db.get(entry),
        });
    }
    
    HttpResponse::Ok().json(GetResponse { results })
}

async fn delete_handler(body: web::Json<DeleteRequest>) -> impl Responder {
    let mut db = match load_or_create(&body.db) {
        Ok(db) => db,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    };
    
    let mut results = Vec::new();
    let mut deleted = 0;
    
    for entry in &body.ids {
        match db.delete(entry) {
            Ok(msg) => {
                results.push(DeleteResult {
                    id: entry.clone(),
                    status: "Success".to_string(),
                    message: msg,
                });

                deleted += 1;
            },
            Err(e) => {
                results.push(DeleteResult {
                    id: entry.clone(),
                    status: "Failed".to_string(),
                    message: e,
                });
            },
        }
    }
    
    if let Err(e) = db.save(&body.db) {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e}));
    }
    
    HttpResponse::Ok().json(DeleteResponse { results, deleted })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/insert").route(web::post().to(insert_handler)))
       .service(web::resource("/search").route(web::post().to(search_handler)))
       .service(web::resource("/get").route(web::post().to(get_handler)))
       .service(web::resource("/delete").route(web::post().to(delete_handler)));
}

