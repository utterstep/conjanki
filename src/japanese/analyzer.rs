use jp_inflections::VerbType;
use lindera::tokenizer::Tokenizer;

#[allow(dead_code)] // base_form and reading used in tests
pub struct VerbAnalysis {
    pub base_form: String,
    pub reading: String,
    pub verb_type: VerbType,
}

pub fn analyze_verb(tokenizer: &Tokenizer, word: &str) -> Option<VerbAnalysis> {
    let mut tokens = tokenizer.tokenize(word).ok()?;

    // Reject phrases: if any token is a particle (助詞) or auxiliary verb (助動詞),
    // this is a phrase or conjugated form, not a dictionary-form verb.
    for token in &mut tokens {
        let pos = token.details().first().copied().unwrap_or("");
        if pos == "助詞" || pos == "助動詞" {
            return None;
        }
    }

    // Re-tokenize (tokenize consumes the iterator)
    let mut tokens = tokenizer.tokenize(word).ok()?;

    for token in &mut tokens {
        let details = token.details();

        // IPADIC detail format (9 fields):
        // [0] = 品詞 (POS)           e.g. "動詞"
        // [1] = 品詞細分類1           e.g. "自立"
        // [2] = 品詞細分類2           e.g. "*"
        // [3] = 品詞細分類3           e.g. "*"
        // [4] = 活用型 (conj. type)   e.g. "五段・カ行イ音便", "一段", "サ変・スル", "カ変・来ル"
        // [5] = 活用形 (conj. form)   e.g. "基本形"
        // [6] = 原形 (base form)      e.g. "食べる"
        // [7] = 読み (reading)        e.g. "タベル"
        // [8] = 発音 (pronunciation)
        let pos = details.first()?;
        if *pos != "動詞" {
            continue;
        }

        // Only accept dictionary form (基本形) — reject conjugated inputs like ください
        let conj_form = details.get(5).copied().unwrap_or("*");
        if conj_form != "基本形" {
            continue;
        }

        let conjugation_type = details.get(4).copied().unwrap_or("*");
        let base_form = details.get(6)?.to_string();
        let reading = details.get(7).map(|s| s.to_string()).unwrap_or_default();

        let verb_type = classify_verb_type(conjugation_type, &base_form)?;

        return Some(VerbAnalysis {
            base_form,
            reading,
            verb_type,
        });
    }
    None
}

fn classify_verb_type(conjugation_type: &str, base_form: &str) -> Option<VerbType> {
    // Check exception types first (サ変 = suru, カ変 = kuru)
    if conjugation_type.contains("サ変") || conjugation_type.contains("カ変") {
        return Some(VerbType::Exception);
    }

    // IPADIC overrides: some common verbs are misclassified.
    // する is listed as 五段・ラ行 instead of サ変.
    // いる (居る) is listed as 五段・ラ行 instead of 一段.
    if let Some(vt) = ipadic_override(base_form) {
        return Some(vt);
    }

    if conjugation_type.contains("五段") {
        return Some(VerbType::Godan);
    }
    if conjugation_type.contains("一段") {
        return Some(VerbType::Ichidan);
    }
    // Unknown conjugation type — skip
    None
}

/// Override IPADIC verb classifications that are linguistically incorrect
/// for the most common reading of each verb.
fn ipadic_override(base_form: &str) -> Option<VerbType> {
    match base_form {
        "する" | "来る" => Some(VerbType::Exception),
        // 居る (to exist, animate) — ichidan, not godan.
        // IPADIC lists いる as 五段・ラ行 (matching 射る "to shoot"),
        // but the overwhelmingly common usage is ichidan 居る.
        "いる" => Some(VerbType::Ichidan),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tokenizer() -> Tokenizer {
        let dictionary = lindera::dictionary::load_dictionary("embedded://ipadic").unwrap();
        let segmenter =
            lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);
        Tokenizer::new(segmenter)
    }

    #[test]
    fn test_godan_verbs() {
        let tok = make_tokenizer();
        for (word, desc) in [
            ("行く", "to go"),
            ("帰る", "to return"),
            ("書く", "to write"),
            ("読む", "to read"),
            ("飲む", "to drink"),
            ("話す", "to speak"),
            ("ある", "to exist inanimate"),
        ] {
            let r = analyze_verb(&tok, word)
                .unwrap_or_else(|| panic!("{word} ({desc}) should be a verb"));
            assert_eq!(
                r.verb_type,
                VerbType::Godan,
                "{word} ({desc}) should be godan"
            );
        }
    }

    #[test]
    fn test_ichidan_verbs() {
        let tok = make_tokenizer();
        for (word, desc) in [
            ("食べる", "to eat"),
            ("見る", "to see"),
            ("教える", "to teach"),
            ("いる", "to exist animate"),
        ] {
            let r = analyze_verb(&tok, word)
                .unwrap_or_else(|| panic!("{word} ({desc}) should be a verb"));
            assert_eq!(
                r.verb_type,
                VerbType::Ichidan,
                "{word} ({desc}) should be ichidan"
            );
        }
    }

    #[test]
    fn test_exception_verbs() {
        let tok = make_tokenizer();
        for (word, desc) in [("する", "to do"), ("来る", "to come")] {
            let r = analyze_verb(&tok, word)
                .unwrap_or_else(|| panic!("{word} ({desc}) should be a verb"));
            assert_eq!(
                r.verb_type,
                VerbType::Exception,
                "{word} ({desc}) should be exception"
            );
        }
    }

    #[test]
    fn test_non_verbs() {
        let tok = make_tokenizer();
        for (word, desc) in [
            ("猫", "noun"),
            ("好き", "na-adjective"),
            ("大きい", "i-adjective"),
            ("東京", "proper noun"),
        ] {
            assert!(
                analyze_verb(&tok, word).is_none(),
                "{word} ({desc}) should NOT be detected as a verb",
            );
        }
    }

    #[test]
    fn test_phrases_rejected() {
        let tok = make_tokenizer();
        for (word, desc) in [
            ("薬を飲む", "phrase with particle を"),
            ("人気がある", "phrase with particle が"),
            ("お願いします", "polite-form phrase with ます"),
            ("失礼します", "polite-form phrase with ます"),
            ("ください", "conjugated form (imperative of くださる)"),
        ] {
            assert!(
                analyze_verb(&tok, word).is_none(),
                "{word} ({desc}) should NOT be detected as a verb",
            );
        }
    }
}
