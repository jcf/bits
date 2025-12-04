/// The important modifier character in Tailwind v4
pub const IMPORTANT_MODIFIER: char = '!';

/// The modifier separator character
const MODIFIER_SEPARATOR: char = ':';

/// Represents a parsed Tailwind class name
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedClassName<'a> {
    /// Whether the class is external (should skip merging logic)
    pub is_external: bool,
    /// Modifiers in the order they appear (e.g., ["hover", "dark"])
    pub modifiers: Vec<&'a str>,
    /// Whether the class has an important modifier (!)
    pub has_important_modifier: bool,
    /// Base class name without modifiers
    pub base_class_name: &'a str,
    /// Position of possible postfix modifier (e.g., the `/` in `bg-gray-100/50`)
    pub maybe_postfix_modifier_position: Option<usize>,
}

/// Parse a class name into its components
///
/// This parses a Tailwind class name, extracting:
/// - Modifiers (hover:, dark:, etc.)
/// - Important modifier (!)
/// - Base class name
/// - Postfix modifier position (for opacity, e.g., bg-red-500/50)
///
/// Inspired by `splitAtTopLevelOnly` used in Tailwind CSS
/// https://github.com/tailwindlabs/tailwindcss/blob/v3.2.2/src/util/splitAtTopLevelOnly.js
pub fn parse_class_name(class_name: &str) -> ParsedClassName {
    let mut modifiers = Vec::new();
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut modifier_start = 0;
    let mut postfix_modifier_position = None;

    let chars: Vec<char> = class_name.chars().collect();
    let len = chars.len();

    for index in 0..len {
        let current_char = chars[index];

        if bracket_depth == 0 && paren_depth == 0 {
            if current_char == MODIFIER_SEPARATOR {
                modifiers.push(&class_name[modifier_start..index]);
                modifier_start = index + 1;
                continue;
            }

            if current_char == '/' {
                postfix_modifier_position = Some(index);
                continue;
            }
        }

        match current_char {
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }
    }

    let base_class_name_with_important = if modifiers.is_empty() {
        class_name
    } else {
        &class_name[modifier_start..]
    };

    // Check for important modifier
    // In Tailwind v4, the important modifier is at the END of the base class name
    // In Tailwind v3 (legacy), it was at the START - we support both
    let (base_class_name, has_important_modifier, important_shift) =
        if base_class_name_with_important.ends_with(IMPORTANT_MODIFIER) {
            (
                &base_class_name_with_important[..base_class_name_with_important.len() - 1],
                true,
                1, // We removed 1 char from the end
            )
        } else if base_class_name_with_important.starts_with(IMPORTANT_MODIFIER) {
            // Legacy Tailwind v3 support
            (&base_class_name_with_important[1..], true, 1)
        } else {
            (base_class_name_with_important, false, 0)
        };

    // Adjust postfix modifier position if we found one
    let maybe_postfix_modifier_position = postfix_modifier_position
        .filter(|&pos| pos > modifier_start)
        .map(|pos| {
            let relative_pos = pos - modifier_start;
            // If important was at the end and postfix is before it, adjust
            if has_important_modifier
                && important_shift > 0
                && relative_pos <= base_class_name.len()
            {
                relative_pos
            } else {
                relative_pos
            }
        });

    ParsedClassName {
        is_external: false,
        modifiers,
        has_important_modifier,
        base_class_name,
        maybe_postfix_modifier_position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_class() {
        let parsed = parse_class_name("flex");
        assert_eq!(parsed.base_class_name, "flex");
        assert!(parsed.modifiers.is_empty());
        assert!(!parsed.has_important_modifier);
        assert_eq!(parsed.maybe_postfix_modifier_position, None);
    }

    #[test]
    fn test_class_with_modifiers() {
        let parsed = parse_class_name("hover:dark:bg-gray-100");
        assert_eq!(parsed.base_class_name, "bg-gray-100");
        assert_eq!(parsed.modifiers, vec!["hover", "dark"]);
        assert!(!parsed.has_important_modifier);
    }

    #[test]
    fn test_important_modifier_v4() {
        let parsed = parse_class_name("bg-red-500!");
        assert_eq!(parsed.base_class_name, "bg-red-500");
        assert!(parsed.has_important_modifier);
    }

    #[test]
    fn test_important_modifier_v3_legacy() {
        let parsed = parse_class_name("!bg-red-500");
        assert_eq!(parsed.base_class_name, "bg-red-500");
        assert!(parsed.has_important_modifier);
    }

    #[test]
    fn test_postfix_modifier() {
        let parsed = parse_class_name("bg-gray-100/50");
        assert_eq!(parsed.base_class_name, "bg-gray-100/50");
        assert_eq!(parsed.maybe_postfix_modifier_position, Some(11));
    }

    #[test]
    fn test_arbitrary_value() {
        let parsed = parse_class_name("bg-[#B91C1C]");
        assert_eq!(parsed.base_class_name, "bg-[#B91C1C]");
    }

    #[test]
    fn test_complex_with_modifiers_and_important() {
        let parsed = parse_class_name("hover:focus:bg-red-500!");
        assert_eq!(parsed.base_class_name, "bg-red-500");
        assert_eq!(parsed.modifiers, vec!["hover", "focus"]);
        assert!(parsed.has_important_modifier);
    }
}
