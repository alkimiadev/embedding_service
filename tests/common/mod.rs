pub mod mock_model;

use embedding_service::config::Config;

use mock_model::MockModel;

pub fn create_test_server(with_auth: bool) -> axum::Router {
    let config = Config {
        model_path: "test-model.gguf".to_string(),
        auth_key: if with_auth { Some("test-key".to_string()) } else { None },
        host: "127.0.0.1".to_string(),
        port: 8080,
        cors_origins: Some("http://localhost:3000".to_string()),
        cors_allow_credentials: true,
        max_batch_size: 100,
        max_input_length: 8192,
        max_request_size_mb: 8,
        normalize_embeddings: false,
    };
    
    create_test_app(config)
}

pub fn create_test_server_with_config(
    max_batch_size: usize,
    max_input_length: usize,
    auth_key: Option<String>,
) -> axum::Router {
    let config = Config {
        model_path: "test-model.gguf".to_string(),
        auth_key,
        host: "127.0.0.1".to_string(),
        port: 8080,
        cors_origins: Some("http://localhost:3000".to_string()),
        cors_allow_credentials: true,
        max_batch_size,
        max_input_length,
        max_request_size_mb: 8,
        normalize_embeddings: false,
    };
    
    create_test_app(config)
}

fn create_test_app(config: Config) -> axum::Router {
    use embedding_service::{handlers, auth};
    use std::sync::Arc;
    use axum::{routing::{get, post}, Router};
    use tower_http::{cors::CorsLayer, trace::TraceLayer, limit::RequestBodyLimitLayer};
    
    // Create mock model
    let mock_model = MockModel::new();
    
    let model_name = "test-model".to_string();

    // Create shared state - note we're using MockModel as trait object
    let state = Arc::new(handlers::AppState { 
        model: Arc::new(mock_model) as Arc<dyn handlers::EmbeddingModel>, 
        model_name,
        max_batch_size: config.max_batch_size,
        max_input_length: config.max_input_length,
    });

    // Create auth config
    let auth_config = Arc::new(auth::AuthConfig {
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

    // Build the application
    Router::new()
        .route("/v1/embeddings", post(handlers::create_embeddings))
        .route("/v1/models", get(handlers::list_models))
        .route("/health", get(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(auth_config.clone(), auth::auth_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(config.max_request_size_mb * 1024 * 1024))
        .layer(cors_layer)
        .with_state(state)
}