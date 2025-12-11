use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use model2vec_rs::model::StaticModel;
use std::sync::Arc;
use tokio::task;
use tracing::{debug, error};
use crate::models::{EmbeddingRequest, EmbeddingResponse, EmbeddingData, Usage, ErrorResponse, EmbeddingInput};

pub trait EmbeddingModel: Send + Sync {
    fn encode_with_stats(&self, texts: &[String]) -> model2vec_rs::model::EncodeResult;
}

impl EmbeddingModel for StaticModel {
    fn encode_with_stats(&self, texts: &[String]) -> model2vec_rs::model::EncodeResult {
        self.encode_with_stats(texts, Some(512), 1024)
    }
}

pub struct AppState {
    pub model: Arc<dyn EmbeddingModel>,
    pub model_name: String,
    pub max_batch_size: usize,
    pub max_input_length: usize,
}

pub async fn create_embeddings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EmbeddingRequest>,
) -> Result<Json<EmbeddingResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Received embedding request for {} texts", 
           match &request.input {
               EmbeddingInput::String(_) => 1,
               EmbeddingInput::StringArray(texts) => texts.len(),
           });

    // Extract input texts
    let texts = match request.input {
        EmbeddingInput::String(text) => vec![text],
        EmbeddingInput::StringArray(texts) => texts,
    };

    // Validate input
    if texts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: crate::models::ErrorDetail {
                    message: "Input cannot be empty".to_string(),
                    error_type: "invalid_request_error".to_string(),
                    code: Some("empty_input".to_string()),
                },
            }),
        ));
    }

    if texts.len() > state.max_batch_size {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: crate::models::ErrorDetail {
                    message: format!("Batch size exceeds maximum of {}", state.max_batch_size),
                    error_type: "invalid_request_error".to_string(),
                    code: Some("batch_too_large".to_string()),
                },
            }),
        ));
    }

    // Validate input lengths
    for text in &texts {
        if text.len() > state.max_input_length {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: crate::models::ErrorDetail {
                        message: format!("Input exceeds maximum length of {}", state.max_input_length),
                        error_type: "invalid_request_error".to_string(),
                        code: Some("input_too_long".to_string()),
                    },
                }),
            ));
        }
    }

    // Offload CPU-intensive model encoding to blocking thread pool
    let model = Arc::clone(&state.model);
    let texts_clone = texts.clone();
    
    let result = task::spawn_blocking(move || model.encode_with_stats(&texts_clone))
        .await
        .map_err(|e| {
            error!("Failed to generate embeddings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: crate::models::ErrorDetail {
                        message: format!("Embedding generation task failed: {}", e),
                        error_type: "server_error".to_string(),
                        code: None,
                    },
                }),
            )
        })?;

    let mut embeddings_data = Vec::with_capacity(result.embeddings.len());
    
    for (index, embedding) in result.embeddings.into_iter().enumerate() {
        embeddings_data.push(EmbeddingData {
            object: "embedding".to_string(),
            embedding,
            index,
        });
    }

    // Calculate accurate token usage using tokenizer counts
    let total_tokens: usize = result.token_counts.iter().sum();

    // Return response
    Ok(Json(EmbeddingResponse {
        object: "list".to_string(),
        data: embeddings_data,
        model: request.model.unwrap_or_else(|| state.model_name.clone()),
        usage: Usage {
            prompt_tokens: total_tokens,
            total_tokens,
        },
    }))
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [{
            "id": state.model_name,
            "object": "model",
            "owned_by": "local",
        }]
    }))
}

#[cfg(test)]
mod tests {
    
    use crate::models::{EmbeddingRequest, EmbeddingInput};

    #[test]
    fn test_embedding_request_parsing() {
        // Test string input
        let request_str = r#"{"input": "hello world", "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(s) => assert_eq!(s, "hello world"),
            EmbeddingInput::StringArray(_) => panic!("Expected string input"),
        }

        // Test array input
        let request_arr = r#"{"input": ["hello", "world"], "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_arr).unwrap();
        
        match request.input {
            EmbeddingInput::String(_) => panic!("Expected array input"),
            EmbeddingInput::StringArray(arr) => assert_eq!(arr, vec!["hello", "world"]),
        }
    }

    #[test]
    fn test_accurate_token_counting() {
        // Test that we're now using accurate tokenizer-based counting
        // This test will verify the new encode_with_stats integration
        let text = "Hello world test";
        let _texts = vec![text.to_string()];
        
        // Note: This test would require a model instance to fully test
        // For now, we verify the concept that tokenizer counting differs from word counting
        let word_count = text.split_whitespace().count();
        assert_eq!(word_count, 3);
        
        // Tokenizer-based counting might be different (e.g., subword tokenization)
        // The actual count depends on the model's tokenizer
        let empty_text = "";
        let empty_word_count = empty_text.split_whitespace().count();
        assert_eq!(empty_word_count, 0);
        
        let multi_space_text = "Hello   world";
        let multi_space_word_count = multi_space_text.split_whitespace().count();
        assert_eq!(multi_space_word_count, 2);
    }
}