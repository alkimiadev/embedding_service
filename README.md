# Embedding Service

A fast, lightweight embedding service built with Rust that provides OpenAI-compatible API endpoints for generating text embeddings using model2vec models.

## Features

- 🚀 **High Performance**: Built with Rust and Axum for maximum speed
- 📝 **OpenAI Compatible**: Drop-in replacement for OpenAI's embedding API
- 🔐 **API Key Authentication**: Optional API key-based authentication with constant-time comparison
- 🌐 **Configurable CORS**: Flexible cross-origin resource sharing
- 📊 **Health Endpoint**: Built-in health check endpoint (no auth required)
- 📋 **Flexible Input**: Supports both single strings and arrays of strings
- 🛡️ **Production Ready**: Input validation, rate limiting, graceful shutdown
- 🔍 **Comprehensive Logging**: Request/response tracing with structured logs
- ⚡ **Non-blocking**: CPU-intensive model operations offloaded to thread pool

## Quick Start

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd embedding_service

# Build the project
cargo build --release

# Run with default settings
./target/release/embedding_service
```

### Usage

Start the service with custom configuration:

```bash
# Basic usage
./target/release/embedding_service --host 0.0.0.0 --port 8080

# With custom model
./target/release/embedding_service --model-path minishlab/potion-base-8M

# With API key authentication
./target/release/embedding_service --auth-key your-secret-api-key
```

## API Endpoints

### Generate Embeddings

**POST** `/v1/embeddings`

Generate embeddings for input text(s).

### List Models

**GET** `/v1/models`

List available embedding models (OpenAI API compatibility).

### Health Check

**GET** `/health`

Returns "OK" if service is running. No authentication required.

#### Request Body

```json
{
  "input": "Your text here",
  "model": "model2vec-potion-base-8M"  // optional
}
```

Or with multiple inputs:

```json
{
  "input": ["Text 1", "Text 2", "Text 3"],
  "model": "model2vec-potion-base-8M"  // optional
}
```

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.1, 0.2, 0.3, ...],
      "index": 0
    }
  ],
  "model": "model2vec-potion-base-8M",
  "usage": {
    "prompt_tokens": 3,
    "total_tokens": 3
  }
}
```

### Health Check

**GET** `/health`

Returns "OK" if the service is running.

## Configuration Options

| Option | Short | Long | Default | Description |
|--------|-------|------|---------|-------------|
| Host | `-H` | `--host` | `127.0.0.1` | Host to bind to |
| Port | `-p` | `--port` | `8080` | Port to bind to |
| Model Path | `-m` | `--model-path` | `minishlab/potion-base-8M` | Model ID or local path |
| Auth Key | `-a` | `--auth-key` | `None` | API key for authentication |
| CORS Origins | | `--cors-origins` | `None` (allow all) | Comma-separated allowed origins |
| CORS Credentials | | `--cors-allow-credentials` | `false` | Allow credentials in CORS requests |
| Max Batch Size | | `--max-batch-size` | `100` | Maximum batch size for requests |
| Max Input Length | | `--max-input-length` | `8192` | Max characters per text input |
| Max Request Size | | `--max-request-size-mb` | `8` | Request body size limit (MB) |
| Normalize Embeddings | | `--normalize-embeddings` | `false` | Whether to normalize embeddings |

### Examples

```bash
# Basic usage
./target/release/embedding_service --host 0.0.0.0 --port 8080

# With authentication and CORS restrictions
./target/release/embedding_service \
  --auth-key your-secret-key \
  --cors-origins "https://example.com,https://app.example.com" \
  --cors-allow-credentials

# Production settings with limits
./target/release/embedding_service \
  --max-batch-size 50 \
  --max-input-length 4096 \
  --max-request-size-mb 4 \
  --normalize-embeddings

# Custom model
./target/release/embedding_service --model-path /path/to/local/model
```

## Authentication

If an `--auth-key` is provided, all requests must include an `Authorization` header:

```bash
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Authorization: Bearer your-secret-api-key" \
  -H "Content-Type: application/json" \
  -d '{"input": "Hello, world!"}'
```

## Example Usage with curl

```bash
# Basic embedding request
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input": "Hello, world!"}'

# Multiple texts
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input": ["Hello", "World", "Embeddings"]}'

# With authentication
curl -X POST http://localhost:8080/v1/embeddings \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"input": "Authenticated request"}'
```

## Development

### Running in Development

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with custom arguments
cargo run -- --host 0.0.0.0 --port 3000
```

### Project Structure

```
src/
├── main.rs      # Application entry point
├── config.rs    # Configuration management
├── handlers.rs  # HTTP request handlers
├── auth.rs      # Authentication middleware
└── models.rs    # Data models and types
```

## Dependencies

- **axum**: Web framework
- **model2vec-rs**: Embedding model inference
- **tokio**: Async runtime
- **clap**: Command-line argument parsing
- **serde**: Serialization/deserialization
- **tracing**: Structured logging
- **tower-http**: HTTP middleware (CORS, tracing, rate limiting)
- **subtle**: Constant-time cryptographic operations (security)

## Security Features

- **Constant-time API key comparison** (prevents timing attacks)
- **Input validation** (prevents DoS attacks)
- **Request size limits** (prevents resource exhaustion)
- **Configurable CORS** (reduces attack surface)
- **Health endpoint isolation** (monitoring tools work without auth)

## Performance Features

- **Non-blocking operations**: CPU-intensive model encoding offloaded to thread pool
- **Batch processing**: Efficient handling of multiple texts
- **Configurable limits**: Tune for your hardware and use case
- **Graceful shutdown**: Clean handling of signals without dropping requests

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🤖 Attribution & Development Process

This project was created through a collaborative AI-assisted development process:

### **Initial Development**
- **Conception & Direction**: @alkimiadev
- **Initial Implementation**: GLM-4.6 (via z.ai interface)
- **Base Architecture**: OpenAI-compatible embeddings API using model2vec-rs

### **Production Refactoring**
- **Security & Performance**: GLM-4.6 (via opencode cli interface)
- **Code Review Analysis**: 4 advanced AI coding agents via OpenRouter(Claude Sonnet 4.5, Gemini 2.5 pro, Grok Code Fast 1, GPT 5)
- **Critical Improvements**: 
  - Security vulnerabilities (timing attacks, auth bypass)
  - Performance optimizations (async runtime, blocking operations)
  - Production readiness (error handling, logging, testing)
  - API compatibility enhancements

### **Development Methodology**
This project demonstrates modern AI-assisted development:
1. **Rapid Prototyping**: Initial working version in single session
2. **Multi-Agent Review**: Comprehensive security and architecture analysis
3. **Iterative Improvement**: Systematic addressing of all identified issues
4. **Production Hardening**: Security, performance, and reliability focus

### **Human Oversight**
All AI contributions were directed, reviewed, and validated by human oversight to ensure:
- Alignment with project goals
- Security best practices
- Production readiness standards
- Ethical development practices

### **Development Efficiency**
- **Traditional Timeline**: ~2 weeks solo development
- **AI-Assisted Timeline**: ~2 hours total
- **Cost**: $0 (existing AI coding plans)
- **Result**: Production-ready service with comprehensive security and performance optimizations
