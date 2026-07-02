//! AI provider wiring for library scan filename guessing.

use std::sync::Arc;

use nest_ai::AiProvider;
use nest_ai_ollama::OllamaProvider;
use nest_error::NestResult;

use crate::config::LoonAiConfig;

/// Runtime AI services used during library scan.
#[derive(Clone)]
pub struct AiRuntime {
    /// Inference provider (`ollama`, …).
    pub provider: Arc<dyn AiProvider>,
    /// Minimum model confidence before applying a guess.
    pub min_confidence: f32,
}

impl AiRuntime {
    /// Builds AI runtime from resolved Loon settings.
    pub fn from_config(config: &LoonAiConfig) -> NestResult<Self> {
        let provider = Arc::new(OllamaProvider::new(config.ollama.clone())?) as Arc<dyn AiProvider>;
        Ok(Self {
            provider,
            min_confidence: config.min_confidence,
        })
    }
}
