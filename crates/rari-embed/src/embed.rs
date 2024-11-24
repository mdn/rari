use std::sync::OnceLock;

use anyhow::Result;
use fastembed::{EmbeddingModel, ExecutionProviderDispatch, InitOptions, TextEmbedding};
use ort::execution_providers::CoreMLExecutionProvider;

// Define a static OnceLock instance
static MODEL: OnceLock<TextEmbedding> = OnceLock::new();

fn get_text_embedding() -> &'static TextEmbedding {
    MODEL.get_or_init(|| {
        let epd: ExecutionProviderDispatch = ExecutionProviderDispatch::from(
            CoreMLExecutionProvider::default()
                .with_ane_only()
                .with_subgraphs(),
        )
        .error_on_failure();

        // BGESmallENV15
        let options = InitOptions::new(EmbeddingModel::AllMiniLML6V2Q)
            .with_show_download_progress(true)
            .with_execution_providers(vec![epd]);
        TextEmbedding::try_new(options).unwrap()
    })
}

pub fn embeds(texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let embeddings = get_text_embedding().embed(texts, None)?;
    Ok(embeddings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embeds() {
        let texts = vec!["hello world".to_string(), "goodbye world".to_string()];
        let embeddings = embeds(texts).unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
    }
}
