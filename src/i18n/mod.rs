//! Internationalization engine for YSH.
//!
//! Provides locale negotiation, a Project Fluent message catalog for five
//! locales (es, en, pt, ar, fr) with a fallback chain, per-locale number/
//! currency/date formatting, RTL metadata, and runtime translation overrides
//! used by the admin panel.

pub mod catalog;
pub mod format;
pub mod locales;

use std::collections::HashMap;
use std::sync::Arc;

use fluent::{FluentArgs, FluentBundle, FluentResource};

/// A typed Fluent argument value. Numeric values enable plural rule selection.
#[derive(Debug, Clone)]
pub enum Arg {
    Number(i64),
    Text(String),
}

/// Registry of supported locales and runtime overrides for message catalogs.
#[derive(Debug, Clone, Default)]
pub struct I18nEngine {
    /// Overrides keyed as `"{locale}::{key}"` -> value. Filled by the admin
    /// panel and layered on top of the bundled catalog.
    pub overrides: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl I18nEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the base FTL source for a supported locale (or `None`).
    pub fn base_source(&self, locale: &str) -> Option<&'static str> {
        catalog::source_for(locale)
    }

    /// Builds a full Fluent bundle for a locale, layering any runtime
    /// overrides (parsed as an extra FTL resource) on top of the base catalog.
    pub fn bundle_for(&self, locale: &str) -> Option<FluentBundle<FluentResource>> {
        let source = self.base_source(locale)?;
        let id: unic_langid::LanguageIdentifier = locale.parse().ok()?;

        let mut bundle = FluentBundle::new(vec![id]);
        let resource = FluentResource::try_new(source.to_string()).ok()?;
        if bundle.add_resource(resource).is_err() {
            return None;
        }

        if let Ok(guard) = self.overrides.lock() {
            let mut extra = String::new();
            for (k, v) in guard.iter() {
                if let Some((l, key)) = k.split_once("::")
                    && l == locale
                {
                    extra.push_str(&format!("{key} = {v}\n"));
                }
            }
            if !extra.is_empty()
                && let Ok(over) = FluentResource::try_new(extra)
            {
                bundle.add_resource_overriding(over);
            }
        }

        Some(bundle)
    }

    /// Translates a message key for a locale using the base catalog with
    /// fallback to other supported locales when the key is missing.
    pub fn translate(&self, locale: &str, key: &str, args: &[(&str, Arg)]) -> String {
        if let Some(out) = self.translate_in(locale, key, args, false) {
            return out;
        }
        // Fallback chain: try each supported locale in turn.
        for fallback in catalog::FALLBACK_CHAIN {
            if fallback == locale {
                continue;
            }
            if let Some(out) = self.translate_in(fallback, key, args, false) {
                return out;
            }
        }
        // Key not found anywhere — return the key itself.
        key.to_string()
    }

    fn translate_in(
        &self,
        locale: &str,
        key: &str,
        args: &[(&str, Arg)],
        include_unresolved: bool,
    ) -> Option<String> {
        let bundle = self.bundle_for(locale)?;
        let msg = bundle.get_message(key)?;
        let value = msg.value()?;

        let mut fluent_args = FluentArgs::new();
        for (name, arg) in args {
            match arg {
                Arg::Number(n) => fluent_args.set(*name, *n),
                Arg::Text(s) => fluent_args.set(*name, s.clone()),
            };
        }
        let mut errors = Vec::new();
        let out = bundle.format_pattern(value, Some(&fluent_args), &mut errors);

        if include_unresolved {
            Some(out.into_owned())
        } else {
            // If the pattern resolved to the key itself or was unresolved,
            // treat as a miss so the caller can fall back.
            if out == key {
                None
            } else {
                Some(out.into_owned())
            }
        }
    }

    /// Returns the complete resolved catalog (key -> value) for a locale,
    /// including all fallback keys and applied overrides. Used by the
    /// `GET /i18n/translations` endpoint and the frontend.
    pub fn full_catalog(&self, locale: &str) -> HashMap<String, String> {
        let mut out = HashMap::new();
        for key in catalog::ALL_KEYS {
            out.insert(key.to_string(), self.translate(locale, key, &[]));
        }
        // Overrides take precedence (covered by translate via bundle overrides),
        // but apply them explicitly too for keys with only an override.
        if let Ok(guard) = self.overrides.lock() {
            for (k, v) in guard.iter() {
                if let Some((l, key)) = k.split_once("::")
                    && l == locale
                {
                    out.insert(key.to_string(), v.clone());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> I18nEngine {
        I18nEngine::new()
    }

    #[test]
    fn negotiates_accept_language() {
        assert_eq!(locales::negotiate("en-US,en;q=0.9,es;q=0.8", "es"), "en");
        assert_eq!(locales::negotiate("ar;q=0.9,en;q=0.5", "es"), "ar");
        assert_eq!(locales::negotiate("fr-FR;q=0.8", "es"), "fr");
        // Unsupported / empty falls back to default.
        assert_eq!(locales::negotiate("de-DE", "es"), "es");
        assert_eq!(locales::negotiate("", "es"), "es");
    }

    #[test]
    fn translates_known_key() {
        let e = engine();
        assert_eq!(e.translate("es", "nav-wallet", &[]), "Cartera");
        assert_eq!(e.translate("pt", "auth-login", &[]), "Entrar");
        assert_eq!(e.translate("ar", "nav-notifications", &[]), "الإشعارات");
        assert_eq!(e.translate("fr", "common-cancel", &[]), "Annuler");
    }

    #[test]
    fn falls_back_when_key_missing() {
        let e = engine();
        // 'stream-viewer-count' exists in es; request in a locale that would
        // miss only if key absent — here fallback returns some locale's value.
        let value = e.translate("pt", "nav-stream", &[]);
        assert!(!value.is_empty());
        // A totally unknown key returns itself.
        assert_eq!(e.translate("en", "no-such-key", &[]), "no-such-key");
    }

    #[test]
    fn applies_plural_rules() {
        let e = engine();
        let one = e.translate("en", "notification-count", &[("n", Arg::Number(1))]);
        let many = e.translate("en", "notification-count", &[("n", Arg::Number(5))]);
        assert!(one.contains("notification"));
        assert!(!one.contains("notifications"));
        assert!(many.contains("notifications"));

        // Arabic has a distinct "two" plural form.
        let two_ar = e.translate("ar", "notification-count", &[("n", Arg::Number(2))]);
        assert!(
            two_ar.contains("إشعاران"),
            "arabic dual form not selected: {two_ar}"
        );
    }

    #[test]
    fn full_catalog_includes_target_locale() {
        let e = engine();
        let es = e.full_catalog("es");
        assert!(es.contains_key("nav-wallet"));
        assert_eq!(es.get("nav-wallet").unwrap(), "Cartera");
        let ar = e.full_catalog("ar");
        assert_eq!(ar.get("auth-login").unwrap(), "تسجيل الدخول");
    }

    #[test]
    fn override_takes_precedence() {
        let e = engine();
        let key = "nav-wallet";
        let fk = format!("es::{key}");
        e.overrides
            .lock()
            .unwrap()
            .insert(fk, "Mi Cartera".to_string());
        assert_eq!(e.translate("es", key, &[]), "Mi Cartera");
        assert_eq!(e.full_catalog("es").get(key).unwrap(), "Mi Cartera");
        // Other locale unaffected.
        assert_eq!(e.translate("en", key, &[]), "Wallet");
    }

    #[test]
    fn rtl_metadata() {
        assert!(locales::is_rtl("ar"));
        assert!(!locales::is_rtl("en"));
        assert!(!locales::is_rtl("es"));
        let meta = locales::meta_for("ar").unwrap();
        assert_eq!(meta.dir, "rtl");
    }

    #[test]
    fn number_currency_date_formatting() {
        assert_eq!(format::number("en", 1234567), "1,234,567");
        assert_eq!(format::number("es", 1234567), "1.234.567");
        assert_eq!(format::currency("en", 1250), "1,250 $");
        assert_eq!(format::currency("ar", 1250), "ر.س 1٬250");
        // 0 => Jan 1 1970; verify format doesn't panic and returns a date.
        let d = format::date("en", 0);
        assert!(d.contains("January"));
        let d = format::date("es", 0);
        assert!(d.contains("enero"));
    }
}
