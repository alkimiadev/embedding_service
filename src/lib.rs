use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use model2vec_rs::model::StaticModel;
use std::sync::Arc;

use tower_http::{cors::CorsLayer, trace::TraceLayer, limit::RequestBodyLimitLayer};

use auth::{auth_middleware, AuthConfig};
use config::Config;
use handlers::{create_embeddings, list_models, AppState, EmbeddingModel};

// Library exports for testing
pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;

/// Create the application router for testing or production use
pub fn create_app(config: Config) -> anyhow::Result<Router> {
    // Load model
    let model = StaticModel::from_pretrained(
        &config.model_path,
        None,  // Hugging Face token
        Some(config.normalize_embeddings),  // Normalize embeddings
        None,  // Subfolder
    )?;
    
    create_app_with_model(config, model)
}

/// Create the application router with an existing model (for testing)
pub fn create_app_with_model(config: Config, model: StaticModel) -> anyhow::Result<Router> {
    
    let model_name = std::path::Path::new(&config.model_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("model2vec-{}", s))
        .unwrap_or_else(|| "model2vec-unknown".to_string());

// Create shared state
    let state = Arc::new(AppState { 
        model: Arc::new(model) as Arc<dyn EmbeddingModel>, 
        model_name,
        max_batch_size: config.max_batch_size,
        max_input_length: config.max_input_length,
    });

    // Create auth config
    let auth_config = Arc::new(AuthConfig {
        api_key: config.auth_key,
    });

    // Configure CORS
    let cors_layer = if let Some(origins) = config.cors_origins {
        let origins: Vec<_> = origins
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        
        CorsLayer::new()
            .allow_origin(origins)
            .allow_credentials(config.cors_allow_credentials)
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
    } else {
        CorsLayer::permissive()
    };

    // Build our application with routes
    let app = Router::new()
        .route("/v1/embeddings", post(create_embeddings))
        .route("/v1/models", get(list_models))
        .layer(middleware::from_fn_with_state(auth_config.clone(), auth_middleware))
        .route("/health", get(|| async { "OK" }))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(config.max_request_size_mb * 1024 * 1024))
        .layer(cors_layer)
        .with_state(state);

    Ok(app)
}