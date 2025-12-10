use embedding_service::handlers::EmbeddingModel;

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
    fn encode(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|text| self.generate_embedding(text))
            .collect()
    }
}