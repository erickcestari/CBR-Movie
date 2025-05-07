// Import the levenshtein function from the strsim crate, which calculates
// the Levenshtein distance between two strings (the minimum number of single-character
// edits required to change one string into the other)
use strsim::levenshtein;

/// Calculates a similarity score between two numbers within a given range
///
/// Parameters:
/// - a: First number to compare
/// - b: Second number to compare
/// - max: Maximum possible value in the range
/// - min: Minimum possible value in the range
///
/// Returns:
/// - A float value between 0.0 and 1.0, where:
///   - 1.0 means the numbers are identical
///   - 0.0 means the numbers are as different as possible within the given range
///
/// The function normalizes the absolute difference between a and b by dividing it
/// by the range (max - min), then subtracts from 1.0 to convert from a distance
/// to a similarity score.
pub fn similarity_number(a: u32, b: u32, max: u32, min: u32) -> f32 {
    let diff = (a as f32 - b as f32).abs() / (max - min) as f32;
    1.0 - diff
}

/// Calculates a similarity score between two strings using Levenshtein distance
///
/// Parameters:
/// - a: First string to compare
/// - b: Second string to compare
///
/// Returns:
/// - A float value between 0.0 and 1.0, where:
///   - 1.0 means the strings are identical
///   - 0.0 means the strings are completely different
///
/// Special case: If both strings are empty, they are considered identical (1.0).
/// For all other cases, the function calculates the Levenshtein distance between
/// the strings and normalizes it by dividing by the length of the longer string,
/// then subtracts from 1.0 to convert from a distance to a similarity score.
pub fn similarity_string(a: &str, b: &str) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let max_len = a.len().max(b.len()) as f32;
    let distance = levenshtein(a, b) as f32;
    1.0 - (distance / max_len)
}

/// Calculates a similarity score between two collections of items by comparing their IDs
/// using the Jaccard index (intersection over union)
///
/// Parameters:
/// - a: First collection of items that implement the HasId trait
/// - b: Second collection of items that implement the HasId trait
///
/// Returns:
/// - A float value between 0.0 and 1.0, where:
///   - 1.0 means all IDs are identical in both collections
///   - 0.0 means there are no common IDs between the collections
///
/// The function extracts IDs from both collections, calculates the size of their
/// intersection and union, and returns the ratio intersection/union.
/// Special case: If the union is empty (both collections are empty), returns 0.0.
pub fn similarity_id<T: HasId>(a: &[T], b: &[T]) -> f32 {
    use std::collections::HashSet;

    // Extract IDs from both collections into HashSets for efficient operations
    let a_ids: HashSet<u32> = a.iter().map(|item| item.id()).collect();
    let b_ids: HashSet<u32> = b.iter().map(|item| item.id()).collect();

    // Calculate intersection (IDs that appear in both collections)
    let intersection: HashSet<_> = a_ids.intersection(&b_ids).collect();
    // Calculate union (unique IDs that appear in either collection)
    let union: HashSet<_> = a_ids.union(&b_ids).collect();

    // Return Jaccard index (intersection over union)
    // If both collections are empty (union is empty), return 0.0
    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

/// A trait that requires implementing types to provide an ID method
///
/// Any type that implements this trait can be used with the similarity_id function
/// to compare collections of objects based on their IDs.
pub trait HasId {
    /// Returns the unique ID of an object as a u32
    fn id(&self) -> u32;
}
