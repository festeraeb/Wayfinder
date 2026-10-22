//! Per-request records for opt-in MoE research: numeric codes only, day-only time, randomised label, shuffled batch.

/// A single opt-in research record.
///
/// `day` is a `YYYY-MM-DD` date only (never a timestamp), `category` is a
/// numeric request-type code, `experts` are the activated expert indices and
/// `tokens` is the token count. No text, identity or exact time is stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub day: String,
    pub category: u8,
    pub experts: Vec<u16>,
    pub tokens: u32,
}

/// Randomized-response label for a request.
///
/// With probability `keep_num / keep_den` the true category is reported;
/// otherwise a uniformly random category in `0..n_categories` is reported
/// (which may coincidentally equal the true one). `r_keep` and `r_pick` are
/// caller-supplied random numbers, so this function stays pure.
///
/// Degenerate parameters (`keep_den == 0` or `n_categories == 0`) fall back to
/// the true category.
pub fn randomize_category(
    true_category: u8,
    n_categories: u8,
    keep_num: u32,
    keep_den: u32,
    r_keep: u32,
    r_pick: u32,
) -> u8 {
    if keep_den == 0 || n_categories == 0 {
        return true_category;
    }
    if r_keep % keep_den < keep_num {
        true_category
    } else {
        (r_pick % n_categories as u32) as u8
    }
}

/// Unbiased estimate of the true per-category counts from randomized-response
/// observations.
///
/// For each category `k`, `true_k = (observed_k - (1 - keep_prob) * N / n) / keep_prob`
/// where `N` is the total number of observations and `n` the number of
/// categories. Returns the observed counts unchanged when `keep_prob <= 0.0`
/// or the slice is empty.
pub fn estimate_true_counts(observed: &[u64], keep_prob: f64) -> Vec<f64> {
    let n = observed.len();
    if keep_prob <= 0.0 || n == 0 {
        return observed.iter().map(|&c| c as f64).collect();
    }
    let total = observed.iter().sum::<u64>() as f64;
    let n = n as f64;
    observed
        .iter()
        .map(|&c| (c as f64 - (1.0 - keep_prob) * total / n) / keep_prob)
        .collect()
}

/// Deterministic splitmix64 generator: pure, no OS randomness.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Shuffle `records` in place with a Fisher-Yates pass driven by splitmix64.
///
/// The result is fully determined by `seed`; no clock or OS randomness is used.
pub fn shuffle_batch(records: &mut [Record], seed: u64) {
    let mut rng = SplitMix64::new(seed);
    for i in (1..records.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        records.swap(i, j);
    }
}

