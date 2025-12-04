use bits_tailwind_merge::{tw_join, tw_merge};

#[test]
fn test_tw_join_basic() {
    assert_eq!(tw_join!("flex", "items-center"), "flex items-center");
    assert_eq!(
        tw_join!("flex", "items-center", "justify-center"),
        "flex items-center justify-center"
    );
}

#[test]
fn test_tw_join_with_empty() {
    assert_eq!(tw_join!("flex", "", "items-center"), "flex items-center");
    assert_eq!(tw_join!("", "flex", ""), "flex");
}

#[test]
fn test_tw_join_multiple_classes_in_string() {
    assert_eq!(
        tw_join!("flex gap-4", "items-center justify-center"),
        "flex gap-4 items-center justify-center"
    );
}

// Basic conflict resolution tests
#[test]
fn test_merge_basic_conflicts() {
    // Later class wins
    assert_eq!(tw_merge!("p-4", "p-2"), "p-2");
    assert_eq!(tw_merge!("m-4", "m-2"), "m-2");
}

#[test]
fn test_merge_compound_conflicts() {
    // Compound classes conflict with individual ones
    assert_eq!(tw_merge!("px-2 py-1", "p-3"), "p-3");
    assert_eq!(tw_merge!("mx-2 my-1", "m-3"), "m-3");
}

#[test]
fn test_merge_refinements() {
    // More specific classes after general ones are refinements (allowed)
    assert_eq!(tw_merge!("p-4", "py-2"), "p-4 py-2");
    assert_eq!(tw_merge!("m-4", "mx-2"), "m-4 mx-2");
}

#[test]
fn test_merge_reverse_refinements() {
    // General class after specific should override
    assert_eq!(tw_merge!("py-2", "p-4"), "p-4");
    assert_eq!(tw_merge!("px-2 py-2", "p-4"), "p-4");
}

// Modifier tests
#[test]
fn test_merge_with_modifiers() {
    assert_eq!(tw_merge!("hover:p-4", "hover:p-2"), "hover:p-2");
    assert_eq!(tw_merge!("focus:p-4", "focus:p-2"), "focus:p-2");
    assert_eq!(tw_merge!("dark:bg-black", "dark:bg-white"), "dark:bg-white");
}

#[test]
fn test_merge_different_modifiers() {
    // Different modifiers don't conflict
    assert_eq!(tw_merge!("hover:p-4", "focus:p-2"), "hover:p-4 focus:p-2");
    assert_eq!(tw_merge!("p-4", "hover:p-2"), "p-4 hover:p-2");
}

#[test]
fn test_merge_multiple_modifiers() {
    assert_eq!(
        tw_merge!("hover:dark:p-4", "hover:dark:p-2"),
        "hover:dark:p-2"
    );
}

// Important modifier tests
#[test]
fn test_merge_important_v4_syntax() {
    assert_eq!(tw_merge!("p-4", "p-2!"), "p-2!");
    assert_eq!(tw_merge!("p-4!", "p-2"), "p-2");
    assert_eq!(tw_merge!("p-4!", "p-2!"), "p-2!");
}

#[test]
fn test_merge_important_with_modifiers() {
    assert_eq!(tw_merge!("hover:p-4!", "hover:p-2!"), "hover:p-2!");
    assert_eq!(tw_merge!("hover:p-4", "hover:p-2!"), "hover:p-2!");
}

// Color tests
#[test]
fn test_merge_colors() {
    assert_eq!(tw_merge!("bg-red-500", "bg-blue-500"), "bg-blue-500");
    assert_eq!(tw_merge!("text-red-500", "text-blue-500"), "text-blue-500");
    assert_eq!(
        tw_merge!("bg-red-500 text-white", "bg-blue-500"),
        "text-white bg-blue-500"
    );
}

// Sizing tests
#[test]
fn test_merge_sizing() {
    assert_eq!(tw_merge!("w-full", "w-1/2"), "w-1/2");
    assert_eq!(tw_merge!("h-screen", "h-full"), "h-full");
    assert_eq!(tw_merge!("w-1/2", "w-full"), "w-full");
}

#[test]
fn test_merge_size_conflicts() {
    // size utility conflicts with w and h when size comes last
    assert_eq!(tw_merge!("w-full h-full", "size-10"), "size-10");
    // Refinement: w after size is allowed (more specific wins)
    assert_eq!(tw_merge!("size-10", "w-full"), "size-10 w-full");
}

// Arbitrary value tests
#[test]
fn test_merge_arbitrary_values() {
    assert_eq!(tw_merge!("bg-red-500", "bg-[#B91C1C]"), "bg-[#B91C1C]");
    assert_eq!(tw_merge!("w-full", "w-[500px]"), "w-[500px]");
    assert_eq!(tw_merge!("p-4", "p-[20px]"), "p-[20px]");
}

