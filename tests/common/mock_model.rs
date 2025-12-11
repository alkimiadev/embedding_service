use embedding_service::handlers::EmbeddingModel;
use model2vec_rs::model::EncodeResult;

pub struct MockModel;

impl MockModel {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_embedding(&self, text: &str) -> Vec<f32> {
        // Generate deterministic mock embeddings based on text content
        let size = 384; // Typical embedding size
        let hash = text.chars().map(|c| c as u32).sum::<u32>() as f32;
        (0..size).map(|i| {
            let base = i as f32 / size as f32;
            let variation = ((hash as u32 + i as u32) % 100) as f32 / 100.0;
            base + variation * 0.1
        }).collect()
    }
}

impl EmbeddingModel for MockModel {
    fn encode_with_stats(&self, texts: &[String]) -> EncodeResult {
        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .map(|text| self.generate_embedding(text))
            .collect();
        
        // Mock token counting - simple word-based for testing
        let token_counts: Vec<usize> = texts
            .iter()
            .map(|text| text.split_whitespace().count())
            .collect();
        
        EncodeResult {
            embeddings,
            token_counts,
        }
    }
}