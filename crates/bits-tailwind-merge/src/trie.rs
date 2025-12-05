use crate::config::{ClassDef, ClassValidator, Config};
use std::collections::HashMap;

/// A trie node for efficient class group matching
#[derive(Default)]
pub struct TrieNode {
    /// Child nodes indexed by character
    children: HashMap<char, Box<TrieNode>>,

    /// Class group ID and prefix length if this node represents a valid match
    /// Multiple entries because the same prefix can match multiple groups
    matches: Vec<(String, usize)>,

    /// Validators that should be checked at this node
    /// (group_id, validator, prefix_length)
    validators: Vec<(String, ClassValidator, usize)>,
}

impl TrieNode {
    /// Find the best matching class group ID for a class name
    /// Returns (group_id, prefix_length) for the longest matching prefix
    pub fn find_match(&self, class_name: &str) -> Option<(String, usize)> {
        self.find_match_internal(class_name)
    }

    /// Internal recursive match for finding the longest matching prefix
    fn find_match_internal(&self, remaining: &str) -> Option<(String, usize)> {
        let mut best_match: Option<(String, usize)> = None;

        // Check validators at current node - these can match partial remaining strings
        for (group_id, validator, base_len) in &self.validators {
            if validator(remaining) {
                // The stored base_len already includes the full prefix length
                if best_match.is_none() || *base_len > best_match.as_ref().unwrap().1 {
                    best_match = Some((group_id.clone(), *base_len));
                }
            }
        }

        // Check exact matches at current node
        // These represent complete class prefixes (e.g., "text-shadow", "border-r")
        // We check them if:
        // 1. We've consumed the entire input (remaining is empty), OR
        // 2. The remaining part starts with '-' (for prefix matching like "text-shadow" in "text-shadow-md")
        //
        // This prevents matching "border-r" when searching "border-red-500" (remaining="ed-500"),
        // while still matching "text-shadow" when searching "text-shadow-md" (remaining="-md")
        if remaining.is_empty() || remaining.starts_with('-') {
            for (group_id, stored_len) in &self.matches {
                if best_match.is_none() || *stored_len > best_match.as_ref().unwrap().1 {
                    best_match = Some((group_id.clone(), *stored_len));
                }
            }
        }

        // Try to traverse further
        if let Some(first_char) = remaining.chars().next() {
            if let Some(child) = self.children.get(&first_char) {
                let char_len = first_char.len_utf8();
                let new_remaining = &remaining[char_len..];
                if let Some(child_match) = child.find_match_internal(new_remaining) {
                    if best_match.is_none() || child_match.1 > best_match.as_ref().unwrap().1 {
                        best_match = Some(child_match);
                    }
                }
            }
        }

        best_match
    }

    /// Insert a prefix into the trie
    fn insert_prefix(&mut self, prefix: &str, group_id: String, prefix_len: usize) {
        if prefix.is_empty() {
            self.matches.push((group_id, prefix_len));
            return;
        }

        let mut chars = prefix.chars();
        let first_char = chars.next().unwrap();
        let remaining = chars.as_str();

        let child = self
            .children
            .entry(first_char)
            .or_insert_with(|| Box::new(TrieNode::default()));
        child.insert_prefix(remaining, group_id, prefix_len);
    }

    /// Add a validator at the current node
    fn add_validator(&mut self, group_id: String, validator: ClassValidator, prefix_len: usize) {
        self.validators.push((group_id, validator, prefix_len));
    }
}

/// Build a trie from the config for fast class group lookups
pub fn build_class_group_trie(config: &Config) -> TrieNode {
    let mut root = TrieNode::default();

    for (group_id, class_defs) in &config.class_groups {
        for class_def in class_defs {
            insert_class_def(&mut root, group_id, class_def, 0);
        }
    }

    root
}

/// Recursively insert a ClassDef into the trie
fn insert_class_def(node: &mut TrieNode, group_id: &str, def: &ClassDef, base_prefix_len: usize) {
    match def {
        ClassDef::Literal(lit) => {
            // Insert literal as exact prefix match
            node.insert_prefix(lit, group_id.to_string(), base_prefix_len + lit.len());
        }
        ClassDef::Validator(validator) => {
            // Add validator at current node
            node.add_validator(group_id.to_string(), *validator, base_prefix_len);
        }
        ClassDef::Object(obj) => {
            // Collect exact matches first (before we start mutating)
            let exact_matches: Vec<_> = obj
                .iter()
                .filter(|(prefix, values)| {
                    !prefix.is_empty()
                        && values
                            .iter()
                            .any(|v| matches!(v, ClassDef::Literal(s) if s.is_empty()))
                })
                .map(|(prefix, _)| (prefix.clone(), base_prefix_len + prefix.len()))
                .collect();

            // Handle empty prefix cases
            for (prefix, values) in obj {
                if prefix.is_empty() {
                    for value in values {
                        insert_class_def(node, group_id, value, base_prefix_len);
                    }
                }
            }

            // Handle prefixed cases
            for (prefix, values) in obj {
                if !prefix.is_empty() {
                    // Navigate to prefix node and insert values there
                    let full_prefix = format!("{}-", prefix);

                    // Navigate through the prefix
                    let mut current = &mut *node;
                    for ch in full_prefix.chars() {
                        current = current
                            .children
                            .entry(ch)
                            .or_insert_with(|| Box::new(TrieNode::default()));
                    }

                    // Insert values at the prefix node
                    for value in values {
                        insert_class_def(
                            current,
                            group_id,
                            value,
                            base_prefix_len + full_prefix.len(),
                        );
                    }
                }
            }

            // Finally add exact matches
            for (prefix, prefix_len) in exact_matches {
                node.insert_prefix(&prefix, group_id.to_string(), prefix_len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_trie() {
        let mut root = TrieNode::default();
        root.insert_prefix("flex", "display".to_string(), 4);
        root.insert_prefix("block", "display".to_string(), 5);

        assert_eq!(root.find_match("flex"), Some(("display".to_string(), 4)));
        assert_eq!(root.find_match("block"), Some(("display".to_string(), 5)));
        assert_eq!(root.find_match("unknown"), None);
    }

    #[test]
    fn test_prefix_specificity() {
        let mut root = TrieNode::default();
        root.insert_prefix("text", "text-color".to_string(), 4);
        root.insert_prefix("text-shadow", "text-shadow".to_string(), 11);

        // Should match the longest prefix
        let result = root.find_match("text-shadow-md");
        assert_eq!(
            result.as_ref().map(|(id, _)| id.as_str()),
            Some("text-shadow")
        );
        assert!(result.unwrap().1 >= 11);
    }
}
