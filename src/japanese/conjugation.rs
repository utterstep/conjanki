use jp_inflections::{VerbType, Word, WordForm};
use serde::{Deserialize, Serialize};

/// Conjugation forms available in jp_inflections 0.1.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConjugationForm {
    Dictionary,
    Negative,
    Past,
    NegativePast,
    TeForm,
    NegativeTeForm,
    Potential,
    NegativePotential,
    Passive,
    NegativePassive,
    Causative,
    NegativeCausative,
    CausativePassive,
    NegativeCausativePassive,
    Imperative,
    ImperativeNegative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Formality {
    Plain,
    Polite,
}

impl ConjugationForm {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dictionary => "dictionary form (辞書形)",
            Self::Negative => "negative (〜ない)",
            Self::Past => "past (〜た)",
            Self::NegativePast => "negative past (〜なかった)",
            Self::TeForm => "て-form (〜て)",
            Self::NegativeTeForm => "negative て-form (〜ないで)",
            Self::Potential => "potential (〜られる/〜える)",
            Self::NegativePotential => "negative potential",
            Self::Passive => "passive (〜られる)",
            Self::NegativePassive => "negative passive",
            Self::Causative => "causative (〜させる)",
            Self::NegativeCausative => "negative causative",
            Self::CausativePassive => "causative passive (〜させられる)",
            Self::NegativeCausativePassive => "negative causative passive",
            Self::Imperative => "imperative (〜ろ/〜なさい)",
            Self::ImperativeNegative => "negative imperative (〜するな)",
        }
    }

    /// Short name used in drill prompts (without the Japanese suffix).
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Dictionary => "dictionary form",
            Self::Negative => "negative",
            Self::Past => "past",
            Self::NegativePast => "negative past",
            Self::TeForm => "て-form",
            Self::NegativeTeForm => "negative て-form",
            Self::Potential => "potential",
            Self::NegativePotential => "negative potential",
            Self::Passive => "passive",
            Self::NegativePassive => "negative passive",
            Self::Causative => "causative",
            Self::NegativeCausative => "negative causative",
            Self::CausativePassive => "causative passive",
            Self::NegativeCausativePassive => "negative causative passive",
            Self::Imperative => "imperative",
            Self::ImperativeNegative => "negative imperative",
        }
    }

    /// Whether this form takes a WordForm (plain/polite) parameter.
    pub fn supports_formality(&self) -> bool {
        matches!(
            self,
            Self::Dictionary
                | Self::Negative
                | Self::Past
                | Self::NegativePast
                | Self::Potential
                | Self::NegativePotential
        )
    }

    pub fn all() -> &'static [ConjugationForm] {
        &[
            Self::Dictionary,
            Self::Negative,
            Self::Past,
            Self::NegativePast,
            Self::TeForm,
            Self::NegativeTeForm,
            Self::Potential,
            Self::NegativePotential,
            Self::Passive,
            Self::NegativePassive,
            Self::Causative,
            Self::NegativeCausative,
            Self::CausativePassive,
            Self::NegativeCausativePassive,
            Self::Imperative,
            Self::ImperativeNegative,
        ]
    }

    /// Forms grouped roughly by Genki textbook order, with group labels.
    pub fn grouped() -> &'static [(&'static str, &'static [ConjugationForm])] {
        &[
            (
                "Basic (Genki I ch. 3–8)",
                &[
                    Self::Dictionary,
                    Self::Negative,
                    Self::Past,
                    Self::NegativePast,
                    Self::TeForm,
                    Self::NegativeTeForm,
                ],
            ),
            (
                "Intermediate (Genki I ch. 13+)",
                &[Self::Potential, Self::NegativePotential],
            ),
            (
                "Advanced (Genki II ch. 21–23)",
                &[
                    Self::Passive,
                    Self::NegativePassive,
                    Self::Causative,
                    Self::NegativeCausative,
                    Self::CausativePassive,
                    Self::NegativeCausativePassive,
                ],
            ),
            ("Other", &[Self::Imperative, Self::ImperativeNegative]),
        ]
    }

    pub fn db_name(&self) -> &'static str {
        match self {
            Self::Dictionary => "dictionary",
            Self::Negative => "negative",
            Self::Past => "past",
            Self::NegativePast => "negative_past",
            Self::TeForm => "te_form",
            Self::NegativeTeForm => "negative_te_form",
            Self::Potential => "potential",
            Self::NegativePotential => "negative_potential",
            Self::Passive => "passive",
            Self::NegativePassive => "negative_passive",
            Self::Causative => "causative",
            Self::NegativeCausative => "negative_causative",
            Self::CausativePassive => "causative_passive",
            Self::NegativeCausativePassive => "negative_causative_passive",
            Self::Imperative => "imperative",
            Self::ImperativeNegative => "imperative_negative",
        }
    }

    pub fn from_db_name(s: &str) -> Option<Self> {
        match s {
            "dictionary" => Some(Self::Dictionary),
            "negative" => Some(Self::Negative),
            "past" => Some(Self::Past),
            "negative_past" => Some(Self::NegativePast),
            "te_form" => Some(Self::TeForm),
            "negative_te_form" => Some(Self::NegativeTeForm),
            "potential" => Some(Self::Potential),
            "negative_potential" => Some(Self::NegativePotential),
            "passive" => Some(Self::Passive),
            "negative_passive" => Some(Self::NegativePassive),
            "causative" => Some(Self::Causative),
            "negative_causative" => Some(Self::NegativeCausative),
            "causative_passive" => Some(Self::CausativePassive),
            "negative_causative_passive" => Some(Self::NegativeCausativePassive),
            "imperative" => Some(Self::Imperative),
            "imperative_negative" => Some(Self::ImperativeNegative),
            _ => None,
        }
    }
}

