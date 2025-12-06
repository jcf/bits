#![forbid(unsafe_code)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::fn_params_excessive_bools)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::unneeded_field_pattern)]
#![deny(clippy::wildcard_enum_match_arm)]

mod config;
mod merge;
mod parser;
mod trie;
mod validator;

pub use merge::{tw_merge, tw_merge_slice};

/// Join Tailwind classes without conflict resolution.
///
/// This macro joins multiple class strings with whitespace,
/// filtering out empty strings. No conflict resolution is performed.
///
/// # Example
///
/// ```
/// use bits_tailwind_merge::tw_join;
///
/// assert_eq!(
///     tw_join!("flex", "items-center", "justify-center"),
///     "flex items-center justify-center"
/// );
/// ```
#[macro_export]
macro_rules! tw_join {
    ($($class:expr),* $(,)?) => {{
        let mut result = String::new();
        $(
            let s = $class;
            if !s.is_empty() {
                for class in s.split_whitespace() {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(class);
                }
            }
        )*
        result
    }};
}

/// Merge Tailwind classes with conflict resolution.
///
/// This macro merges multiple class strings, resolving conflicts between
/// Tailwind utility classes. Later classes override earlier ones when they
/// belong to the same class group.
///
/// # Example
///
/// ```
/// use bits_tailwind_merge::tw_merge;
///
/// // Conflicts are resolved - p-4 overrides py-2 px-4
/// assert_eq!(
///     tw_merge!("py-2 px-4", "p-4"),
///     "p-4"
/// );
///
/// // Refinements are allowed - py-2 refines p-4
/// assert_eq!(
///     tw_merge!("p-4", "py-2"),
///     "p-4 py-2"
/// );
/// ```
#[macro_export]
macro_rules! tw_merge {
    ($($class:expr),* $(,)?) => {{
        $crate::tw_merge_slice(&[$($class),*])
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tw_join() {
        assert_eq!(tw_join!("flex", "items-center"), "flex items-center");
        assert_eq!(tw_join!("flex", "", "items-center"), "flex items-center");
        assert_eq!(
            tw_join!("flex gap-4", "items-center"),
            "flex gap-4 items-center"
        );
    }

    #[test]
    fn test_tw_merge_basic() {
        // Basic conflict resolution
        assert_eq!(tw_merge!("p-4", "p-2"), "p-2");
        assert_eq!(tw_merge!("px-2 py-1", "p-3"), "p-3");
    }

    #[test]
    fn test_size_conflicts_debug() {
        assert_eq!(tw_merge!("w-full h-full", "size-10"), "size-10");
    }
}
