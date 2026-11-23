//! Weighted reciprocal rank fusion of the vector list and the full-text list.

use std::collections::HashMap;

/// The constant k of reciprocal rank fusion.
///
/// Provisional value after ADR 0132, chosen on the test data of criterion E48
/// (embedding test, weight 0.5): with the common k = 60 the hybrid search found 47
/// of 52 questions against 51 of the vector search alone; with k from 0.5 to 8 it
/// found 51. Phase 2 sets k and the weight on the question catalogue, judged by the
/// first twenty hits (README, "Fusion").
pub const RRF_K: f64 = 5.0;

/// Fused score of one item: `w / (k + r_v) + (1 - w) / (k + r_f)`, with 1-based ranks;
/// a list the item is missing from contributes nothing.
pub fn rrf_score(vector_weight: f64, vector_rank: Option<usize>, text_rank: Option<usize>) -> f64 {
    let part = |weight: f64, rank: Option<usize>| match rank {
        Some(r) => weight / (RRF_K + r as f64),
        None => 0.0,
    };
    part(vector_weight, vector_rank) + part(1.0 - vector_weight, text_rank)
}

/// Fuses two ranked id lists (best first). Returns `(id, score)` best first; equal
/// scores are ordered by id, so the result is deterministic.
pub fn fuse(vector_weight: f64, vector: &[String], text: &[String]) -> Vec<(String, f64)> {
    let mut ranks: HashMap<&str, (Option<usize>, Option<usize>)> = HashMap::new();
    for (i, id) in vector.iter().enumerate() {
        ranks.entry(id).or_default().0.get_or_insert(i + 1);
    }
    for (i, id) in text.iter().enumerate() {
        ranks.entry(id).or_default().1.get_or_insert(i + 1);
    }
    let mut out: Vec<(String, f64)> = ranks
        .into_iter()
        .map(|(id, (v, t))| (id.to_string(), rrf_score(vector_weight, v, t)))
        .collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn items_in_both_lists_beat_items_with_the_same_rank_in_one() {
        let fused = fuse(0.5, &ids(&["a", "b"]), &ids(&["a", "c"]));
        assert_eq!(fused[0].0, "a");
        assert!((fused[0].1 - (0.5 / (RRF_K + 1.0) + 0.5 / (RRF_K + 1.0))).abs() < 1e-12);
        // b and c have one rank 2 each and equal scores; ties go by id.
        assert_eq!(fused[1].0, "b");
        assert_eq!(fused[2].0, "c");
        assert!((fused[1].1 - fused[2].1).abs() < 1e-12);
    }

    #[test]
    fn weight_one_keeps_vector_order_and_weight_zero_text_order() {
        let v = ids(&["x", "y", "z"]);
        let t = ids(&["z", "w"]);
        let one: Vec<String> = fuse(1.0, &v, &t).into_iter().map(|p| p.0).take(3).collect();
        assert_eq!(one, v);
        let zero: Vec<String> = fuse(0.0, &v, &t).into_iter().map(|p| p.0).take(2).collect();
        assert_eq!(zero, t);
    }
}
