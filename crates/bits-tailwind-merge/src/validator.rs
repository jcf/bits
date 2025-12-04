use regex::Regex;
use std::sync::OnceLock;

static ARBITRARY_VALUE_REGEX: OnceLock<Regex> = OnceLock::new();
static ARBITRARY_VARIABLE_REGEX: OnceLock<Regex> = OnceLock::new();
static FRACTION_REGEX: OnceLock<Regex> = OnceLock::new();
static TSHIRT_UNIT_REGEX: OnceLock<Regex> = OnceLock::new();
static LENGTH_UNIT_REGEX: OnceLock<Regex> = OnceLock::new();
static COLOR_FUNCTION_REGEX: OnceLock<Regex> = OnceLock::new();
static SHADOW_REGEX: OnceLock<Regex> = OnceLock::new();
#[allow(dead_code)]
static IMAGE_REGEX: OnceLock<Regex> = OnceLock::new();

fn arbitrary_value_regex() -> &'static Regex {
    ARBITRARY_VALUE_REGEX.get_or_init(|| Regex::new(r"^\[(?:(\w[\w-]*):)?(.+)\]$").unwrap())
}

fn arbitrary_variable_regex() -> &'static Regex {
    ARBITRARY_VARIABLE_REGEX.get_or_init(|| Regex::new(r"^\((?:(\w[\w-]*):)?(.+)\)$").unwrap())
}

fn fraction_regex() -> &'static Regex {
    FRACTION_REGEX.get_or_init(|| Regex::new(r"^\d+/\d+$").unwrap())
}

fn tshirt_unit_regex() -> &'static Regex {
    TSHIRT_UNIT_REGEX.get_or_init(|| Regex::new(r"^(\d+(\.\d+)?)?(xs|sm|md|lg|xl)$").unwrap())
}

fn length_unit_regex() -> &'static Regex {
    LENGTH_UNIT_REGEX.get_or_init(|| {
        Regex::new(r"\d+(%|px|r?em|[sdl]?v([hwib]|min|max)|pt|pc|in|cm|mm|cap|ch|ex|r?lh|cq(w|h|i|b|min|max))|\b(calc|min|max|clamp)\(.+\)|^0$").unwrap()
    })
}

fn color_function_regex() -> &'static Regex {
    COLOR_FUNCTION_REGEX
        .get_or_init(|| Regex::new(r"^(rgba?|hsla?|hwb|(ok)?(lab|lch)|color-mix)\(.+\)$").unwrap())
}

fn shadow_regex() -> &'static Regex {
    SHADOW_REGEX.get_or_init(|| {
        Regex::new(r"^(inset_)?-?((\d+)?\.?(\d+)[a-z]+|0)_-?((\d+)?\.?(\d+)[a-z]+|0)").unwrap()
    })
}

#[allow(dead_code)]
fn image_regex() -> &'static Regex {
    IMAGE_REGEX.get_or_init(|| {
        Regex::new(r"^(url|image|image-set|cross-fade|element|(repeating-)?(linear|radial|conic)-gradient)\(.+\)$").unwrap()
    })
}

pub fn is_fraction(value: &str) -> bool {
    fraction_regex().is_match(value)
}

pub fn is_number(value: &str) -> bool {
    !value.is_empty() && value.parse::<f64>().is_ok()
}

#[allow(dead_code)]
pub fn is_integer(value: &str) -> bool {
    !value.is_empty() && value.parse::<i64>().is_ok()
}

#[allow(dead_code)]
pub fn is_percent(value: &str) -> bool {
    value.ends_with('%') && is_number(&value[..value.len() - 1])
}

pub fn is_tshirt_size(value: &str) -> bool {
    tshirt_unit_regex().is_match(value)
}

pub fn is_any(_value: &str) -> bool {
    true
}

#[allow(dead_code)]
pub fn is_any_non_arbitrary(value: &str) -> bool {
    !is_arbitrary_value(value) && !is_arbitrary_variable(value)
}

fn is_length_only(value: &str) -> bool {
    // colorFunctionRegex check is necessary because color functions can have percentages
    // in them which would be incorrectly classified as lengths.
    // For example, `hsl(0 0% 0%)` would be classified as a length without this check.
    length_unit_regex().is_match(value) && !color_function_regex().is_match(value)
}

fn is_shadow(value: &str) -> bool {
    shadow_regex().is_match(value)
}

#[allow(dead_code)]
fn is_image(value: &str) -> bool {
    image_regex().is_match(value)
}

pub fn is_arbitrary_value(value: &str) -> bool {
    arbitrary_value_regex().is_match(value)
}

pub fn is_arbitrary_length(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_length, is_length_only)
}

