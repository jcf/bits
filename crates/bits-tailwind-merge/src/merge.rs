use crate::config::{ClassDef, Config};
use crate::parser::{parse_class_name, IMPORTANT_MODIFIER};
use std::collections::HashSet;

/// Merge multiple class strings with conflict resolution
pub fn tw_merge_slice(class_list: &[&str]) -> String {
    let config = Config::default_v4();
    merge_class_list(&class_list.join(" "), &config)
}

/// Merge a single class list string with conflict resolution
pub fn tw_merge(classes: &str) -> String {
    let config = Config::default_v4();
    merge_class_list(classes, &config)
}

/// Core merge algorithm - processes classes right-to-left
fn merge_class_list(class_list: &str, config: &Config) -> String {
    // Split on whitespace
    let class_names: Vec<&str> = class_list.split_whitespace().collect();

    // Track which class IDs we've seen (to detect conflicts)
    let mut class_groups_in_conflict: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    // Process right-to-left (later classes win)
    for original_class_name in class_names.iter().rev() {
        let parsed = parse_class_name(original_class_name);

        // External classes pass through without merging
        if parsed.is_external {
            result.push(*original_class_name);
            continue;
        }

        // Try to get class group ID
        let has_postfix_modifier = parsed.maybe_postfix_modifier_position.is_some();
        let base_for_lookup = if has_postfix_modifier {
            if let Some(pos) = parsed.maybe_postfix_modifier_position {
                &parsed.base_class_name[..pos]
            } else {
                parsed.base_class_name
            }
        } else {
            parsed.base_class_name
        };

        let mut class_group_id = get_class_group_id(base_for_lookup, config);

        // If not found and we had a postfix modifier, try without it
        if class_group_id.is_none() && has_postfix_modifier {
            class_group_id = get_class_group_id(parsed.base_class_name, config);
        }

        // If still not found, it's not a Tailwind class - pass through
        if class_group_id.is_none() {
            result.push(*original_class_name);
            continue;
        }

        let class_group_id = class_group_id.unwrap();

        // Build the class ID: modifiers + important + classGroupId
        let variant_modifier = if parsed.modifiers.is_empty() {
            String::new()
        } else {
            parsed.modifiers.join(":")
        };

        let modifier_id = if parsed.has_important_modifier {
            format!("{}{}", variant_modifier, IMPORTANT_MODIFIER)
        } else {
            variant_modifier.clone()
        };

        let class_id = format!("{}{}", modifier_id, class_group_id);

        // Check if we've already seen a conflicting class
        if class_groups_in_conflict.contains(&class_id) {
            // Skip this class - we already have a later one
            continue;
        }

        // Mark this class as seen
        class_groups_in_conflict.insert(class_id.clone());

        // Also mark the opposite important variant (important conflicts with non-important)
        if parsed.has_important_modifier {
            // If this has important, also mark the non-important version
            let non_important_id = format!("{}{}", variant_modifier, class_group_id);
            class_groups_in_conflict.insert(non_important_id);
        } else {
            // If this doesn't have important, also mark the important version
            let important_id = format!(
                "{}{}{}",
                variant_modifier, IMPORTANT_MODIFIER, class_group_id
            );
            class_groups_in_conflict.insert(important_id);
        }

        // Mark all conflicting groups as seen
        if let Some(conflicts) = get_conflicting_class_group_ids(&class_group_id, config) {
            for conflict in conflicts {
                let conflict_id = format!("{}{}", modifier_id, conflict);
                class_groups_in_conflict.insert(conflict_id);

                // Also mark the important variant of the conflict
                let conflict_important_id =
                    format!("{}{}{}", variant_modifier, IMPORTANT_MODIFIER, conflict);
                class_groups_in_conflict.insert(conflict_important_id);
            }
        }

        // This class is not in conflict - include it
        result.push(*original_class_name);
    }

    // Reverse back to original order and join
    result.reverse();
    result.join(" ")
}

