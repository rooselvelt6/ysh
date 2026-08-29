//! Locale metadata and negotiation (Accept-Language resolution).

use fluent_langneg::NegotiationStrategy;
use fluent_langneg::negotiate_languages;
use unic_langid::LanguageIdentifier;

/// Human-readable metadata for a supported locale.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocaleMeta {
    pub code: String,
    pub name: String,
    pub native: String,
    pub dir: &'static str,
    pub rtl: bool,
    /// Locale-specific decimal separator.
    pub decimal_sep: char,
    /// Locale-specific thousands separator.
    pub group_sep: char,
    /// Locale-specific currency symbol.
    pub currency: &'static str,
}

pub fn supported_meta() -> Vec<LocaleMeta> {
    vec![
        LocaleMeta {
            code: "es".into(),
            name: "Spanish".into(),
            native: "Español".into(),
            dir: "ltr",
            rtl: false,
            decimal_sep: ',',
            group_sep: '.',
            currency: "€",
        },
        LocaleMeta {
            code: "en".into(),
            name: "English".into(),
            native: "English".into(),
            dir: "ltr",
            rtl: false,
            decimal_sep: '.',
            group_sep: ',',
            currency: "$",
        },
        LocaleMeta {
            code: "pt".into(),
            name: "Portuguese".into(),
            native: "Português".into(),
            dir: "ltr",
            rtl: false,
            decimal_sep: ',',
            group_sep: '.',
            currency: "R$",
        },
        LocaleMeta {
            code: "ar".into(),
            name: "Arabic".into(),
            native: "العربية".into(),
            dir: "rtl",
            rtl: true,
            decimal_sep: '٫',
            group_sep: '٬',
            currency: "ر.س",
        },
        LocaleMeta {
            code: "fr".into(),
            name: "French".into(),
            native: "Français".into(),
            dir: "ltr",
            rtl: false,
            decimal_sep: ',',
            group_sep: ' ',
            currency: "€",
        },
    ]
}

/// Returns the best supported locale for a raw `Accept-Language` header value.
///
/// Falls back to the default locale (`es`) when nothing negotiates.
pub fn negotiate(accept_language: &str, default: &str) -> String {
    let accepted: Vec<LanguageIdentifier> = accept_language
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            let tag = part.split(';').next().unwrap_or("").trim();
            tag.parse::<LanguageIdentifier>().ok()
        })
        .collect();

    let available: Vec<LanguageIdentifier> = crate::i18n::catalog::SUPPORTED
        .iter()
        .filter_map(|c| c.parse().ok())
        .collect();
    let default_id: LanguageIdentifier = default.parse().unwrap_or_else(|_| "en".parse().unwrap());

    let negotiated = negotiate_languages(
        &accepted,
        &available,
        Some(&default_id),
        NegotiationStrategy::Filtering,
    );

    negotiated
        .first()
        .map(|l| l.to_string())
        .filter(|c| crate::i18n::catalog::SUPPORTED.contains(&c.as_str()))
        .unwrap_or_else(|| default.to_string())
}

/// Returns RTL metadata for a normalized locale code.
#[allow(dead_code)]
pub fn is_rtl(locale: &str) -> bool {
    supported_meta()
        .iter()
        .find(|m| m.code == locale)
        .map(|m| m.rtl)
        .unwrap_or(false)
}

/// Returns the full metadata for a normalized locale code.
pub fn meta_for(locale: &str) -> Option<LocaleMeta> {
    supported_meta().into_iter().find(|m| m.code == locale)
}