#[test]
fn test_merge_arbitrary_values_conflict() {
    assert_eq!(tw_merge!("bg-[#B91C1C]", "bg-[#DC2626]"), "bg-[#DC2626]");
}

// Non-Tailwind class tests
#[test]
fn test_merge_preserves_non_tailwind_classes() {
    assert_eq!(tw_merge!("my-custom-class", "p-4"), "my-custom-class p-4");
    assert_eq!(
        tw_merge!("foo", "bar", "p-4", "p-2", "baz"),
        "foo bar p-2 baz"
    );
}

// Complex real-world examples
#[test]
fn test_merge_complex_button_example() {
    assert_eq!(
        tw_merge!(
            "flex items-center justify-center p-4 bg-blue-500 hover:bg-blue-600",
            "p-2 bg-red-500"
        ),
        "flex items-center justify-center hover:bg-blue-600 p-2 bg-red-500"
    );
}

#[test]
fn test_merge_responsive_classes() {
    assert_eq!(tw_merge!("text-base", "md:text-lg"), "text-base md:text-lg");
    assert_eq!(tw_merge!("p-4", "sm:p-6", "md:p-8"), "p-4 sm:p-6 md:p-8");
}

#[test]
fn test_merge_position_conflicts() {
    // Refinement: inset-x after inset is more specific
    assert_eq!(tw_merge!("inset-0", "inset-x-1"), "inset-0 inset-x-1");
    // General inset overrides more specific
    assert_eq!(tw_merge!("inset-x-1", "inset-0"), "inset-0");
    // inset overrides individual positioning when it comes last
    assert_eq!(tw_merge!("top-0 right-0", "inset-0"), "inset-0");
}

#[test]
fn test_merge_gap_conflicts() {
    assert_eq!(tw_merge!("gap-4", "gap-x-2"), "gap-4 gap-x-2");
    assert_eq!(tw_merge!("gap-x-2 gap-y-4", "gap-8"), "gap-8");
}

// Edge cases
#[test]
fn test_merge_empty_string() {
    assert_eq!(tw_merge!(""), "");
    assert_eq!(tw_merge!("", ""), "");
    assert_eq!(tw_merge!("p-4", ""), "p-4");
}

#[test]
fn test_merge_whitespace() {
    assert_eq!(tw_merge!("  p-4  ", "  p-2  "), "p-2");
    assert_eq!(tw_merge!("p-4\n\t", "p-2"), "p-2");
}

#[test]
fn test_merge_duplicate_classes() {
    assert_eq!(tw_merge!("p-4 p-4"), "p-4");
    assert_eq!(tw_merge!("p-4", "p-4"), "p-4");
}

// Display classes
#[test]
fn test_merge_display_classes() {
    assert_eq!(tw_merge!("block", "flex"), "flex");
    assert_eq!(tw_merge!("flex", "inline-flex"), "inline-flex");
    assert_eq!(tw_merge!("hidden", "block"), "block");
}

// Flexbox
#[test]
fn test_merge_flexbox() {
    assert_eq!(tw_merge!("flex-row", "flex-col"), "flex-col");
    assert_eq!(tw_merge!("items-start", "items-center"), "items-center");
    assert_eq!(
        tw_merge!("justify-start", "justify-center"),
        "justify-center"
    );
}

// Border radius
#[test]
fn test_merge_border_radius() {
    assert_eq!(tw_merge!("rounded", "rounded-lg"), "rounded-lg");
    assert_eq!(tw_merge!("rounded-lg", "rounded-none"), "rounded-none");
}

// Border width
#[test]
fn test_merge_border_width() {
    // Both "border" and "border-2" are in the same class group
    assert_eq!(tw_merge!("border", "border-2"), "border-2");
    assert_eq!(tw_merge!("border-2", "border"), "border");
}

// Shadow
#[test]
fn test_merge_shadow() {
    assert_eq!(tw_merge!("shadow", "shadow-lg"), "shadow-lg");
    assert_eq!(tw_merge!("shadow-lg", "shadow-none"), "shadow-none");
}

// Opacity
#[test]
fn test_merge_opacity() {
    assert_eq!(tw_merge!("opacity-50", "opacity-100"), "opacity-100");
    assert_eq!(tw_merge!("opacity-0", "opacity-50"), "opacity-50");
}

// Font weight
#[test]
fn test_merge_font_weight() {
    assert_eq!(tw_merge!("font-normal", "font-bold"), "font-bold");
    assert_eq!(tw_merge!("font-bold", "font-light"), "font-light");
}

// Font size
#[test]
fn test_merge_font_size() {
    assert_eq!(tw_merge!("text-base", "text-lg"), "text-lg");
    assert_eq!(tw_merge!("text-sm", "text-xl"), "text-xl");
}