/// Encode records as compact JSON, one object per line, joined with `'\n'`
/// (no trailing newline).
///
/// Written by hand (no serde). Only ASCII digits and `-` survive in the `day`
/// field, so no other character can leak through it.
pub fn encode_batch(records: &[Record]) -> String {
    let mut out = String::new();
    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str("{\"d\":\"");
        for ch in record.day.chars() {
            if ch.is_ascii_digit() || ch == '-' {
                out.push(ch);
            }
        }
        out.push_str("\",\"c\":");
        out.push_str(&record.category.to_string());
        out.push_str(",\"e\":[");
        for (j, expert) in record.experts.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&expert.to_string());
        }
        out.push_str("],\"t\":");
        out.push_str(&record.tokens.to_string());
        out.push('}');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(i: usize) -> Record {
        Record {
            day: format!("2024-01-{:02}", (i % 28) + 1),
            category: (i % 8) as u8,
            experts: vec![(i % 64) as u16, ((i * 7) % 64) as u16],
            tokens: (i as u32) * 3 + 1,
        }
    }

    #[test]
    fn randomize_keeps_true_category_below_threshold() {
        // r_keep % keep_den == 0 < keep_num -> keep the true category.
        assert_eq!(randomize_category(5, 8, 3, 4, 0, 7), 5);
        assert_eq!(randomize_category(5, 8, 3, 4, 4, 7), 5);
        assert_eq!(randomize_category(5, 8, 3, 4, 8, 7), 5);
    }

    #[test]
    fn randomize_picks_uniformly_above_threshold() {
        // r_keep % keep_den == 3 >= keep_num -> r_pick % n_categories.
        assert_eq!(randomize_category(5, 8, 3, 4, 3, 11), 3);
        assert_eq!(randomize_category(5, 8, 3, 4, 7, 0), 0);
        assert_eq!(randomize_category(5, 8, 3, 4, 7, 8), 0);
        assert_eq!(randomize_category(5, 8, 3, 4, 7, 9), 1);
    }

    #[test]
    fn randomize_edge_cases() {
        // keep_den == 0 -> true category.
        assert_eq!(randomize_category(2, 8, 3, 0, 0, 5), 2);
        // n_categories == 0 -> true category.
        assert_eq!(randomize_category(2, 0, 3, 4, 3, 5), 2);
    }

    #[test]
    fn estimate_recovers_known_totals() {
        const N: u64 = 100_000;
        const CATEGORIES: usize = 8;
        const KEEP_PROB: f64 = 0.75;

        // Skewed true distribution (weights sum to 100).
        let weights: [u64; CATEGORIES] = [22, 18, 15, 12, 10, 9, 8, 6];
        let mut true_counts = [0u64; CATEGORIES];
        let mut observed = [0u64; CATEGORIES];

        let mut rng = SplitMix64::new(0xC0FF_EE12_3456_789A);
        for _ in 0..N {
            // Pick a true category from the skewed distribution.
            let roll = rng.next_u64() % 100;
            let mut acc = 0u64;
            let mut true_cat = CATEGORIES - 1;
            for (k, w) in weights.iter().enumerate() {
                acc += w;
                if roll < acc {
                    true_cat = k;
                    break;
                }
            }
            true_counts[true_cat] += 1;

            // Randomized response: keep with probability KEEP_PROB.
            let r_keep = (rng.next_u64() % 100) as u32;
            let r_pick = (rng.next_u64() % CATEGORIES as u64) as u32;
            let reported = randomize_category(
                true_cat as u8,
                CATEGORIES as u8,
                75,
                100,
                r_keep,
                r_pick,
            );
            observed[reported as usize] += 1;
        }

        let estimated = estimate_true_counts(&observed, KEEP_PROB);
        assert_eq!(estimated.len(), CATEGORIES);

        for k in 0..CATEGORIES {
            let truth = true_counts[k] as f64;
            // Sampling noise for observed_k is binomial; allow four standard
            // errors on top of the 2% relative tolerance.
            let p_k = KEEP_PROB * truth / N as f64 + (1.0 - KEEP_PROB) / CATEGORIES as f64;
            let se = (N as f64 * p_k * (1.0 - p_k)).sqrt() / KEEP_PROB;
            let tol = (0.02 * truth).max(4.0 * se);
            assert!(
                (estimated[k] - truth).abs() <= tol,
                "category {}: estimate {} vs true {} (tol {})",
                k,
                estimated[k],
                truth,
                tol
            );
        }
    }

    #[test]
    fn estimate_edge_cases() {
        let observed = [1u64, 2, 3];
        assert_eq!(estimate_true_counts(&observed, 0.0), vec![1.0, 2.0, 3.0]);
        assert_eq!(estimate_true_counts(&observed, -1.0), vec![1.0, 2.0, 3.0]);
        assert!(estimate_true_counts(&[], 0.75).is_empty());
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let original: Vec<Record> = (0..50).map(sample_record).collect();
        let mut shuffled = original.clone();
        shuffle_batch(&mut shuffled, 42);

        let mut a = original.clone();
        let mut b = shuffled.clone();
        a.sort_by_key(|r| r.tokens);
        b.sort_by_key(|r| r.tokens);
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_is_deterministic_for_a_seed() {
        let original: Vec<Record> = (0..50).map(sample_record).collect();
        let mut first = original.clone();
        let mut second = original.clone();
        shuffle_batch(&mut first, 7);
        shuffle_batch(&mut second, 7);
        assert_eq!(first, second);
    }

    #[test]
    fn shuffle_differs_between_seeds() {
        let original: Vec<Record> = (0..50).map(sample_record).collect();
        let mut first = original.clone();
        let mut second = original.clone();
        shuffle_batch(&mut first, 1);
        shuffle_batch(&mut second, 2);
        assert_ne!(first, second);
    }

    #[test]
    fn encode_exact_output() {
        let records = vec![
            Record {
                day: "2024-03-01".to_string(),
                category: 3,
                experts: vec![1, 2],
                tokens: 10,
            },
            Record {
                day: "2024-03-02".to_string(),
                category: 5,
                experts: vec![3],
                tokens: 20,
            },
        ];
        assert_eq!(
            encode_batch(&records),
            "{\"d\":\"2024-03-01\",\"c\":3,\"e\":[1,2],\"t\":10}\n{\"d\":\"2024-03-02\",\"c\":5,\"e\":[3],\"t\":20}"
        );
    }

    #[test]
    fn encode_drops_non_date_characters() {
        let records = vec![Record {
            day: "2024-03-01 (draft)".to_string(),
            category: 1,
            experts: vec![],
            tokens: 0,
        }];
        assert_eq!(
            encode_batch(&records),
            "{\"d\":\"2024-03-01\",\"c\":1,\"e\":[],\"t\":0}"
        );
    }
}
