# Token Counting Issue in Model2Vec Embedding Service

## Problem Summary

The current embedding service at `src/handlers.rs:118-119` uses a simplified word-based token counting approach:

```rust
// Calculate token usage (simplified - you might want to implement a proper tokenizer)
let total_tokens: usize = texts.iter().map(|t| t.split_whitespace().count()).sum();
```

This approach is inaccurate because it counts words rather than actual tokens, which can lead to significant discrepancies in usage reporting.

## Why This Matters for Model2Vec

Model2Vec models are distilled from sentence transformers and inherit their tokenization behavior. Each Model2Vec model includes a `tokenizer.json` file that defines the proper tokenization rules used during training.

The `model2vec-rs` library already loads and uses this tokenizer internally for encoding (as seen in the `StaticModel` struct), but the tokenizer is not exposed publicly.

## Technical Details

1. **Current Issue**: `split_whitespace()` counts words, not subword tokens
2. **Proper Solution**: Use the model's actual tokenizer for accurate token counting
3. **Performance Concern**: Tokenizing twice (once for counting, once for encoding) is inefficient

## Proposed Solutions

### Option 1: Expose Tokenizer
Add a public getter method to `StaticModel`:
```rust
pub fn tokenizer(&self) -> &Tokenizer
```

### Option 2: Enhanced Encoding Method
Add `encode_with_stats()` that returns both embeddings and token counts:
```rust
pub fn encode_with_stats(&self, sentences: &[String]) -> (Vec<Vec<f32>>, Vec<usize>)
```

## Implementation Strategy

1. Fork `model2vec-rs` (single file library)
2. Create main branch with tokenizer getter
3. Create secondary branch with `encode_with_stats` method
4. Submit PR to upstream project
5. Use fork in meantime via `Cargo.toml` path dependency

This approach provides immediate fix while potentially improving the upstream library for all users.