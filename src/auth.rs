use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use crate::models::{ErrorResponse, ErrorDetail};

pub struct AuthConfig {
    pub api_key: Option<String>,
}

pub async fn auth_middleware(
    State(auth_config): State<Arc<AuthConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no auth key is configured, allow all requests
    if auth_config.api_key.is_none() {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let provided_key = auth_header.and_then(|h| h.strip_prefix("Bearer "));

    if let Some(provided) = provided_key {
        let expected_key = auth_config.api_key.as_deref().unwrap_or_default();
        
        // Use constant-time comparison to prevent timing attacks
        if provided.as_bytes().ct_eq(expected_key.as_bytes()).into() {
            // Clear the authorization header after validation
            request.headers_mut().remove(header::AUTHORIZATION);
            return Ok(next.run(request).await);
        }
    }

    // Return proper OpenAI-style error response
    let error_response = ErrorResponse {
        error: ErrorDetail {
            message: "Invalid API key".to_string(),
            error_type: "invalid_api_key".to_string(),
            code: None,
        },
    };

    let mut response = (StatusCode::UNAUTHORIZED, axum::Json(error_response)).into_response();
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer"),
    );
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_auth_config_creation() {
        let config_with_key = AuthConfig {
            api_key: Some("test-key".to_string()),
        };
        
        let config_no_key = AuthConfig {
            api_key: None,
        };
        
        assert!(config_with_key.api_key.is_some());
        assert!(config_no_key.api_key.is_none());
    }

    #[test]
    fn test_constant_time_eq() {
        use subtle::ConstantTimeEq;
        
        let key1 = "same-key";
        let key2 = "same-key";
        let key3 = "different-key";
        
        // Same keys should be equal
        assert!(key1.as_bytes().ct_eq(key2.as_bytes()).unwrap_u8() == 1);
        
        // Different keys should not be equal
        assert!(key1.as_bytes().ct_eq(key3.as_bytes()).unwrap_u8() == 0);
    }

    #[test]
    fn test_error_response_creation() {
        let error_response = ErrorResponse {
            error: ErrorDetail {
                message: "Invalid API key".to_string(),
                error_type: "invalid_api_key".to_string(),
                code: None,
            },
        };
        
        assert_eq!(error_response.error.message, "Invalid API key");
        assert_eq!(error_response.error.error_type, "invalid_api_key");
        assert!(error_response.error.code.is_none());
    }
}