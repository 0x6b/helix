//! Utility functions to categorize a `char`.

use crate::LineEnding;

#[derive(Debug, Eq, PartialEq)]
pub enum CharCategory {
    Whitespace,
    Eol,
    Word,
    Punctuation,
    Unknown,
    Hiragana,
    Katakana,
    Kanji,
}

#[inline]
pub fn categorize_char(ch: char) -> CharCategory {
    if char_is_hiragana(ch) {
        CharCategory::Hiragana
    } else if char_is_katakana(ch) {
        CharCategory::Katakana
    } else if char_is_kanji(ch) {
        CharCategory::Kanji
    } else if char_is_line_ending(ch) {
        CharCategory::Eol
    } else if ch.is_whitespace() {
        CharCategory::Whitespace
    } else if char_is_word(ch) {
        CharCategory::Word
    } else if char_is_punctuation(ch) {
        CharCategory::Punctuation
    } else {
        CharCategory::Unknown
    }
}

// Determine whether a character is a hiragana character.
#[inline]
pub fn char_is_hiragana(ch: char) -> bool {
    ('\u{3041}'..='\u{3096}').contains(&ch) || ('\u{3099}'..='\u{309F}').contains(&ch) // Hiragana: https://www.unicode.org/charts/PDF/U3040.pdf
        || ('\u{1B100}'..='\u{1B12F}').contains(&ch) // Kana Extended-A: https://www.unicode.org/charts/PDF/U1B100.pdf
        || ('\u{1AFF0}'..='\u{1AFFF}').contains(&ch) // Kana Extended-B: https://www.unicode.org/charts/PDF/U1AFF0.pdf
        || ('\u{1B000}'..='\u{1B0FF}').contains(&ch) // Kana Supplement: https://www.unicode.org/charts/PDF/U1B000.pdf
        || ('\u{1B130}'..='\u{1B16F}').contains(&ch) // Small Kana Extension: https://www.unicode.org/charts/PDF/U1B130.pdf
}

// Determine whether a character is a katakana character.
#[inline]
pub fn char_is_katakana(ch: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&ch) // Katakana: https://www.unicode.org/charts/PDF/U30A0.pdf
}

// Determine whether a character is a kanji, or CJK Unified Ideographs, character.
#[inline]
pub fn char_is_kanji(ch: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&ch) // CJK Unified Ideographs: https://www.unicode.org/charts/PDF/U4E00.pdf
        || ('\u{3400}'..='\u{4DBF}').contains(&ch) // CJK Unified Ideographs Extension A: https://www.unicode.org/charts/PDF/U3400.pdf
        || ('\u{20000}'..='\u{2A6DF}').contains(&ch) // CJK Unified Ideographs Extension B: https://www.unicode.org/charts/PDF/U20000.pdf
        || ('\u{2A700}'..='\u{2B739}').contains(&ch) // CJK Unified Ideographs Extension C: https://www.unicode.org/charts/PDF/U2A700.pdf
        || ('\u{2B740}'..='\u{2B81D}').contains(&ch) // CJK Unified Ideographs Extension D: https://www.unicode.org/charts/PDF/U2B740.pdf
        || ('\u{2B820}'..='\u{2CEA1}').contains(&ch) // CJK Unified Ideographs Extension E: https://www.unicode.org/charts/PDF/U2B820.pdf
        || ('\u{2CEB0}'..='\u{2EBE0}').contains(&ch) // CJK Unified Ideographs Extension F: https://www.unicode.org/charts/PDF/U2CEB0.pdf
        || ('\u{30000}'..='\u{3134A}').contains(&ch) // CJK Unified Ideographs Extension G: https://www.unicode.org/charts/PDF/U30000.pdf
        || ('\u{31350}'..='\u{323AF}').contains(&ch) // CJK Unified Ideographs Extension H: https://www.unicode.org/charts/PDF/U31350.pdf
        || ('\u{2EBF0}'..='\u{2EE5D}').contains(&ch) // CJK Unified Ideographs Extension H: https://www.unicode.org/charts/PDF/U2EBF0.pdf
        || ('\u{F900}'..='\u{FAFF}').contains(&ch) // CJK Compatibility Ideographs: https://www.unicode.org/charts/PDF/UF900.pdf
        || ('\u{2F800}'..='\u{2FA1F}').contains(&ch) // CJK Compatibility Ideographs Supplement: https://www.unicode.org/charts/PDF/U2F800.pdf
}

