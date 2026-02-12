use kvdb::VecDB;
use std::time::Instant;

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

const MODEL_ID: &str = "BAAI/bge-base-en-v1.5";
const DB_FILE: &str = "wikipedia.db";

fn mean_pooling(
    hidden_states: &Tensor,
    attention_mask: &Tensor,
) -> candle_core::Result<Tensor> {
    let mask_expanded = attention_mask
        .unsqueeze(2)?
        .broadcast_as(hidden_states.shape())?
        .to_dtype(hidden_states.dtype())?;
    let sum_embeddings = (hidden_states * &mask_expanded)?.sum(1)?;
    let sum_mask = mask_expanded.sum(1)?.clamp(1e-9, f64::MAX)?;
    sum_embeddings.broadcast_div(&sum_mask)
}

fn l2_normalize(tensor: &Tensor) -> candle_core::Result<Tensor> {
    let norm = tensor.sqr()?.sum_keepdim(1)?.sqrt()?;
    tensor.broadcast_div(&norm.clamp(1e-12, f64::MAX)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get query from command line args
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: cargo run --example demo_semantic_search -- \"your search query\"");
        std::process::exit(1);
    }
    let query = args.join(" ");

    let device = Device::cuda_if_available(0)?;

    // Load model
    println!("Loading model...");
    let start = Instant::now();

    let api = Api::new()?;
    let repo = api.repo(Repo::new(MODEL_ID.to_string(), RepoType::Model));

    let tokenizer_path = repo.get("tokenizer.json")?;
    let config_path = repo.get("config.json")?;
    let weights_path = repo.get("model.safetensors")?;

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| e.to_string())?;

    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)?
    };
    let model = BertModel::load(vb, &config)?;
    println!("Model loaded in {:.3}s", start.elapsed().as_secs_f64());

    // Load database
    println!("Loading database...");
    let start = Instant::now();
    let db = VecDB::load(DB_FILE)?;
    println!("Loaded {} vectors in {:.3}s\n", db.count(), start.elapsed().as_secs_f64());

    // Embed query
    let start = Instant::now();
    let encoding = tokenizer.encode(query.as_str(), true).map_err(|e| e.to_string())?;
    let ids = Tensor::from_vec(
        encoding.get_ids().to_vec(),
        (1, encoding.get_ids().len()),
        &device,
    )?;
    let mask = Tensor::from_vec(
        encoding.get_attention_mask().to_vec(),
        (1, encoding.get_attention_mask().len()),
        &device,
    )?;
    let type_ids = ids.zeros_like()?;
    let hidden = model.forward(&ids, &type_ids, Some(&mask))?;
    let pooled = mean_pooling(&hidden, &mask)?;
    let normalized = l2_normalize(&pooled)?;
    let query_vec: Vec<f32> = normalized.get(0)?.to_vec1()?;
    let embed_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Search
    let start = Instant::now();
    let results = db.search(query_vec, 10)?;
    let search_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Print results
    println!("Query: \"{}\"", query);
    println!("Embed: {:.1}ms | Search: {:.1}ms\n", embed_ms, search_ms);
    for (rank, (id, _vec, score)) in results.iter().enumerate() {
        println!("  {:2}. {:<40} (score: {:.4})", rank + 1, id, score);
    }

    Ok(())
}