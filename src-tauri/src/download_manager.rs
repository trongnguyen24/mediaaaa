use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub status: String,
}

pub async fn process_job<F>(app: AppHandle, url: String, output_dir: PathBuf, on_progress: F) -> Result<PathBuf, String> 
where
    F: Fn(String) + Send + 'static,
{
    on_progress("Starting job...".to_string());
    println!("Processing job for URL: {}", url);

    // 1. Resolve yt-dlp path
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let binaries_dir = current_dir.join("binaries"); 
    
    let yt_dlp_path = binaries_dir.join("yt-dlp-aarch64-apple-darwin");
    let ffmpeg_path = binaries_dir.join("ffmpeg-aarch64-apple-darwin");

    println!("Current dir: {:?}", current_dir);
    println!("Binaries dir: {:?}", binaries_dir);
    println!("yt-dlp path: {:?} (Exists: {})", yt_dlp_path, yt_dlp_path.exists());
    println!("ffmpeg path: {:?} (Exists: {})", ffmpeg_path, ffmpeg_path.exists());

    // 2. Run yt-dlp (Real)
    on_progress("Downloading video...".to_string());
    let id = uuid::Uuid::new_v4();
    let temp_video = output_dir.join(format!("{}.webm", id));
    let output_file = output_dir.join(format!("{}.wav", id));

    println!("Running yt-dlp...");
    let status = Command::new(&yt_dlp_path)
        .arg("-f").arg("ba[ext=webm]") // Best audio, webm format
        .arg("-o").arg(&temp_video)
        .arg(&url)
        .status()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !status.success() {
        return Err("yt-dlp failed".to_string());
    }

    // 3. Run ffmpeg (Real)
    on_progress("Converting to audio...".to_string());
    println!("Running ffmpeg...");
    let status = Command::new(&ffmpeg_path)
        .arg("-i")
        .arg(&temp_video)
        .arg("-ar").arg("16000") // 16kHz
        .arg("-ac").arg("1")     // Mono
        .arg(&output_file)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !status.success() {
        return Err("ffmpeg failed".to_string());
    }

    // 4. Check/Download Model
    on_progress("Checking AI model...".to_string());
    println!("Checking AI model...");
    let model_path = crate::model_manager::check_or_download_model(&app).await?;

    // 5. Transcribe
    on_progress("Transcribing...".to_string());
    println!("Transcribing...");
    // We need to run this on a blocking thread because whisper-rs might be heavy/blocking
    // But for now, let's just run it here.
    let transcript = crate::ai_manager::transcribe(&output_file, &model_path)?;
    
    println!("TRANSCRIPT: {}", transcript);

    Ok(output_file)
}
