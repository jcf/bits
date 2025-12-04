use bits_tailwind_merge::{tw_merge, tw_merge_slice};
use proptest::prelude::*;

// Generators for Tailwind class components

/// Generate valid Tailwind class prefixes
fn class_prefix() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("p".to_string()),
        Just("m".to_string()),
        Just("px".to_string()),
        Just("py".to_string()),
        Just("mx".to_string()),
        Just("my".to_string()),
        Just("w".to_string()),
        Just("h".to_string()),
        Just("bg".to_string()),
        Just("text".to_string()),
        Just("border".to_string()),
        Just("rounded".to_string()),
        Just("flex".to_string()),
        Just("gap".to_string()),
        Just("inset".to_string()),
        Just("top".to_string()),
        Just("size".to_string()),
    ]
}

/// Generate valid Tailwind values for spacing (padding/margin accept these)
fn spacing_value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("0".to_string()),
        Just("1".to_string()),
        Just("2".to_string()),
        Just("4".to_string()),
        Just("8".to_string()),
        Just("px".to_string()),
    ]
}

/// Generate valid values for margin (includes auto)
fn margin_value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("0".to_string()),
        Just("1".to_string()),
        Just("2".to_string()),
        Just("4".to_string()),
        Just("auto".to_string()),
    ]
}

/// Generate valid values for sizing (includes auto, full)
fn sizing_value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("0".to_string()),
        Just("1".to_string()),
        Just("4".to_string()),
        Just("auto".to_string()),
        Just("full".to_string()),
    ]
}

/// Generate valid Tailwind values for colors
fn color_value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("red-500".to_string()),
        Just("blue-500".to_string()),
        Just("green-500".to_string()),
    ]
}

/// Generate valid Tailwind values for alignment
fn align_value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("center".to_string()),
        Just("start".to_string()),
        Just("end".to_string()),
    ]
}

/// Generate a simple Tailwind class (prefix-value)
fn simple_tailwind_class() -> impl Strategy<Value = String> {
    prop_oneof![
        // Padding classes (no auto)
        (
            prop_oneof![Just("p"), Just("px"), Just("py"), Just("pt"), Just("pr"), Just("pb"), Just("pl")],
            spacing_value()
        )
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Margin classes (includes auto)
        (
            prop_oneof![Just("m"), Just("mx"), Just("my"), Just("mt"), Just("mr"), Just("mb"), Just("ml")],
            margin_value()
        )
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Sizing classes (includes auto, full)
        (prop_oneof![Just("w"), Just("h")], sizing_value())
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Color classes
        (prop_oneof![Just("bg"), Just("text")], color_value())
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Position classes (includes auto, full)
        (
            prop_oneof![Just("top"), Just("left"), Just("right"), Just("bottom"), Just("inset")],
            sizing_value()
        )
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Alignment classes
        (prop_oneof![Just("items"), Just("justify")], align_value())
            .prop_map(|(p, v)| format!("{}-{}", p, v)),
        // Simple classes
        Just("flex".to_string()),
        Just("block".to_string()),
        Just("hidden".to_string()),
    ]
}

/// Generate Tailwind modifiers
fn modifier() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("hover".to_string()),
        Just("focus".to_string()),
        Just("active".to_string()),
        Just("dark".to_string()),
        Just("sm".to_string()),
        Just("md".to_string()),
        Just("lg".to_string()),
    ]
}

/// Generate a class with modifiers
fn class_with_modifiers() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(modifier(), 0..=3),
        simple_tailwind_class(),
        prop::bool::ANY,
    )
        .prop_map(|(mods, class, important)| {
            let mut result = class;
            if important {
                result.push('!');
            }
            if mods.is_empty() {
                result
            } else {
                format!("{}:{}", mods.join(":"), result)
            }
        })
}

/// Generate arbitrary values
fn arbitrary_value_class() -> impl Strategy<Value = String> {
    (class_prefix(), "[0-9]{1,3}(px|rem|%)")
        .prop_map(|(prefix, value)| format!("{}-[{}]", prefix, value))
}

/// Generate any valid Tailwind class
fn any_tailwind_class() -> impl Strategy<Value = String> {
    prop_oneof![
        simple_tailwind_class(),
        class_with_modifiers(),
        arbitrary_value_class(),
    ]
}

