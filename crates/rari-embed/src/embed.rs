use std::sync::OnceLock;

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

// Define a static OnceLock instance
static MODEL: OnceLock<TextEmbedding> = OnceLock::new();

fn get_text_embedding() -> &'static TextEmbedding {
    MODEL.get_or_init(|| {
        let options =
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true);
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
