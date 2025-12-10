use reqwest::Client;
use std::net::SocketAddr;
use tokio::time::{timeout, Duration};
use embedding_service::{auth, config, handlers};
use embedding_service::models::{EmbeddingRequest, EmbeddingInput};

/// Test the actual running server with real HTTP requests
/// This test starts the server and makes real HTTP calls to it
#[tokio::test]
async fn test_e2e_embedding_request() {
    // Create a test configuration
    let config = config::Config {
        host: "127.0.0.1".to_string(),
        port: 8080, // Use port 0 to let OS assign a random free port
        model_path: "minishlab/potion-base-8M".to_string(),
        auth_key: None,
        cors_origins: None,
        cors_allow_credentials: false,
        max_batch_size: 100,
        max_input_length: 8192,
        max_request_size_mb: 8,
        normalize_embeddings: false,
    };

    // Create the app
    let app = create_test_app(config).await;
    
    // Start the server on a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind to random port");
    let addr = listener.local_addr().unwrap();
    
    // Spawn the server in the background
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test the server
    let result = test_server_endpoint(addr).await;
    
    // Shutdown the server
    server_handle.abort();
    
    // Assert the test passed
    result.unwrap();
}

async fn create_test_app(config: config::Config) -> axum::Router {
    // Load the actual model (this will be slow but tests real behavior)
    let model = model2vec_rs::model::StaticModel::from_pretrained(
        &config.model_path,
        None,  // Hugging Face token
        Some(config.normalize_embeddings),  // Normalize embeddings
        None,  // Subfolder
    ).expect("Failed to load model");
    
    let model_name = std::path::Path::new(&config.model_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("model2vec-{}", s))
        .unwrap_or_else(|| "model2vec-unknown".to_string());

    // Create shared state
    let state = std::sync::Arc::new(handlers::AppState { 
        model: std::sync::Arc::new(model), 
        model_name,
        max_batch_size: config.max_batch_size,
        max_input_length: config.max_input_length,
    });

    // Create auth config
    let auth_config = std::sync::Arc::new(auth::AuthConfig {
        api_key: config.auth_key,
    });

    // Build the application
    axum::Router::new()
        .route("/v1/embeddings", axum::routing::post(handlers::create_embeddings))
        .route("/v1/models", axum::routing::get(handlers::list_models))
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(auth_config.clone(), auth::auth_middleware))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::limit::RequestBodyLimitLayer::new(config.max_request_size_mb * 1024 * 1024))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

async fn test_server_endpoint(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = format!("http://localhost:{}", addr.port());

    // Test health endpoint
    let health_response = timeout(
        Duration::from_secs(5),
        client.get(format!("{}/health", base_url)).send()
    ).await??;
    
    assert_eq!(health_response.status(), 200);
    let health_text = health_response.text().await?;
    assert_eq!(health_text, "OK");

    // Test models endpoint
    let models_response = timeout(
        Duration::from_secs(10),
        client.get(format!("{}/v1/models", base_url)).send()
    ).await??;
    
    assert_eq!(models_response.status(), 200);
    let models_json: serde_json::Value = models_response.json().await?;
    assert_eq!(models_json["object"], "list");
    assert_eq!(models_json["data"].as_array().unwrap().len(), 1);

    // Test embedding endpoint
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: None,
    };

    let embedding_response = timeout(
        Duration::from_secs(15),
        client
            .post(format!("{}/v1/embeddings", base_url))
            .json(&request)
            .send()
    ).await??;
    
    assert_eq!(embedding_response.status(), 200);
    let embedding_json: serde_json::Value = embedding_response.json().await?;
    assert_eq!(embedding_json["object"], "list");
    assert_eq!(embedding_json["data"].as_array().unwrap().len(), 1);
    assert!(!embedding_json["data"][0]["embedding"].as_array().unwrap().is_empty());

    Ok(())
}

/// Test with authentication enabled
#[tokio::test]
async fn test_e2e_with_auth() {
    // Create a test configuration with auth
    let config = config::Config {
        host: "127.0.0.1".to_string(),
        port: 8080,
        model_path: "minishlab/potion-base-8M".to_string(),
        auth_key: Some("test-secret-key".to_string()),
        cors_origins: None,
        cors_allow_credentials: false,
        max_batch_size: 100,
        max_input_length: 8192,
        max_request_size_mb: 8,
        normalize_embeddings: false,
    };

    let app = create_test_app(config).await;
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = test_auth_endpoint(addr).await;
    
    server_handle.abort();
    
    result.unwrap();
}

async fn test_auth_endpoint(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = format!("http://localhost:{}", addr.port());

    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: None,
    };

    // Test without auth key (should fail)
    let response = timeout(
        Duration::from_secs(5),
        client
            .post(format!("{}/v1/embeddings", base_url))
            .json(&request)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 401);

    // Test with correct auth key (should succeed)
    let response = timeout(
        Duration::from_secs(15),
        client
            .post(format!("{}/v1/embeddings", base_url))
            .header("Authorization", "Bearer test-secret-key")
            .json(&request)
            .send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await?;
    assert_eq!(json["object"], "list");

    Ok(())
}