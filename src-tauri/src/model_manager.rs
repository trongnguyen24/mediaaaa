use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tauri::AppHandle;
use tauri::Manager;

const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin";
const MODEL_FILENAME: &str = "ggml-tiny.bin";

pub async fn check_or_download_model(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    // In dev, resource_dir might be different. Let's use a "models" folder in the app data dir or just local for now.
    // For simplicity in this phase, let's use the current working directory's "models" folder if we are in dev.
    
    // Better approach: Use AppData directory.
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let models_dir = app_data_dir.join("models");

    if !models_dir.exists() {
        tokio::fs::create_dir_all(&models_dir).await.map_err(|e| e.to_string())?;
    }

    let model_path = models_dir.join(MODEL_FILENAME);

    if model_path.exists() {
        println!("Model found at: {:?}", model_path);
        return Ok(model_path);
    }

    println!("Downloading model to: {:?}", model_path);
    download_file(MODEL_URL, &model_path).await?;
    
    Ok(model_path)
}

async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download model: {}", response.status()));
    }

    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
    }

    println!("Model downloaded successfully.");
    Ok(())
}
