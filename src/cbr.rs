use strsim::levenshtein;

pub fn similarity_number(a: u32, b: u32, max: u32, min: u32) -> f32 {
    let diff = (a as f32 - b as f32).abs() / (max - min) as f32;
    diff
}

pub fn similarity_string(a: &str, b: &str) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let max_len = a.len().max(b.len()) as f32;
    let distance = levenshtein(a, b) as f32;
    1.0 - (distance / max_len)
}

pub fn similarity_id<T: HasId>(a: &[T], b: &[T]) -> f32 {
    use std::collections::HashSet;

    let a_ids: HashSet<u32> = a.iter().map(|item| item.id()).collect();
    let b_ids: HashSet<u32> = b.iter().map(|item| item.id()).collect();

    let intersection: HashSet<_> = a_ids.intersection(&b_ids).collect();
    let union: HashSet<_> = a_ids.union(&b_ids).collect();

    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

pub trait HasId {
    fn id(&self) -> u32;
}
