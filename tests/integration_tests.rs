mod common;

use axum_test::TestServer;
use embedding_service::models::{EmbeddingRequest, EmbeddingInput};
use common::{create_test_server, create_test_server_with_config};
use serial_test::serial;
use axum_test::http::StatusCode;

#[tokio::test]
#[serial]
async fn test_health_endpoint() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let response = server.get("/health").await;
    
    response.assert_status_ok();
    assert_eq!(response.text(), "OK");
}

#[tokio::test]
#[serial]
async fn test_list_models_no_auth() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let response = server.get("/v1/models").await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["object"], "list");
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
    assert_eq!(json["data"][0]["id"], "test-model");
    assert_eq!(json["data"][0]["object"], "model");
    assert_eq!(json["data"][0]["owned_by"], "local");
}

#[tokio::test]
#[serial]
async fn test_batch_embedding_no_auth() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::StringArray(vec![
            "First text".to_string(),
            "Second text".to_string(),
        ]),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["object"], "list");
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"][0]["index"], 0);
    assert_eq!(json["data"][1]["index"], 1);
    assert_eq!(json["usage"]["prompt_tokens"], 4); // 2 words each
    assert_eq!(json["usage"]["total_tokens"], 4);
}

#[tokio::test]
#[serial]
async fn test_embedding_with_valid_auth() {
    let server = TestServer::new(create_test_server(true)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: Some("test-model".to_string()),
    };
    
    let response = server
        .post("/v1/embeddings")
        .add_header("Authorization", "Bearer test-key")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["object"], "list");
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
#[serial]
async fn test_embedding_with_invalid_auth() {
    let server = TestServer::new(create_test_server(true)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: Some("test-model".to_string()),
    };
    
    let response = server
        .post("/v1/embeddings")
        .add_header("Authorization", "Bearer wrong-key")
        .json(&request)
        .await;
    
    response.assert_status(StatusCode::UNAUTHORIZED);
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"]["message"], "Invalid API key");
    assert_eq!(json["error"]["type"], "invalid_api_key");
}

#[tokio::test]
#[serial]
async fn test_embedding_missing_auth() {
    let server = TestServer::new(create_test_server(true)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status(StatusCode::UNAUTHORIZED);
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"]["message"], "Invalid API key");
    assert_eq!(json["error"]["type"], "invalid_api_key");
}

#[tokio::test]
#[serial]
async fn test_empty_input_array() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::StringArray(vec![]),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status(StatusCode::BAD_REQUEST);
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"]["message"], "Input cannot be empty");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "empty_input");
}

#[tokio::test]
#[serial]
async fn test_oversized_batch() {
    let server = TestServer::new(create_test_server_with_config(
        2, // max_batch_size
        8192, // max_input_length
        None, // no auth
    )).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::StringArray(vec![
            "text1".to_string(),
            "text2".to_string(),
            "text3".to_string(), // Exceeds batch size of 2
        ]),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status(StatusCode::BAD_REQUEST);
    
    let json: serde_json::Value = response.json();
    assert!(json["error"]["message"].as_str().unwrap().contains("Batch size exceeds maximum of 2"));
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "batch_too_large");
}

#[tokio::test]
#[serial]
async fn test_oversized_input_length() {
    let server = TestServer::new(create_test_server_with_config(
        100, // max_batch_size
        10, // max_input_length
        None, // no auth
    )).unwrap();
    
    let long_text = "a".repeat(20);
    let request = EmbeddingRequest {
        input: EmbeddingInput::String(long_text),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status(StatusCode::BAD_REQUEST);
    
    let json: serde_json::Value = response.json();
    assert!(json["error"]["message"].as_str().unwrap().contains("Input exceeds maximum length of 10"));
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "input_too_long");
}

#[tokio::test]
#[serial]
async fn test_malformed_json() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let response = server
        .post("/v1/embeddings")
        .text("{ invalid json }")
        .await;
    
    // Axum returns 415 (Unsupported Media Type) for malformed JSON
    response.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
#[serial]
async fn test_missing_input_field() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = serde_json::json!({
        "model": "test-model"
        // Missing "input" field
    });
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    // Axum returns 422 (Unprocessable Entity) for missing required fields
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[serial]
async fn test_unicode_text_handling() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello 世界 🌍".to_string()),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
    assert!(!json["data"][0]["embedding"].as_array().unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn test_empty_string_input() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("".to_string()),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
    assert_eq!(json["usage"]["prompt_tokens"], 0); // Empty string -> 0 words
    assert_eq!(json["usage"]["total_tokens"], 0);
}

#[tokio::test]
#[serial]
async fn test_default_model_when_not_specified() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::String("Hello world".to_string()),
        model: None, // No model specified
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["model"], "test-model"); // Should use default model name
}

#[tokio::test]
#[serial]
async fn test_cors_headers() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    // Test CORS with a POST request instead since axum-test doesn't support OPTIONS
    let response = server
        .post("/v1/embeddings")
        .add_header("Origin", "http://localhost:3000")  // Must match configured CORS origin
        .json(&serde_json::json!({
            "input": "test",
            "model": "test-model"
        }))
        .await;
    
    response.assert_status(StatusCode::OK);
    // Check CORS headers are present
    assert!(response.headers().get("access-control-allow-origin").is_some());
}

#[tokio::test]
#[serial]
async fn test_large_valid_batch() {
    let server = TestServer::new(create_test_server(false)).unwrap();
    
    // Create a batch of 50 texts (within default limit of 100)
    let texts: Vec<String> = (0..50)
        .map(|i| format!("Test text number {}", i))
        .collect();
    
    let request = EmbeddingRequest {
        input: EmbeddingInput::StringArray(texts),
        model: Some("test-model".to_string()),
    };
    
    let response = server.post("/v1/embeddings").json(&request).await;
    
    response.assert_status_ok();
    
    let json: serde_json::Value = response.json();
    assert_eq!(json["data"].as_array().unwrap().len(), 50);
    assert_eq!(json["usage"]["prompt_tokens"], 200); // 4 words per text ("Test text number N") * 50 texts
    assert_eq!(json["usage"]["total_tokens"], 200);
}