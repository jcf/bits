use dioxus::prelude::*;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

/// Localization context that provides access to translated strings
#[derive(Clone)]
pub struct LocaleContext {
    locale: LanguageIdentifier,
    #[allow(clippy::arc_with_non_send_sync)]
    bundle: Arc<FluentBundle<FluentResource>>,
}

impl LocaleContext {
    /// Create a new LocaleContext with the given locale and Fluent resources
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(locale: LanguageIdentifier, ftl_string: &str) -> Result<Self, String> {
        let resource =
            FluentResource::try_new(ftl_string.to_string()).map_err(|e| format!("{:?}", e))?;

        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("{:?}", e))?;

        Ok(Self {
            locale,
            bundle: Arc::new(bundle),
        })
    }

    /// Get the current locale
    pub fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }

    /// Translate a message by its ID
    pub fn t(&self, message_id: &str) -> String {
        let message = match self.bundle.get_message(message_id) {
            Some(msg) => msg,
            None => return format!("Missing translation: {}", message_id),
        };

        let pattern = match message.value() {
            Some(val) => val,
            None => return format!("Missing value for: {}", message_id),
        };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, None, &mut errors);

        #[cfg(not(target_arch = "wasm32"))]
        if !errors.is_empty() {
            tracing::warn!("Translation errors for {}: {:?}", message_id, errors);
        }

        value.to_string()
    }

    /// Translate a message with arguments
    pub fn t_with_args<'a>(&self, message_id: &str, args: FluentArgs<'a>) -> String {
        let message = match self.bundle.get_message(message_id) {
            Some(msg) => msg,
            None => return format!("Missing translation: {}", message_id),
        };

        let pattern = match message.value() {
            Some(val) => val,
            None => return format!("Missing value for: {}", message_id),
        };

        let mut errors = vec![];
        let value = self
            .bundle
            .format_pattern(pattern, Some(&args), &mut errors);

        #[cfg(not(target_arch = "wasm32"))]
        if !errors.is_empty() {
            tracing::warn!("Translation errors for {}: {:?}", message_id, errors);
        }

        value.to_string()
    }
}

/// Create a default English locale context with embedded resources
pub fn create_default_locale() -> Result<LocaleContext, String> {
    let locale: LanguageIdentifier = "en-US".parse().expect("Failed to parse locale");
    let ftl_string = include_str!("../resources/locales/en-US/main.ftl");
    LocaleContext::new(locale, ftl_string)
}

/// Parse a locale from an Accept-Language header or similar string
pub fn parse_locale(locale_str: &str) -> Option<LanguageIdentifier> {
    locale_str.parse().ok()
}

/// Hook to access the translation function
pub fn use_translation() -> LocaleContext {
    use_context::<LocaleContext>()
}