/// Generate non-Tailwind classes (custom classes)
fn custom_class() -> impl Strategy<Value = String> {
    "[a-z]{3,10}(-[a-z]{3,10}){0,2}".prop_map(|s| format!("custom-{}", s))
}

/// Generate a class string (space-separated classes)
fn class_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            3 => any_tailwind_class(),
            1 => custom_class(),
        ],
        1..20,
    )
    .prop_map(|classes| classes.join(" "))
}

// Property tests

proptest! {
    /// Test that output only contains classes from input
    #[test]
    fn prop_output_is_subset_of_input(input in class_string()) {
        let output = tw_merge(&input);
        let input_classes: Vec<&str> = input.split_whitespace().collect();
        let output_classes: Vec<&str> = output.split_whitespace().collect();

        for out_class in &output_classes {
            prop_assert!(
                input_classes.contains(out_class),
                "Output class '{}' not found in input", out_class
            );
        }
    }

    /// Test that non-Tailwind classes always pass through
    #[test]
    fn prop_custom_classes_preserved(
        custom in custom_class(),
        tailwind in class_string(),
    ) {
        let input = format!("{} {}", custom, tailwind);
        let output = tw_merge(&input);

        prop_assert!(
            output.contains(&custom),
            "Custom class '{}' was removed from output", custom
        );
    }

    /// Test idempotency: merging output again should give same result
    #[test]
    fn prop_idempotent(input in class_string()) {
        let output1 = tw_merge(&input);
        let output2 = tw_merge(&output1);

        prop_assert_eq!(output1, output2, "Merge is not idempotent");
    }

    /// Test that later duplicates override earlier ones
    #[test]
    fn prop_later_wins(class in simple_tailwind_class()) {
        let input = format!("{} {}", class, class);
        let output = tw_merge(&input);

        // Should only appear once
        let count = output.split_whitespace()
            .filter(|c| *c == class)
            .count();

        prop_assert_eq!(count, 1, "Duplicate class not merged");
    }

    /// Test that merging preserves the "later wins" semantic
    /// Note: tw_merge is NOT associative due to conflict resolution
    #[test]
    fn prop_flatten_equivalence(
        a in class_string(),
        b in class_string(),
        c in class_string(),
    ) {
        // Merging separately then together should equal merging all at once
        let separate = tw_merge_slice(&[&a, &b, &c]);
        let nested = tw_merge(&format!("{} {} {}", a, b, c));

        prop_assert_eq!(separate, nested, "Merge results differ");
    }

    /// Test that important modifier is preserved
    #[test]
    fn prop_important_preserved(class in simple_tailwind_class()) {
        let important_class = format!("{}!", class);
        let output = tw_merge(&important_class);

        prop_assert!(
            output.contains('!'),
            "Important modifier was lost"
        );
    }

    /// Test that modifiers are preserved
    #[test]
    fn prop_modifiers_preserved(
        mods in prop::collection::vec(modifier(), 1..=3),
        class in simple_tailwind_class(),
    ) {
        let input = format!("{}:{}", mods.join(":"), class);
        let output = tw_merge(&input);

        for modifier in &mods {
            prop_assert!(
                output.contains(modifier),
                "Modifier '{}' was lost", modifier
            );
        }
    }

    /// Test that same modifiers with different base classes don't conflict
    #[test]
    fn prop_different_base_no_conflict(
        modifier in modifier(),
        class1 in class_prefix(),
        class2 in class_prefix(),
    ) {
        prop_assume!(class1 != class2);

        let input = format!("{}:{}-4 {}:{}-2", modifier, class1, modifier, class2);
        let output = tw_merge(&input);
        let output_classes: Vec<&str> = output.split_whitespace().collect();

        // Both should be present if they're different properties
        // (e.g., hover:p-4 hover:m-2 shouldn't conflict)
        if class1.starts_with('p') && class2.starts_with('m') {
            prop_assert_eq!(
                output_classes.len(), 2,
                "Non-conflicting classes were merged"
            );
        }
    }

    /// Test pathological: very long class strings
    #[test]
    fn prop_handles_long_strings(
        classes in prop::collection::vec(any_tailwind_class(), 50..100)
    ) {
        let input = classes.join(" ");
        let output = tw_merge(&input);

        // Should not panic and should return something
        prop_assert!(!output.is_empty() || input.split_whitespace().all(|c| c.is_empty()));
    }

    /// Test pathological: many duplicates
    #[test]
    fn prop_handles_many_duplicates(class in simple_tailwind_class()) {
        let input = vec![class.as_str(); 100].join(" ");
        let output = tw_merge(&input);

        // Should deduplicate to single instance
        let count = output.split_whitespace()
            .filter(|c| *c == class)
            .count();
        prop_assert_eq!(count, 1, "Failed to deduplicate 100 instances to 1");
    }

    /// Test pathological: deeply nested modifiers
    #[test]
    fn prop_handles_deep_modifiers(
        mods in prop::collection::vec(modifier(), 5..10),
        class in simple_tailwind_class(),
    ) {
        let input = format!("{}:{}", mods.join(":"), class);
        let output = tw_merge(&input);

        // Should preserve all modifiers
        for modifier in &mods {
            prop_assert!(
                output.contains(modifier),
                "Lost modifier '{}' in deep nesting", modifier
            );
        }
    }

    /// Test that empty input gives empty output
    #[test]
    fn prop_empty_input(whitespace in "[ \\t\\n]*") {
        let output = tw_merge(&whitespace);
        prop_assert!(output.is_empty(), "Non-empty output from whitespace");
    }

    /// Test conflict resolution with same group
    #[test]
    fn prop_same_group_conflicts(
        (prefix, val1, val2) in prop_oneof![
            // Padding (no auto)
            (Just("p"), spacing_value(), spacing_value()),
            // Margin (has auto)
            (Just("m"), margin_value(), margin_value()),
            // Sizing (has auto and full)
            (Just("w"), sizing_value(), sizing_value()),
        ]
    ) {
        prop_assume!(val1 != val2);

        let class1 = format!("{}-{}", prefix, val1);
        let class2 = format!("{}-{}", prefix, val2);
        let input = format!("{} {}", class1, class2);
        let output = tw_merge(&input);

        // Later class should win
        prop_assert!(
            output.contains(&class2),
            "Later class '{}' not in output", class2
        );

        // Earlier class should be removed
        prop_assert!(
            !output.contains(&class1),
            "Earlier class '{}' should be removed", class1
        );
    }

    /// Test that order is preserved for non-conflicting classes
    #[test]
    fn prop_order_preserved_non_conflicting(
        class1 in prop_oneof![
            Just("flex".to_string()),
            Just("p-4".to_string()),
            Just("bg-red-500".to_string()),
        ],
        class2 in prop_oneof![
            Just("items-center".to_string()),
            Just("m-2".to_string()),
            Just("text-blue-500".to_string()),
        ],
    ) {
        prop_assume!(class1 != class2);

        let input = format!("{} {}", class1, class2);
        let output = tw_merge(&input);

        // Both should be present (these are guaranteed non-conflicting)
        prop_assert!(output.contains(&class1), "Lost first class: {}", class1);
        prop_assert!(output.contains(&class2), "Lost second class: {}", class2);

        // Order should be preserved
        let idx1 = output.find(&class1);
        let idx2 = output.find(&class2);

        if let (Some(i1), Some(i2)) = (idx1, idx2) {
            prop_assert!(i1 < i2, "Order was reversed");
        }
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    /// Regression test: ensure specific known inputs produce expected outputs
    #[test]
    fn test_known_pathological_cases() {
        // Very long duplicate string
        let long_dup = vec!["p-4"; 1000].join(" ");
        assert_eq!(tw_merge(&long_dup), "p-4");

        // Mixed valid and invalid
        let mixed = "p-4 invalid-class-123 m-2 another-custom p-2";
        let result = tw_merge(mixed);
        assert!(result.contains("invalid-class-123"));
        assert!(result.contains("another-custom"));
        assert!(result.contains("p-2"));
        assert!(!result.contains("p-4"));

        // Deep modifiers
        let deep = "hover:focus:active:dark:sm:md:lg:p-4";
        let result = tw_merge(deep);
        assert_eq!(result, deep);

        // Arbitrary values with special chars
        let arb = "p-[20px] p-[1.5rem]";
        let result = tw_merge(arb);
        assert_eq!(result, "p-[1.5rem]");
    }
}
