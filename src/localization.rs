use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use rust_embed::RustEmbed;
use unicode_bidi::BidiInfo;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader.load_fallback_language(&Localizations).expect("Error while loading fallback language");

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localization::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localization::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

/// Get the `Localizer` to be used for localizing this library.
#[must_use]
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

/// Get translated text
pub fn get_locale_text(message_id: &str) -> String {
    LANGUAGE_LOADER.get(message_id)
}

#[must_use]
pub fn is_rtl() -> bool {
    std::env::var("LANGUAGE")
        .ok()
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LC_MESSAGES").ok())
        .or_else(|| std::env::var("LANG").ok())
        .is_some_and(|language| language.split(':').any(|value| value.starts_with("ar")))
}

#[must_use]
pub fn terminal_text(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if !is_rtl() || !text.chars().any(|character| ('\u{0600}'..='\u{06ff}').contains(&character)) {
        return text.to_owned();
    }

    visual_order(text)
}

fn visual_order(text: &str) -> String {
    let bidi_info = BidiInfo::new(text, None);
    let paragraph = &bidi_info.paragraphs[0];
    bidi_info.reorder_line(paragraph, paragraph.range.clone()).into_owned()
}

#[cfg(test)]
mod tests {
    use super::visual_order;

    #[test]
    fn arabic_is_rendered_in_visual_order() {
        assert_eq!(visual_order("الاسم"), "مسالا");
        assert_eq!(visual_order("صلاحيات root مطلوبة"), "ةبولطم root تايحالص");
    }

    #[test]
    fn terminal_text_does_not_emit_bidi_controls() {
        let rendered = visual_order("الأولوية");
        assert!(!rendered.chars().any(|character| ('\u{2066}'..='\u{2069}').contains(&character)));
    }
}
