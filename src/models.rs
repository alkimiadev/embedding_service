use serde::{Deserialize, Serialize};

// Request structure mimicking OpenAI's embeddings API
#[derive(Debug, Deserialize, Serialize)]
pub struct EmbeddingRequest {
    pub input: EmbeddingInput,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

// Response structure mimicking OpenAI's embeddings API
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_embedding_request_string_input() {
        let request_str = r#"{"input": "hello world", "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(s) => assert_eq!(s, "hello world"),
            EmbeddingInput::StringArray(_) => panic!("Expected string input"),
        }
        assert_eq!(request.model, Some("test".to_string()));
    }

    #[test]
    fn test_embedding_request_array_input() {
        let request_str = r#"{"input": ["hello", "world"], "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(_) => panic!("Expected array input"),
            EmbeddingInput::StringArray(arr) => assert_eq!(arr, vec!["hello", "world"]),
        }
        assert_eq!(request.model, Some("test".to_string()));
    }

    #[test]
    fn test_embedding_request_optional_model() {
        let request_str = r#"{"input": "hello world"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(s) => assert_eq!(s, "hello world"),
            EmbeddingInput::StringArray(_) => panic!("Expected string input"),
        }
        assert_eq!(request.model, None);
    }

    #[test]
    fn test_embedding_response_serialization() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![
                EmbeddingData {
                    object: "embedding".to_string(),
                    embedding: vec![0.1, 0.2, 0.3],
                    index: 0,
                }
            ],
            model: "test-model".to_string(),
            usage: Usage {
                prompt_tokens: 2,
                total_tokens: 2,
            },
        };
        
        let json_str = serde_json::to_string(&response).unwrap();
        let parsed: EmbeddingResponse = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed.object, "list");
        assert_eq!(parsed.data.len(), 1);
        assert_eq!(parsed.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(parsed.model, "test-model");
        assert_eq!(parsed.usage.prompt_tokens, 2);
        assert_eq!(parsed.usage.total_tokens, 2);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: ErrorDetail {
                message: "Invalid API key".to_string(),
                error_type: "invalid_api_key".to_string(),
                code: Some("invalid_key".to_string()),
            },
        };
        
        let json_str = serde_json::to_string(&error).unwrap();
        let parsed: ErrorResponse = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed.error.message, "Invalid API key");
        assert_eq!(parsed.error.error_type, "invalid_api_key");
        assert_eq!(parsed.error.code, Some("invalid_key".to_string()));
    }

    #[test]
    fn test_error_response_without_code() {
        let error = ErrorResponse {
            error: ErrorDetail {
                message: "Internal server error".to_string(),
                error_type: "server_error".to_string(),
                code: None,
            },
        };
        
        let json_str = serde_json::to_string(&error).unwrap();
        let parsed: ErrorResponse = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed.error.message, "Internal server error");
        assert_eq!(parsed.error.error_type, "server_error");
        assert_eq!(parsed.error.code, None);
    }

    #[test]
    fn test_empty_array_input() {
        let request_str = r#"{"input": [], "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(_) => panic!("Expected array input"),
            EmbeddingInput::StringArray(arr) => assert_eq!(arr, Vec::<String>::new()),
        }
    }

    #[test]
    fn test_empty_string_input() {
        let request_str = r#"{"input": "", "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(s) => assert_eq!(s, ""),
            EmbeddingInput::StringArray(_) => panic!("Expected string input"),
        }
    }

    #[test]
    fn test_unicode_handling() {
        let request_str = r#"{"input": "Hello 世界 🌍", "model": "test"}"#;
        let request: EmbeddingRequest = serde_json::from_str(request_str).unwrap();
        
        match request.input {
            EmbeddingInput::String(s) => assert_eq!(s, "Hello 世界 🌍"),
            EmbeddingInput::StringArray(_) => panic!("Expected string input"),
        }
    }
}