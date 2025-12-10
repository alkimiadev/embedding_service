use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Host to bind to
    // Use 'H' instead of the default 'h' to avoid conflict with --help
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Model ID from Hugging Face or local path to model directory
    #[arg(short, long, default_value = "minishlab/potion-base-8M")]
    pub model_path: String,

    /// API key for authentication
    #[arg(short, long)]
    pub auth_key: Option<String>,

    /// CORS origins to allow (comma-separated). If not specified, allows all origins
    #[arg(long)]
    pub cors_origins: Option<String>,

    /// Whether to allow credentials in CORS requests
    #[arg(long, default_value = "false")]
    pub cors_allow_credentials: bool,

    /// Maximum batch size for embedding requests
    #[arg(long, default_value = "100")]
    pub max_batch_size: usize,

    /// Maximum input length per text (characters)
    #[arg(long, default_value = "8192")]
    pub max_input_length: usize,

    /// Request body size limit in MB
    #[arg(long, default_value = "8")]
    pub max_request_size_mb: usize,

    /// Whether to normalize embeddings
    #[arg(long, default_value = "false")]
    pub normalize_embeddings: bool,
}