pub fn is_arbitrary_number(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_number, is_number)
}

#[allow(dead_code)]
pub fn is_arbitrary_size(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_size, |_| false)
}

#[allow(dead_code)]
pub fn is_arbitrary_position(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_position, |_| false)
}

#[allow(dead_code)]
pub fn is_arbitrary_image(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_image, is_image)
}

pub fn is_arbitrary_shadow(value: &str) -> bool {
    get_is_arbitrary_value(value, is_label_shadow, is_shadow)
}

pub fn is_arbitrary_variable(value: &str) -> bool {
    arbitrary_variable_regex().is_match(value)
}

pub fn is_arbitrary_variable_length(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_length, false)
}

#[allow(dead_code)]
pub fn is_arbitrary_variable_family_name(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_family_name, false)
}

#[allow(dead_code)]
pub fn is_arbitrary_variable_position(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_position, false)
}

#[allow(dead_code)]
pub fn is_arbitrary_variable_size(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_size, false)
}

#[allow(dead_code)]
pub fn is_arbitrary_variable_image(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_image, false)
}

pub fn is_arbitrary_variable_shadow(value: &str) -> bool {
    get_is_arbitrary_variable(value, is_label_shadow, true)
}

// Helper functions

fn get_is_arbitrary_value<F, G>(value: &str, test_label: F, test_value: G) -> bool
where
    F: Fn(&str) -> bool,
    G: Fn(&str) -> bool,
{
    if let Some(caps) = arbitrary_value_regex().captures(value) {
        if let Some(label) = caps.get(1) {
            return test_label(label.as_str());
        }
        if let Some(val) = caps.get(2) {
            return test_value(val.as_str());
        }
    }
    false
}

fn get_is_arbitrary_variable<F>(value: &str, test_label: F, should_match_no_label: bool) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(caps) = arbitrary_variable_regex().captures(value) {
        if let Some(label) = caps.get(1) {
            return test_label(label.as_str());
        }
        return should_match_no_label;
    }
    false
}

// Label validators

#[allow(dead_code)]
fn is_label_position(label: &str) -> bool {
    label == "position" || label == "percentage"
}

#[allow(dead_code)]
fn is_label_image(label: &str) -> bool {
    label == "image" || label == "url"
}

#[allow(dead_code)]
fn is_label_size(label: &str) -> bool {
    label == "length" || label == "size" || label == "bg-size"
}

fn is_label_length(label: &str) -> bool {
    label == "length"
}

fn is_label_number(label: &str) -> bool {
    label == "number"
}

#[allow(dead_code)]
fn is_label_family_name(label: &str) -> bool {
    label == "family-name"
}

fn is_label_shadow(label: &str) -> bool {
    label == "shadow"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fraction() {
        assert!(is_fraction("1/2"));
        assert!(is_fraction("3/4"));
        assert!(!is_fraction("1.5"));
        assert!(!is_fraction("full"));
    }

    #[test]
    fn test_is_number() {
        assert!(is_number("0"));
        assert!(is_number("42"));
        assert!(is_number("3.14"));
        assert!(!is_number("abc"));
    }

    #[test]
    fn test_is_integer() {
        assert!(is_integer("0"));
        assert!(is_integer("42"));
        assert!(!is_integer("3.14"));
        assert!(!is_integer("abc"));
    }

    #[test]
    fn test_is_percent() {
        assert!(is_percent("50%"));
        assert!(is_percent("100%"));
        assert!(!is_percent("50"));
        assert!(!is_percent("abc%"));
    }

    #[test]
    fn test_is_tshirt_size() {
        assert!(is_tshirt_size("xs"));
        assert!(is_tshirt_size("sm"));
        assert!(is_tshirt_size("md"));
        assert!(is_tshirt_size("lg"));
        assert!(is_tshirt_size("xl"));
        assert!(is_tshirt_size("2xl"));
        assert!(!is_tshirt_size("abc"));
    }

    #[test]
    fn test_is_arbitrary_value() {
        assert!(is_arbitrary_value("[100px]"));
        assert!(is_arbitrary_value("[#B91C1C]"));
        assert!(is_arbitrary_value("[length:100px]"));
        assert!(!is_arbitrary_value("100px"));
    }

    #[test]
    fn test_is_arbitrary_variable() {
        assert!(is_arbitrary_variable("(--my-var)"));
        assert!(is_arbitrary_variable("(color:--my-color)"));
        assert!(!is_arbitrary_variable("--my-var"));
    }

    #[test]
    fn test_is_arbitrary_length() {
        assert!(is_arbitrary_length("[100px]"));
        assert!(is_arbitrary_length("[length:100px]"));
        assert!(!is_arbitrary_length("[#fff]"));
    }
}