impl Formality {
    pub fn db_name(&self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Polite => "polite",
        }
    }

    pub fn from_db_name(s: &str) -> Option<Self> {
        match s {
            "plain" => Some(Self::Plain),
            "polite" => Some(Self::Polite),
            _ => None,
        }
    }
}

pub fn conjugate(
    kanji: Option<&str>,
    kana: &str,
    verb_type: VerbType,
    form: ConjugationForm,
    formality: Formality,
) -> Result<Vec<String>, String> {
    let word = Word::new(kana, kanji);
    let verb = word.into_verb(verb_type).map_err(|e| format!("{e:?}"))?;
    let wf = match formality {
        Formality::Plain => WordForm::Short,
        Formality::Polite => WordForm::Long,
    };

    let result: jp_inflections::Word = match form {
        ConjugationForm::Dictionary => verb.dictionary(wf),
        ConjugationForm::Negative => verb.negative(wf),
        ConjugationForm::Past => verb.past(wf),
        ConjugationForm::NegativePast => verb.negative_past(wf),
        ConjugationForm::TeForm => verb.te_form(),
        ConjugationForm::NegativeTeForm => verb.negative_te_form(),
        ConjugationForm::Potential => verb.potential(wf),
        ConjugationForm::NegativePotential => verb.negative_potential(wf),
        ConjugationForm::Passive => verb.passive(),
        ConjugationForm::NegativePassive => verb.negative_passive(),
        ConjugationForm::Causative => verb.causative(),
        ConjugationForm::NegativeCausative => verb.negative_causative(),
        ConjugationForm::CausativePassive => verb.causative_passive(),
        ConjugationForm::NegativeCausativePassive => verb.negative_causative_passive(),
        ConjugationForm::Imperative => verb.imperative(),
        ConjugationForm::ImperativeNegative => verb.imperative_negative(),
    }
    .map_err(|e| format!("{e:?}"))?;

    let mut answers = vec![result.kana.clone()];
    if let Some(ref k) = result.kanji
        && k != &result.kana
    {
        answers.push(k.clone());
    }
    Ok(answers)
}