/// Determine whether a character is a line ending.
#[inline]
pub fn char_is_line_ending(ch: char) -> bool {
    LineEnding::from_char(ch).is_some()
}

/// Determine whether a character qualifies as (non-line-break)
/// whitespace.
#[inline]
pub fn char_is_whitespace(ch: char) -> bool {
    // TODO: this is a naive binary categorization of whitespace
    // characters.  For display, word wrapping, etc. we'll need a better
    // categorization based on e.g. breaking vs non-breaking spaces
    // and whether they're zero-width or not.
    match ch {
        //'\u{1680}' | // Ogham Space Mark (here for completeness, but usually displayed as a dash, not as whitespace)
        '\u{0009}' | // Character Tabulation
        '\u{0020}' | // Space
        '\u{00A0}' | // No-break Space
        '\u{180E}' | // Mongolian Vowel Separator
        '\u{202F}' | // Narrow No-break Space
        '\u{205F}' | // Medium Mathematical Space
        '\u{3000}' | // Ideographic Space
        '\u{FEFF}'   // Zero Width No-break Space
        => true,

        // En Quad, Em Quad, En Space, Em Space, Three-per-em Space,
        // Four-per-em Space, Six-per-em Space, Figure Space,
        // Punctuation Space, Thin Space, Hair Space, Zero Width Space.
        ch if ('\u{2000}' ..= '\u{200B}').contains(&ch) => true,

        _ => false,
    }
}

#[inline]
pub fn char_is_punctuation(ch: char) -> bool {
    use unicode_general_category::{get_general_category, GeneralCategory};

    matches!(
        get_general_category(ch),
        GeneralCategory::OtherPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
    )
}

#[inline]
pub fn char_is_word(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_categorize() {
        #[cfg(not(feature = "unicode-lines"))]
        const EOL_TEST_CASE: &str = "\n";
        #[cfg(feature = "unicode-lines")]
        const EOL_TEST_CASE: &str = "\n\u{000B}\u{000C}\u{0085}\u{2028}\u{2029}";
        const WORD_TEST_CASE: &str = "_hello_world_あいうえおー1234567890１２３４５６７８９０";
        const PUNCTUATION_TEST_CASE: &str =
            "!\"#$%&\'()*+,-./:;<=>?@[\\]^`{|}~！”＃＄％＆’（）＊＋、。：；＜＝＞？＠「」＾｀｛｜｝～";
        const WHITESPACE_TEST_CASE: &str = "  　   ";

        for ch in EOL_TEST_CASE.chars() {
            assert_eq!(CharCategory::Eol, categorize_char(ch));
        }

        for ch in WHITESPACE_TEST_CASE.chars() {
            assert_eq!(
                CharCategory::Whitespace,
                categorize_char(ch),
                "Testing '{}', but got `{:?}` instead of `Category::Whitespace`",
                ch,
                categorize_char(ch)
            );
        }

        for ch in WORD_TEST_CASE.chars() {
            assert_eq!(
                CharCategory::Word,
                categorize_char(ch),
                "Testing '{}', but got `{:?}` instead of `Category::Word`",
                ch,
                categorize_char(ch)
            );
        }

        for ch in PUNCTUATION_TEST_CASE.chars() {
            assert_eq!(
                CharCategory::Punctuation,
                categorize_char(ch),
                "Testing '{}', but got `{:?}` instead of `Category::Punctuation`",
                ch,
                categorize_char(ch)
            );
        }
    }
}
