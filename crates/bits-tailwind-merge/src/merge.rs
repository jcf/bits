use crate::config::Config;
use crate::parser::{parse_class_name, IMPORTANT_MODIFIER};
use std::collections::HashSet;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::default_v4)
}

/// Merge multiple class strings with conflict resolution
pub fn tw_merge_slice(class_list: &[&str]) -> String {
    merge_class_list(&class_list.join(" "), get_config())
}

/// Merge a single class list string with conflict resolution
pub fn tw_merge(classes: &str) -> String {
    merge_class_list(classes, get_config())
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
        // Sort modifiers to ensure consistent conflict detection regardless of order
        let variant_modifier = if parsed.modifiers.is_empty() {
            String::new()
        } else if parsed.modifiers.len() == 1 {
            parsed.modifiers[0].to_string()
        } else {
            // Sort modifiers alphabetically (preserving arbitrary variants)
            let mut sorted_mods = parsed.modifiers.clone();
            sorted_mods.sort_unstable();
            sorted_mods.join(":")
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

/// Get the class group ID for a given class name using the trie
/// Finds the most specific match (longest matching prefix) in O(m) time
fn get_class_group_id(class_name: &str, config: &Config) -> Option<String> {
    config
        .class_group_trie
        .find_match(class_name)
        .map(|(group_id, _)| group_id)
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
