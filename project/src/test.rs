// keywords.rs
//
// Drop this next to your other code (add `mod keywords;` in main/lib), or paste it
// into your file. Run the tests with:  cargo test
//
// EXERCISE: delete the body of `has_keyword`, write your own version, and make the
// tests pass. The tests are the spec. Peek at mine only if you get stuck.

/// "Glue" = a character that would stick a Latin keyword INTO a bigger English word.
/// Burmese letters, spaces, punctuation and start/end-of-text are NOT glue.
fn is_glue(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// True if `word` appears in `text` without being glued into a bigger Latin word/number.
/// Only the keyword's OWN edges are guarded: a Burmese edge needs no guarding,
/// because Burmese has no spaces and English may legitimately touch it.
pub fn has_keyword(text: &str, word: &str) -> bool {
    let guard_left = word.chars().next().is_some_and(is_glue);
    let guard_right = word.chars().next_back().is_some_and(is_glue);

    text.match_indices(word).any(|(start, found)| {
        let before = text[..start].chars().next_back(); // char just before the match
        let after = text[start + found.len()..].chars().next(); // char just after it
        let left_ok = !guard_left || !before.is_some_and(is_glue);
        let right_ok = !guard_right || !after.is_some_and(is_glue);
        left_ok && right_ok
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_inside_english_words() {
        assert!(!has_keyword("this is it", "hi"));
        assert!(!has_keyword("taxi", "xi"));
        assert!(!has_keyword("i accept", "acc"));
        assert!(!has_keyword("11 aed", "1 aed"));
    }

    #[test]
    fn found_next_to_burmese() {
        assert!(has_keyword("အကိုaedmmk လေးပြောပေးပါ", "aedmmk"));
        assert!(has_keyword("ကaedmmk", "aedmmk")); // Burmese CONSONANT touching English
        assert!(has_keyword("aedမြန်မာကိုလွှဲ", "မြန်မာကိုလွှဲ")); // English touching a Burmese keyword
    }

    #[test]
    fn plain_cases() {
        assert!(has_keyword("hi", "hi"));
        assert!(has_keyword("hi, bro", "hi"));
        assert!(has_keyword("acc pls", "acc"));
    }

    // YOUR DECISION: should "hiii" or "helloo" count as a greeting?
    // Right now they don't. Write the test for what YOU want, then change the code.
}