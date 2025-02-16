// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Internationalization
//!
//! Firstly Initialize the language selection with [`init`]. Then use the [`fl`] macro to request a string by id.

use std::sync::LazyLock;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader as _, Localizer,
};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

/// Applies the requested language(s) to be used when calling the `fl!()` macro.
pub fn init(requested_languages: &[LanguageIdentifier]) {
    if let Err(err) = localizer().select(requested_languages) {
        eprintln!("error while loading fluent localizations: {err}");
    }
}

/// Get the `Localizer` to be used for localizing this library.
#[must_use]
fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

/// Request a localized string by ID.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

/// Lowercase the first letter of the string.
pub(crate) fn lowercase_first_char(string: &str) -> String {
    let mut chars = string.chars();
    let Some(uppercase) = chars.next() else {
        return "".into();
    };

    let lowercase = uppercase.to_lowercase().to_string();
    lowercase + chars.collect::<String>().as_str()
}
