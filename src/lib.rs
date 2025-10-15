pub mod cache_repository;
pub mod constants;
pub mod entities;
pub mod fetch_translations;
pub mod migration;
pub mod user_repository;

use constants::{LANG_EN_IT, LANG_EN_PT, LANG_IT_EN, LANG_PT_EN};

/// Reverses a translation direction by swapping source and target languages.
///
/// Takes a 4-letter direction code (format: `{source}{target}` where each is 2 letters)
/// and returns the reversed direction. Returns None if the input is not exactly 4 letters
/// or if the direction is not in the allowed list (enpt, pten, enit, iten).
///
/// # Examples
///
/// ```
/// use pt_dict_bot::flip_direction;
///
/// assert_eq!(flip_direction("pten"), Some("enpt".to_string()));
/// assert_eq!(flip_direction("enpt"), Some("pten".to_string()));
/// assert_eq!(flip_direction("iten"), Some("enit".to_string()));
/// assert_eq!(flip_direction("enit"), Some("iten".to_string()));
/// assert_eq!(flip_direction("fres"), None); // not in allowed list
/// assert_eq!(flip_direction("invalid"), None); // not 4 letters
/// ```
pub fn flip_direction(current: &str) -> Option<String> {
    // Direction must be exactly 4 characters
    if current.len() != 4 {
        return None;
    }

    // Only allow specific directions
    const ALLOWED_DIRECTIONS: [&str; 4] = [LANG_PT_EN, LANG_EN_PT, LANG_IT_EN, LANG_EN_IT];
    if !ALLOWED_DIRECTIONS.contains(&current) {
        return None;
    }

    // Split into two 2-character parts and swap them
    let source = &current[0..2];
    let target = &current[2..4];

    Some(format!("{}{}", target, source))
}
