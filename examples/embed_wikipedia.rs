use kvdb::VecDB;
use std::time::Instant;

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::{PaddingParams, Tokenizer, TruncationParams};

const MODEL_ID: &str = "BAAI/bge-base-en-v1.5";
const INPUT_FILE: &str = "examples/wikipedia_100k.json";
const OUTPUT_FILE: &str = "wikipedia.db";
const BATCH_SIZE: usize = 64;

#[derive(serde::Deserialize)]
struct WikiPage {
    title: String,
    description: String,
}

fn load_wikipedia_pages(path: &str) -> Vec<WikiPage> {
    let data = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read {}. Run fetch_wikipedia.py first.", path));
    serde_json::from_str(&data).expect("Failed to parse JSON")
}

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
    let device = Device::cuda_if_available(0)?;
    match &device {
        Device::Cuda(_) => println!("Using CUDA GPU\n"),
        _ => println!("CUDA not available, using CPU (build with --features cuda for GPU)\n"),
    }

    // Phase 1: Load Wikipedia pages
    println!("Phase 1: Loading Wikipedia pages from '{}'...", INPUT_FILE);
    let start = Instant::now();
    let pages = load_wikipedia_pages(INPUT_FILE);
    println!("  Loaded {} pages in {:.3}s\n", pages.len(), start.elapsed().as_secs_f64());

    // Phase 2: Load model and tokenizer from HuggingFace Hub
    println!("Phase 2: Loading model '{}'...", MODEL_ID);
    let start = Instant::now();

    let api = Api::new()?;
    let repo = api.repo(Repo::new(MODEL_ID.to_string(), RepoType::Model));

    let tokenizer_path = repo.get("tokenizer.json")?;
    let config_path = repo.get("config.json")?;
    let weights_path = repo.get("model.safetensors")?;

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
    let mut tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| e.to_string())?;

    // Set up padding and truncation for batch processing
    tokenizer.with_padding(Some(PaddingParams::default()));
    tokenizer.with_truncation(Some(TruncationParams {
        max_length: 128,
        ..Default::default()
    })).map_err(|e| e.to_string())?;

    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)?
    };
    let model = BertModel::load(vb, &config)?;

    println!("  Model loaded in {:.3}s\n", start.elapsed().as_secs_f64());

    // Phase 3: Embed title+description in batches
    println!("Phase 3: Embedding {} pages (batch_size={})...", pages.len(), BATCH_SIZE);
    let start = Instant::now();
    let mut db = VecDB::new();
    let total_batches = (pages.len() + BATCH_SIZE - 1) / BATCH_SIZE;

    for (batch_idx, chunk) in pages.chunks(BATCH_SIZE).enumerate() {
        let texts: Vec<String> = chunk
            .iter()
            .map(|p| format!("{}: {}", p.title, p.description))
            .collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        // Tokenize batch
        let encodings = tokenizer
            .encode_batch(text_refs, true)
            .map_err(|e| e.to_string())?;

        let token_ids: Vec<&[u32]> = encodings.iter().map(|e| e.get_ids()).collect();
        let attention_masks: Vec<&[u32]> = encodings.iter().map(|e| e.get_attention_mask()).collect();

        let batch_len = token_ids.len();
        let seq_len = token_ids[0].len();

        // Create tensors
        let token_ids_flat: Vec<u32> = token_ids.iter().flat_map(|ids| ids.iter().copied()).collect();
        let mask_flat: Vec<u32> = attention_masks.iter().flat_map(|m| m.iter().copied()).collect();

        let token_ids_tensor = Tensor::from_vec(token_ids_flat, (batch_len, seq_len), &device)?;
        let attention_mask_tensor = Tensor::from_vec(mask_flat, (batch_len, seq_len), &device)?;
        let token_type_ids = token_ids_tensor.zeros_like()?;

        // Forward pass
        let hidden_states = model.forward(
            &token_ids_tensor,
            &token_type_ids,
            Some(&attention_mask_tensor),
        )?;

        // Mean pooling + L2 normalize
        let pooled = mean_pooling(&hidden_states, &attention_mask_tensor)?;
        let normalized = l2_normalize(&pooled)?;

        // Insert into VecDB
        for (i, page) in chunk.iter().enumerate() {
            let embedding: Vec<f32> = normalized.get(i)?.to_vec1()?;
            db.insert(page.title.clone(), embedding)?;
        }

        if (batch_idx + 1) % 50 == 0 || batch_idx + 1 == total_batches {
            let elapsed = start.elapsed().as_secs_f64();
            let done = (batch_idx + 1) * BATCH_SIZE;
            let done = done.min(pages.len());
            let rate = done as f64 / elapsed;
            println!(
                "  Batch {}/{}: {}/{} pages ({:.0} pages/s, elapsed {:.1}s)",
                batch_idx + 1, total_batches, done, pages.len(), rate, elapsed
            );
        }
    }

    let embed_time = start.elapsed();
    println!(
        "  Done! Embedded {} pages in {:.3}s ({:.0} pages/s)\n",
        db.count(),
        embed_time.as_secs_f64(),
        db.count() as f64 / embed_time.as_secs_f64()
    );

    // Phase 4: Save to disk
    println!("Phase 4: Saving to '{}'...", OUTPUT_FILE);
    let start = Instant::now();
    db.save(OUTPUT_FILE)?;
    let file_size = std::fs::metadata(OUTPUT_FILE).map(|m| m.len()).unwrap_or(0);
    println!(
        "  Saved in {:.3}s (file size: {:.2} MB)\n",
        start.elapsed().as_secs_f64(),
        file_size as f64 / 1_048_576.0
    );

    // Phase 5: Sample searches
    println!("Phase 5: Sample semantic searches\n");
    let sample_queries = [
        "capital city of a country",
        "famous scientist",
        "programming language",
        "ocean marine biology",
        "world war battle",
    ];

    for query_text in &sample_queries {
        // Embed the query
        let encoding = tokenizer
            .encode(*query_text, true)
            .map_err(|e| e.to_string())?;

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

        let results = db.search(query_vec, 5)?;

        println!("  Query: \"{}\"", query_text);
        for (rank, (id, _vec, score)) in results.iter().enumerate() {
            println!("    {}. {} (score: {:.4})", rank + 1, id, score);
        }
        println!();
    }

    println!("=== Summary ===");
    println!("Total vectors: {}", db.count());
    println!("Output file: {}", OUTPUT_FILE);

    Ok(())
}