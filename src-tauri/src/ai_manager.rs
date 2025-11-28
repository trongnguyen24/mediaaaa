use std::path::PathBuf;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

pub fn transcribe<F>(audio_path: &PathBuf, model_path: &PathBuf, on_progress: F) -> Result<String, String> 
where
    F: Fn(i32) + Send + Sync + 'static,
{
    println!("Loading model from {:?}", model_path);
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default()
    ).map_err(|e| format!("Failed to load model: {:?}", e))?;

    let mut state = ctx.create_state().map_err(|e| format!("Failed to create state: {:?}", e))?;

    // Load audio data
    println!("Loading audio from {:?}", audio_path);
    let mut reader = hound::WavReader::open(audio_path).map_err(|e| format!("Failed to open WAV file: {:?}", e))?;
    
    // Convert to 32-bit float samples
    // Whisper expects 16kHz mono. We assume the input is correct.
    let samples: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();

    if samples.is_empty() {
        return Err("Audio file is empty".to_string());
    }

    println!("Starting transcription on {} samples...", samples.len());
    
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("vi")); // Default to Vietnamese
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Setup progress callback
    // params.set_progress_callback_safe(move |progress| {
    //     on_progress(progress);
    // });

    // Run whisper
    state.full(params, &samples[..]).map_err(|e| format!("Failed to run whisper: {:?}", e))?;

    let num_segments = state.full_n_segments().map_err(|e| format!("Failed to get segments: {:?}", e))?;
    let mut full_text = String::new();

    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i).map_err(|e| format!("Failed to get segment text: {:?}", e))?;
        let t0 = state.full_get_segment_t0(i).map_err(|e| format!("Failed to get start time: {:?}", e))?;
        let t1 = state.full_get_segment_t1(i).map_err(|e| format!("Failed to get end time: {:?}", e))?;
        
        let start = format_timestamp(t0);
        let end = format_timestamp(t1);
        
        full_text.push_str(&format!("[{} -> {}] {}\n", start, end, segment.trim()));
    }

    Ok(full_text.trim().to_string())
}

fn format_timestamp(t: i64) -> String {
    let seconds = t / 100;
    let minutes = seconds / 60;
    let seconds_remainder = seconds % 60;
    format!("{}:{:02}", minutes, seconds_remainder)
}
