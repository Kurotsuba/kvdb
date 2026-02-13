use actix_web::{App, HttpServer};
use reqwest::Client;
use serde_json::json;
use std::net::TcpListener;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Find a free port by binding to port 0
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[actix_web::test]
async fn test_insert_and_search() {
    let port = free_port();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    // Start server in background
    let server = HttpServer::new(|| App::new().configure(kvdb::server::config))
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
    let handle = server.handle();
    tokio::spawn(server);
    sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // --- Insert 3 vectors ---
    let resp = client
        .post(format!("{}/insert", base))
        .json(&json!({
            "db": db_path,
            "vectors": [
                {"id": "vec1", "values": [1.0, 0.0, 0.0]},
                {"id": "vec2", "values": [0.0, 1.0, 0.0]},
                {"id": "vec3", "values": [0.7, 0.7, 0.0]}
            ]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["inserted"], 3);
    assert_eq!(body["results"].as_array().unwrap().len(), 3);

    // --- Search: closest to [1, 0, 0] should be vec1 ---
    let resp = client
        .post(format!("{}/search", base))
        .json(&json!({
            "db": db_path,
            "queries": [
                {"value": [1.0, 0.0, 0.0], "top_k": 2}
            ]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let matches = body["results"][0]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["id"], "vec1"); // most similar

    handle.stop(true).await;
}

#[actix_web::test]
async fn test_get_existing_and_missing() {
    let port = free_port();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let server = HttpServer::new(|| App::new().configure(kvdb::server::config))
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
    let handle = server.handle();
    tokio::spawn(server);
    sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Insert one vector
    client
        .post(format!("{}/insert", base))
        .json(&json!({
            "db": db_path,
            "vectors": [{"id": "v1", "values": [1.0, 0.0, 0.0]}]
        }))
        .send()
        .await
        .unwrap();

    // --- Get existing + missing ---
    let resp = client
        .post(format!("{}/get", base))
        .json(&json!({
            "db": db_path,
            "ids": ["v1", "v_missing"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();

    // v1 should have values
    assert_eq!(results[0]["id"], "v1");
    assert!(!results[0]["values"].is_null());

    // v_missing should have null values
    assert_eq!(results[1]["id"], "v_missing");
    assert!(results[1]["values"].is_null());

    handle.stop(true).await;
}

#[actix_web::test]
async fn test_delete_and_verify() {
    let port = free_port();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let server = HttpServer::new(|| App::new().configure(kvdb::server::config))
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
    let handle = server.handle();
    tokio::spawn(server);
    sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Insert 2 vectors
    client
        .post(format!("{}/insert", base))
        .json(&json!({
            "db": db_path,
            "vectors": [
                {"id": "a", "values": [1.0, 0.0]},
                {"id": "b", "values": [0.0, 1.0]}
            ]
        }))
        .send()
        .await
        .unwrap();

    // --- Delete one existing, one missing ---
    let resp = client
        .post(format!("{}/delete", base))
        .json(&json!({
            "db": db_path,
            "ids": ["a", "no_such_id"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["deleted"], 1);

    // Verify "a" is gone via get
    let resp = client
        .post(format!("{}/get", base))
        .json(&json!({
            "db": db_path,
            "ids": ["a", "b"]
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results[0]["values"].is_null()); // "a" deleted
    assert!(!results[1]["values"].is_null()); // "b" still exists

    handle.stop(true).await;
}

#[actix_web::test]
async fn test_insert_duplicate_id() {
    let port = free_port();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let server = HttpServer::new(|| App::new().configure(kvdb::server::config))
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
    let handle = server.handle();
    tokio::spawn(server);
    sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Insert a vector
    client
        .post(format!("{}/insert", base))
        .json(&json!({
            "db": db_path,
            "vectors": [{"id": "dup", "values": [1.0, 0.0]}]
        }))
        .send()
        .await
        .unwrap();

    // Insert again with same id — should update (upsert behavior)
    let resp = client
        .post(format!("{}/insert", base))
        .json(&json!({
            "db": db_path,
            "vectors": [{"id": "dup", "values": [0.0, 1.0]}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["inserted"], 1);
    assert_eq!(body["results"][0]["status"], "ok");

    // Verify the vector was updated
    let resp = client
        .post(format!("{}/get", base))
        .json(&json!({ "db": db_path, "ids": ["dup"] }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let values = body["results"][0]["values"].as_array().unwrap();
    // After L2 normalization of [0.0, 1.0], should be [0.0, 1.0]
    assert!((values[0].as_f64().unwrap()).abs() < 0.01);
    assert!((values[1].as_f64().unwrap() - 1.0).abs() < 0.01);

    handle.stop(true).await;
}

#[actix_web::test]
async fn test_search_empty_db() {
    let port = free_port();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("empty.db").to_str().unwrap().to_string();

    let server = HttpServer::new(|| App::new().configure(kvdb::server::config))
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
    let handle = server.handle();
    tokio::spawn(server);
    sleep(Duration::from_millis(200)).await;

    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Search on a db that doesn't exist yet (will be created empty)
    let resp = client
        .post(format!("{}/search", base))
        .json(&json!({
            "db": db_path,
            "queries": [{"value": [1.0, 0.0], "top_k": 5}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    // Should return results with an error message (empty db)
    let group = &body["results"][0];
    assert!(group["matches"].as_array().unwrap().is_empty());

    handle.stop(true).await;
}