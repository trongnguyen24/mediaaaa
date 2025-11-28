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
    F: Fn(String) + Send + Sync + 'static,
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
    on_progress("Downloading video... 0%".to_string());
    let id = uuid::Uuid::new_v4();
    let temp_video = output_dir.join(format!("{}.webm", id));
    let output_file = output_dir.join(format!("{}.wav", id));

    println!("Running yt-dlp...");
    let mut child = Command::new(&yt_dlp_path)
        .arg("--newline") // Force progress to be on new lines
        .arg("-f").arg("ba[ext=webm]") // Best audio, webm format
        .arg("-o").arg(&temp_video)
        .arg(&url)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = std::io::BufReader::new(stdout);
        use std::io::BufRead;
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Parse progress: [download]  45.6% of ...
                    if line.starts_with("[download]") && line.contains('%') {
                        if let Some(pct_idx) = line.find('%') {
                            // Find the number before %
                            let parts: Vec<&str> = line[..pct_idx].split_whitespace().collect();
                            if let Some(last) = parts.last() {
                                on_progress(format!("Downloading video... {}%", last));
                            }
                        }
                    }
                }
                Err(e) => println!("Error reading stdout: {}", e),
            }
        }
    }

    let status = child.wait().map_err(|e| format!("Failed to wait on yt-dlp: {}", e))?;

    if !status.success() {
        return Err("yt-dlp failed".to_string());
    }

    // 3. Run ffmpeg (Real)
    on_progress("Converting to audio... 0%".to_string());
    println!("Running ffmpeg...");
    let mut child = Command::new(&ffmpeg_path)
        .arg("-i")
        .arg(&temp_video)
        .arg("-ar").arg("16000") // 16kHz
        .arg("-ac").arg("1")     // Mono
        .arg(&output_file)
        .stderr(std::process::Stdio::piped()) // ffmpeg writes to stderr
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    if let Some(stderr) = child.stderr.take() {
        let reader = std::io::BufReader::new(stderr);
        use std::io::BufRead;
        
        let mut total_duration_secs = 0.0;

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Parse Duration: Duration: 00:00:30.50, ...
                    if line.contains("Duration:") {
                        if let Some(start) = line.find("Duration: ") {
                            let s = &line[start + 10..];
                            if let Some(end) = s.find(',') {
                                let duration_str = &s[..end];
                                total_duration_secs = parse_duration(duration_str);
                            }
                        }
                    }

                    // Parse time: time=00:00:05.20
                    if line.contains("time=") {
                        if let Some(start) = line.find("time=") {
                            let s = &line[start + 5..];
                            let time_str = s.split_whitespace().next().unwrap_or("");
                            let current_secs = parse_duration(time_str);
                            
                            if total_duration_secs > 0.0 {
                                let pct = (current_secs / total_duration_secs * 100.0) as i32;
                                on_progress(format!("Converting to audio... {}%", pct));
                            }
                        }
                    }
                }
                Err(e) => println!("Error reading stderr: {}", e),
            }
        }
    }

    let status = child.wait().map_err(|e| format!("Failed to wait on ffmpeg: {}", e))?;

    if !status.success() {
        return Err("ffmpeg failed".to_string());
    }

    // 4. Check/Download Model
    on_progress("Checking AI model...".to_string());
    println!("Checking AI model...");
    let model_path = crate::model_manager::check_or_download_model(&app).await?;

    // 5. Transcribe
    on_progress("Transcribing... 0%".to_string());
    println!("Transcribing...");
    
    let model_path_clone = model_path.clone();
    let output_file_clone = output_file.clone();
    
    // We need to move on_progress into the blocking task.
    // Since on_progress is F, and F is Fn, we can't move it if it's not Clone?
    // F is a type parameter. We can't clone it unless F: Clone.
    // But we can only move it once.
    // So we move it into the spawn_blocking closure.
    
    let transcript = tokio::task::spawn_blocking(move || {
        crate::ai_manager::transcribe(&output_file_clone, &model_path_clone, move |progress| {
            on_progress(format!("Transcribing... {}%", progress));
        })
    }).await.map_err(|e| format!("Task join error: {}", e))??;
    
    println!("TRANSCRIPT: {}", transcript);

    Ok(output_file)
}

fn parse_duration(time_str: &str) -> f64 {
    // Format: HH:MM:SS.mm
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].parse().unwrap_or(0.0);
        let minutes: f64 = parts[1].parse().unwrap_or(0.0);
        let seconds: f64 = parts[2].parse().unwrap_or(0.0);
        return hours * 3600.0 + minutes * 60.0 + seconds;
    }
    0.0
}
