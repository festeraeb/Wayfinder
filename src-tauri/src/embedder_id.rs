//! Which embedder produced a set of vectors, and whether two sets may be
//! mixed in one index.
//!
//! Plan for the Local embedder: a one-time hosted pass (Lemonade) for new
//! installs, then a small embedder shipped with the app, using the same model
//! so old and new vectors can live in one index. A measured test showed
//! nomic-embed-text at F16 gives identical vectors in Lemonade and in the
//! shipped runtime (cosine 1.0000); the default Q4 build does not. So vectors
//! may be mixed only when repo and quantisation match AND the quantisation is
//! one that was measured to be identical across runtimes.

/// Quantisations measured to give identical vectors across runtimes.
/// Add to this list only after measuring (cosine 1.0000 on the same text).
pub const MIXABLE_QUANTS: &[&str] = &["F16"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedderId {
    /// e.g. "nomic-ai/nomic-embed-text-v1-GGUF"
    pub repo: String,
    /// e.g. "Q4_K_S" or "F16"
    pub quant: String,
}

/// Parse a Lemonade "checkpoint" string ("<repo>:<quant>").
pub fn from_checkpoint(checkpoint: &str) -> Option<EmbedderId> {
    let _ = checkpoint;
    todo!("WF14")
}

/// True when vectors from `a` and `b` may be mixed in one index.
pub fn can_mix(a: &EmbedderId, b: &EmbedderId) -> bool {
    let _ = (a, b);
    todo!("WF14")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(repo: &str, quant: &str) -> EmbedderId {
        EmbedderId { repo: repo.to_string(), quant: quant.to_string() }
    }

    #[test]
    fn embedder_id_parses_real_lemonade_checkpoint() {
        // Exact string from GET /api/v1/models on Lemonade 2026.41.1.
        assert_eq!(
            from_checkpoint("nomic-ai/nomic-embed-text-v1-GGUF:Q4_K_S"),
            Some(id("nomic-ai/nomic-embed-text-v1-GGUF", "Q4_K_S"))
        );
        assert_eq!(
            from_checkpoint("  nomic-ai/nomic-embed-text-v1-GGUF:F16  "),
            Some(id("nomic-ai/nomic-embed-text-v1-GGUF", "F16"))
        );
    }

    #[test]
    fn embedder_id_splits_on_the_last_colon() {
        assert_eq!(from_checkpoint("a:b:Q8_0"), Some(id("a:b", "Q8_0")));
    }

    #[test]
    fn embedder_id_rejects_malformed_checkpoints() {
        for bad in ["", "   ", "no-colon", ":F16", "repo:", "repo:  ", "  :  "] {
            assert_eq!(from_checkpoint(bad), None, "input {:?}", bad);
        }
    }

    #[test]
    fn embedder_id_f16_same_repo_mixes() {
        let hosted = id("nomic-ai/nomic-embed-text-v1-GGUF", "F16");
        let shipped = id("Nomic-AI/nomic-embed-text-v1-gguf", "f16");
        assert!(can_mix(&hosted, &shipped));
        assert!(can_mix(&shipped, &hosted));
    }

    #[test]
    fn embedder_id_q4_never_mixes_even_with_itself() {
        // Q4 was measured NOT to match across runtimes.
        let q4 = id("nomic-ai/nomic-embed-text-v1-GGUF", "Q4_K_S");
        assert!(!can_mix(&q4, &q4.clone()));
        assert!(!can_mix(&q4, &id("nomic-ai/nomic-embed-text-v1-GGUF", "F16")));
    }

    #[test]
    fn embedder_id_different_models_never_mix() {
        let nomic = id("nomic-ai/nomic-embed-text-v1-GGUF", "F16");
        let gemma = id("unsloth/embeddinggemma-300m-GGUF", "F16");
        assert!(!can_mix(&nomic, &gemma));
    }

    #[test]
    fn embedder_id_mixable_list_is_only_measured_quants() {
        assert_eq!(MIXABLE_QUANTS, &["F16"]);
    }
}