/// Get the class group ID for a given class name
fn get_class_group_id(class_name: &str, config: &Config) -> Option<String> {
    for (group_id, class_defs) in &config.class_groups {
        if matches_class_def(class_name, class_defs) {
            return Some(group_id.clone());
        }
    }
    None
}

/// Check if a class name matches any of the class definitions
fn matches_class_def(class_name: &str, defs: &[ClassDef]) -> bool {
    for def in defs {
        if matches_single_def(class_name, def) {
            return true;
        }
    }
    false
}

/// Check if a class name matches a single class definition
fn matches_single_def(class_name: &str, def: &ClassDef) -> bool {
    match def {
        ClassDef::Literal(lit) => class_name == lit,
        ClassDef::Validator(validator) => validator(class_name),
        ClassDef::Object(obj) => {
            // For objects, we need to check if the class starts with any key
            for (prefix, values) in obj {
                // Handle empty prefix (matches everything)
                if prefix.is_empty() {
                    return matches_class_def(class_name, values);
                }

                // Check if class starts with "prefix-"
                let full_prefix = format!("{}-", prefix);
                if let Some(suffix) = class_name.strip_prefix(&full_prefix) {
                    if matches_class_def(suffix, values) {
                        return true;
                    }
                }

                // Also check exact match with prefix
                if class_name == prefix {
                    // For exact match, check if any value is empty string
                    if values
                        .iter()
                        .any(|v| matches!(v, ClassDef::Literal(s) if s.is_empty()))
                    {
                        return true;
                    }
                }
            }
            false
        }
    }
}

/// Get conflicting class groups for a given class group ID
fn get_conflicting_class_group_ids(class_group_id: &str, config: &Config) -> Option<Vec<String>> {
    config.conflicting_class_groups.get(class_group_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_merge() {
        assert_eq!(tw_merge("p-4 p-2"), "p-2");
        assert_eq!(tw_merge("px-2 py-1 p-3"), "p-3");
    }

    #[test]
    fn test_border_merge() {
        assert_eq!(tw_merge("border border-2"), "border-2");
        assert_eq!(tw_merge("border-2 border"), "border");
    }

    #[test]
    fn test_refinement() {
        // Refinements are allowed - more specific class after general
        assert_eq!(tw_merge("p-4 py-2"), "p-4 py-2");
    }

    #[test]
    fn test_modifiers() {
        assert_eq!(tw_merge("hover:p-4 hover:p-2"), "hover:p-2");
        assert_eq!(tw_merge("p-4 hover:p-2"), "p-4 hover:p-2");
    }

    #[test]
    fn test_important() {
        assert_eq!(tw_merge("p-4 p-2!"), "p-2!");
        assert_eq!(tw_merge("p-4! p-2"), "p-2");
    }

    #[test]
    fn test_colors() {
        assert_eq!(tw_merge("bg-red-500 bg-blue-500"), "bg-blue-500");
        assert_eq!(tw_merge("text-red-500 text-blue-500"), "text-blue-500");
    }

    #[test]
    fn test_sizing() {
        assert_eq!(tw_merge("w-full w-1/2"), "w-1/2");
        assert_eq!(tw_merge("h-screen h-full"), "h-full");
    }

    #[test]
    fn test_arbitrary_values() {
        assert_eq!(tw_merge("bg-red-500 bg-[#B91C1C]"), "bg-[#B91C1C]");
        assert_eq!(tw_merge("w-full w-[500px]"), "w-[500px]");
    }

    #[test]
    fn test_non_tailwind_classes() {
        assert_eq!(tw_merge("my-custom-class p-4"), "my-custom-class p-4");
        assert_eq!(tw_merge("foo bar p-4 p-2 baz"), "foo bar p-2 baz");
    }

    #[test]
    fn test_multiple_classes() {
        assert_eq!(
            tw_merge("flex items-center justify-center p-4 hover:bg-red-500"),
            "flex items-center justify-center p-4 hover:bg-red-500"
        );
    }
}