/// Compute the masu-stem (連用形) of a verb in kana.
pub fn masu_stem(kana: &str, verb_type: VerbType) -> String {
    match verb_type {
        VerbType::Ichidan => {
            // Drop final る
            kana.strip_suffix('る').unwrap_or(kana).to_string()
        }
        VerbType::Godan => {
            let mut chars: Vec<char> = kana.chars().collect();
            if let Some(last) = chars.last_mut() {
                *last = godan_u_to_i(*last);
            }
            chars.into_iter().collect()
        }
        VerbType::Exception => {
            if kana == "する" {
                "し".to_string()
            } else if kana.ends_with("する") {
                format!("{}し", kana.strip_suffix("する").unwrap())
            } else if kana == "くる" {
                "き".to_string()
            } else {
                kana.to_string()
            }
        }
    }
}

fn godan_u_to_i(ch: char) -> char {
    match ch {
        'う' => 'い',
        'く' => 'き',
        'ぐ' => 'ぎ',
        'す' => 'し',
        'つ' => 'ち',
        'ぬ' => 'に',
        'ぶ' => 'び',
        'む' => 'み',
        'る' => 'り',
        other => other,
    }
}

/// Generate a brief explanation of how a conjugation is formed.
pub fn explain(verb_type: VerbType, form: ConjugationForm, formality: Formality) -> String {
    let vt = match verb_type {
        VerbType::Godan => "godan (五段)",
        VerbType::Ichidan => "ichidan (一段)",
        VerbType::Exception => "irregular",
    };

    let rule = match (form, formality) {
        // Basic forms
        (ConjugationForm::Dictionary, Formality::Polite) => "stem + ます",
        (ConjugationForm::Negative, Formality::Plain) => match verb_type {
            VerbType::Ichidan => "drop る → ない",
            VerbType::Godan => "u→a row + ない",
            VerbType::Exception => "irregular: する→しない, 来る→こない",
        },
        (ConjugationForm::Negative, Formality::Polite) => "stem + ません",
        (ConjugationForm::Past, Formality::Plain) => match verb_type {
            VerbType::Ichidan => "drop る → た",
            VerbType::Godan => "te-form with た instead of て",
            VerbType::Exception => "irregular: する→した, 来る→きた",
        },
        (ConjugationForm::Past, Formality::Polite) => "stem + ました",
        (ConjugationForm::NegativePast, Formality::Plain) => match verb_type {
            VerbType::Ichidan => "drop る → なかった",
            VerbType::Godan => "u→a row + なかった",
            VerbType::Exception => "irregular: する→しなかった, 来る→こなかった",
        },
        (ConjugationForm::NegativePast, Formality::Polite) => "stem + ませんでした",
        (ConjugationForm::TeForm, _) => match verb_type {
            VerbType::Ichidan => "drop る → て",
            VerbType::Godan => "く→いて, ぐ→いで, す→して, む/ぶ/ぬ→んで, つ/る/う→って",
            VerbType::Exception => "irregular: する→して, 来る→きて",
        },
        (ConjugationForm::NegativeTeForm, _) => "negative (ない) → ないで",
        // Intermediate
        (ConjugationForm::Potential, _) => match verb_type {
            VerbType::Ichidan => "drop る → られる (or れる colloquial)",
            VerbType::Godan => "u→e row + る",
            VerbType::Exception => "irregular: する→できる, 来る→こられる",
        },
        (ConjugationForm::NegativePotential, _) => match verb_type {
            VerbType::Ichidan => "drop る → られない",
            VerbType::Godan => "u→e row + ない",
            VerbType::Exception => "irregular: する→できない, 来る→こられない",
        },
        // Advanced
        (ConjugationForm::Passive, _) => match verb_type {
            VerbType::Ichidan => "drop る → られる",
            VerbType::Godan => "u→a row + れる",
            VerbType::Exception => "irregular: する→される, 来る→こられる",
        },
        (ConjugationForm::Causative, _) => match verb_type {
            VerbType::Ichidan => "drop る → させる",
            VerbType::Godan => "u→a row + せる",
            VerbType::Exception => "irregular: する→させる, 来る→こさせる",
        },
        (ConjugationForm::CausativePassive, _) => match verb_type {
            VerbType::Ichidan => "drop る → させられる",
            VerbType::Godan => "u→a row + せられる (or contracted: 〜される)",
            VerbType::Exception => "irregular: する→させられる, 来る→こさせられる",
        },
        (ConjugationForm::Imperative, _) => match verb_type {
            VerbType::Ichidan => "drop る → ろ",
            VerbType::Godan => "u→e row",
            VerbType::Exception => "irregular: する→しろ, 来る→こい",
        },
        _ => "see conjugation rule",
    };

    format!("{vt}: {rule}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conj(
        kana: &str,
        kanji: Option<&str>,
        vt: VerbType,
        form: ConjugationForm,
        f: Formality,
    ) -> Vec<String> {
        conjugate(kanji, kana, vt, form, f).unwrap()
    }

    #[test]
    fn test_ichidan_conjugations() {
        use ConjugationForm::*;
        use Formality::*;
        let vt = VerbType::Ichidan;

        // 食べる
        assert!(
            conj("たべる", Some("食べる"), vt, Negative, Plain).contains(&"たべない".to_string())
        );
        assert!(conj("たべる", Some("食べる"), vt, Past, Plain).contains(&"たべた".to_string()));
        assert!(conj("たべる", Some("食べる"), vt, TeForm, Plain).contains(&"たべて".to_string()));
        assert!(
            conj("たべる", Some("食べる"), vt, Negative, Polite)
                .contains(&"たべません".to_string())
        );
        assert!(
            conj("たべる", Some("食べる"), vt, Past, Polite).contains(&"たべました".to_string())
        );

        // いる (ichidan, not godan!)
        assert!(conj("いる", None, vt, Negative, Plain).contains(&"いない".to_string()));
        assert!(conj("いる", None, vt, Past, Plain).contains(&"いた".to_string()));
        assert!(conj("いる", None, vt, TeForm, Plain).contains(&"いて".to_string()));

        // 見る
        assert!(conj("みる", Some("見る"), vt, Negative, Plain).contains(&"みない".to_string()));
    }

    #[test]
    fn test_godan_conjugations() {
        use ConjugationForm::*;
        use Formality::*;
        let vt = VerbType::Godan;

        // 行く (irregular te-form: 行って not 行いて)
        assert!(conj("いく", Some("行く"), vt, Negative, Plain).contains(&"いかない".to_string()));
        assert!(conj("いく", Some("行く"), vt, Past, Plain).contains(&"いった".to_string()));
        assert!(conj("いく", Some("行く"), vt, TeForm, Plain).contains(&"いって".to_string()));

        // 読む
        assert!(conj("よむ", Some("読む"), vt, Negative, Plain).contains(&"よまない".to_string()));
        assert!(conj("よむ", Some("読む"), vt, TeForm, Plain).contains(&"よんで".to_string()));

        // 話す
        assert!(conj("はなす", Some("話す"), vt, TeForm, Plain).contains(&"はなして".to_string()));
    }

    #[test]
    fn test_exception_conjugations() {
        use ConjugationForm::*;
        use Formality::*;
        let vt = VerbType::Exception;

        // する
        assert!(conj("する", None, vt, Negative, Plain).contains(&"しない".to_string()));
        assert!(conj("する", None, vt, Past, Plain).contains(&"した".to_string()));
        assert!(conj("する", None, vt, TeForm, Plain).contains(&"して".to_string()));
        assert!(conj("する", None, vt, Negative, Polite).contains(&"しません".to_string()));
        assert!(conj("する", None, vt, Potential, Plain).contains(&"できる".to_string()));

        // 来る
        assert!(conj("くる", Some("来る"), vt, Negative, Plain).contains(&"こない".to_string()));
        assert!(conj("くる", Some("来る"), vt, Past, Plain).contains(&"きた".to_string()));
        assert!(conj("くる", Some("来る"), vt, TeForm, Plain).contains(&"きて".to_string()));
        assert!(conj("くる", Some("来る"), vt, Negative, Polite).contains(&"きません".to_string()));
    }
}
