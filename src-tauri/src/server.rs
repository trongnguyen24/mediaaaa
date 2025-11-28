use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tower_http::cors::CorsLayer;
use crate::download_manager;

#[derive(Clone, Serialize, Debug)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub status: String,
    pub result_path: Option<String>,
}

#[derive(Clone)]
struct AppState {
    app: AppHandle,
    jobs: Arc<Mutex<Vec<Job>>>,
}

#[derive(Deserialize)]
struct TranscribeRequest {
    url: String,
}

#[derive(Serialize)]
struct TranscribeResponse {
    job_id: String,
    status: String,
}

pub async fn start_server(app: AppHandle) {
    let state = Arc::new(AppState { 
        app,
        jobs: Arc::new(Mutex::new(Vec::new())),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/transcribe", post(transcribe))
        .route("/api/jobs", get(get_jobs))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 14200));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": "1.0.0",
        "models_loaded": false
    }))
}

async fn get_jobs(State(state): State<Arc<AppState>>) -> Json<Vec<Job>> {
    let jobs = state.jobs.lock().unwrap();
    Json(jobs.clone())
}

async fn transcribe(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TranscribeRequest>,
) -> Json<TranscribeResponse> {
    let app_handle = state.app.clone();
    let url = payload.url.clone();
    let job_id = uuid::Uuid::new_v4().to_string();

    // Add to store
    {
        let mut jobs = state.jobs.lock().unwrap();
        jobs.push(Job {
            id: job_id.clone(),
            url: url.clone(),
            status: "queued".to_string(),
            result_path: None,
        });
    }

    let jobs_clone = state.jobs.clone();
    let job_id_clone = job_id.clone();

    tokio::spawn(async move {
        // Use a temp dir for now
        let output_dir = std::env::temp_dir(); 
        
        // Update status to processing
        {
            let mut jobs = jobs_clone.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
                job.status = "processing".to_string();
            }
        }

        let jobs_for_callback = jobs_clone.clone();
        let job_id_for_callback = job_id_clone.clone();

        match download_manager::process_job(app_handle, url, output_dir, move |status| {
            let mut jobs = jobs_for_callback.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_for_callback) {
                job.status = status;
            }
        }).await {
            Ok(path) => {
                println!("Job completed. Output at: {:?}", path);
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
                    job.status = "completed".to_string();
                    job.result_path = Some(path.to_string_lossy().to_string());
                }
            },
            Err(e) => {
                println!("Job failed: {}", e);
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id_clone) {
                    job.status = format!("failed: {}", e);
                }
            },
        }
    });

    Json(TranscribeResponse {
        job_id,
        status: "queued".to_string(),
    })
}
