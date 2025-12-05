use bits_tailwind_merge::tw_merge;

/// Tests for Tailwind CSS v3.3 features
/// Ported from tailwind-merge/tests/tailwind-css-versions.test.ts
#[cfg(test)]
mod v3_3_features {
    use super::*;

    #[test]
    fn test_line_height_modifiers() {
        assert_eq!(
            tw_merge!("text-red text-lg/7 text-lg/8"),
            "text-red text-lg/8"
        );
    }

    #[test]
    fn test_logical_properties() {
        assert_eq!(
            tw_merge!(
                "start-0 start-1",
                "end-0 end-1",
                "ps-0 ps-1 pe-0 pe-1",
                "ms-0 ms-1 me-0 me-1",
                "rounded-s-sm rounded-s-md rounded-e-sm rounded-e-md",
                "rounded-ss-sm rounded-ss-md rounded-ee-sm rounded-ee-md",
            ),
            "start-1 end-1 ps-1 pe-1 ms-1 me-1 rounded-s-md rounded-e-md rounded-ss-md rounded-ee-md"
        );
    }

    #[test]
    fn test_logical_properties_conflicts() {
        assert_eq!(
            tw_merge!(
                "start-0 end-0 inset-0 ps-0 pe-0 p-0 ms-0 me-0 m-0 rounded-ss rounded-es rounded-s"
            ),
            "inset-0 p-0 m-0 rounded-s"
        );
    }

    #[test]
    fn test_hyphens() {
        assert_eq!(tw_merge!("hyphens-auto hyphens-manual"), "hyphens-manual");
    }

    #[test]
    fn test_gradient_color_stops() {
        assert_eq!(
            tw_merge!(
                "from-0% from-10% from-[12.5%] via-0% via-10% via-[12.5%] to-0% to-10% to-[12.5%]"
            ),
            "from-[12.5%] via-[12.5%] to-[12.5%]"
        );
        assert_eq!(tw_merge!("from-0% from-red"), "from-0% from-red");
    }

    #[test]
    fn test_list_image() {
        assert_eq!(
            tw_merge!("list-image-none list-image-[url(./my-image.png)] list-image-[var(--value)]"),
            "list-image-[var(--value)]"
        );
    }

    #[test]
    fn test_caption_side() {
        assert_eq!(tw_merge!("caption-top caption-bottom"), "caption-bottom");
    }

    #[test]
    fn test_line_clamp() {
        assert_eq!(
            tw_merge!("line-clamp-2 line-clamp-none line-clamp-[10]"),
            "line-clamp-[10]"
        );
    }

    #[test]
    fn test_delay_and_duration() {
        assert_eq!(
            tw_merge!("delay-150 delay-0 duration-150 duration-0"),
            "delay-0 duration-0"
        );
    }

    #[test]
    fn test_justify_and_content_stretch() {
        assert_eq!(
            tw_merge!("justify-normal justify-center justify-stretch"),
            "justify-stretch"
        );
        assert_eq!(
            tw_merge!("content-normal content-center content-stretch"),
            "content-stretch"
        );
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(
            tw_merge!("whitespace-nowrap whitespace-break-spaces"),
            "whitespace-break-spaces"
        );
    }
}

/// Tests for Tailwind CSS v3.4 features
#[cfg(test)]
mod v3_4_features {
    use super::*;

    #[test]
    fn test_dynamic_viewport_units() {
        assert_eq!(tw_merge!("h-svh h-dvh w-svw w-dvw"), "h-dvh w-dvw");
    }

    #[test]
    fn test_has_variant() {
        assert_eq!(
            tw_merge!("has-[[data-potato]]:p-1 has-[[data-potato]]:p-2 group-has-[:checked]:grid group-has-[:checked]:flex"),
            "has-[[data-potato]]:p-2 group-has-[:checked]:flex"
        );
    }

    #[test]
    fn test_text_wrap_pretty() {
        assert_eq!(tw_merge!("text-wrap text-pretty"), "text-pretty");
    }

    #[test]
    fn test_size_utility() {
        assert_eq!(tw_merge!("w-5 h-3 size-10 w-12"), "size-10 w-12");
    }

    #[test]
    fn test_grid_subgrid() {
        assert_eq!(
            tw_merge!("grid-cols-2 grid-cols-subgrid grid-rows-5 grid-rows-subgrid"),
            "grid-cols-subgrid grid-rows-subgrid"
        );
    }

    #[test]
    fn test_min_max_width() {
        assert_eq!(
            tw_merge!("min-w-0 min-w-50 min-w-px max-w-0 max-w-50 max-w-px"),
            "min-w-px max-w-px"
        );
    }

    #[test]
    fn test_forced_color_adjust() {
        assert_eq!(
            tw_merge!("forced-color-adjust-none forced-color-adjust-auto"),
            "forced-color-adjust-auto"
        );
    }

    #[test]
    fn test_appearance() {
        assert_eq!(
            tw_merge!("appearance-none appearance-auto"),
            "appearance-auto"
        );
    }

    #[test]
    fn test_float_and_clear_logical() {
        assert_eq!(
            tw_merge!("float-start float-end clear-start clear-end"),
            "float-end clear-end"
        );
    }

    #[test]
    fn test_star_variant() {
        assert_eq!(
            tw_merge!("*:p-10 *:p-20 hover:*:p-10 hover:*:p-20"),
            "*:p-20 hover:*:p-20"
        );
    }
}

/// Tests for Tailwind CSS v4.0 features
#[cfg(test)]
mod v4_0_features {
    use super::*;

    #[test]
    fn test_3d_transforms() {
        assert_eq!(tw_merge!("transform-3d transform-flat"), "transform-flat");
    }

    #[test]
    fn test_multi_axis_rotation() {
        assert_eq!(
            tw_merge!("rotate-12 rotate-x-2 rotate-none rotate-y-3"),
            "rotate-x-2 rotate-none rotate-y-3"
        );
    }

    #[test]
    fn test_perspective() {
        assert_eq!(
            tw_merge!("perspective-dramatic perspective-none perspective-midrange"),
            "perspective-midrange"
        );
    }

    #[test]
    fn test_perspective_origin() {
        assert_eq!(
            tw_merge!("perspective-origin-center perspective-origin-top-left"),
            "perspective-origin-top-left"
        );
    }

    #[test]
    fn test_gradient_directions() {
        assert_eq!(tw_merge!("bg-linear-to-r bg-linear-45"), "bg-linear-45");
        assert_eq!(
            tw_merge!("bg-linear-to-r bg-radial-[something] bg-conic-10"),
            "bg-conic-10"
        );
    }

    #[test]
    fn test_inset_ring() {
        assert_eq!(
            tw_merge!("ring-4 ring-orange inset-ring inset-ring-3 inset-ring-blue"),
            "ring-4 ring-orange inset-ring-3 inset-ring-blue"
        );
    }

    #[test]
    fn test_field_sizing() {
        assert_eq!(
            tw_merge!("field-sizing-content field-sizing-fixed"),
            "field-sizing-fixed"
        );
    }

    #[test]
    fn test_color_scheme() {
        assert_eq!(tw_merge!("scheme-normal scheme-dark"), "scheme-dark");
    }

    #[test]
    fn test_font_stretch() {
        assert_eq!(
            tw_merge!("font-stretch-expanded font-stretch-[66.66%] font-stretch-50%"),
            "font-stretch-50%"
        );
    }

    #[test]
    fn test_span_shortcuts() {
        assert_eq!(
            tw_merge!("col-span-full col-2 row-span-3 row-4"),
            "col-2 row-4"
        );
    }

    #[test]
    fn test_css_variables_in_gradients() {
        assert_eq!(
            tw_merge!("via-red-500 via-(--mobile-header-gradient)"),
            "via-(--mobile-header-gradient)"
        );
        assert_eq!(
            tw_merge!("via-red-500 via-(length:--mobile-header-gradient)"),
            "via-red-500 via-(length:--mobile-header-gradient)"
        );
    }
}

/// Tests for Tailwind CSS v4.1 features
#[cfg(test)]
mod v4_1_features {
    use super::*;

    #[test]
    fn test_baseline_last() {
        assert_eq!(
            tw_merge!("items-baseline items-baseline-last"),
            "items-baseline-last"
        );
        assert_eq!(
            tw_merge!("self-baseline self-baseline-last"),
            "self-baseline-last"
        );
    }

    #[test]
    fn test_safe_alignment() {
        assert_eq!(
            tw_merge!("place-content-center place-content-end-safe place-content-center-safe"),
            "place-content-center-safe"
        );
        assert_eq!(
            tw_merge!("items-center-safe items-baseline items-end-safe"),
            "items-end-safe"
        );
    }

    #[test]
    fn test_word_wrap() {
        assert_eq!(
            tw_merge!("wrap-break-word wrap-normal wrap-anywhere"),
            "wrap-anywhere"
        );
    }

    #[test]
    fn test_text_shadow() {
        assert_eq!(
            tw_merge!("text-shadow-none text-shadow-2xl"),
            "text-shadow-2xl"
        );
        assert_eq!(
            tw_merge!("text-shadow-none text-shadow-md text-shadow-red text-shadow-red-500 shadow-red shadow-3xs"),
            "text-shadow-md text-shadow-red-500 shadow-red shadow-3xs"
        );
    }

    #[test]
    fn test_mask_composite() {
        assert_eq!(tw_merge!("mask-add mask-subtract"), "mask-subtract");
    }

    #[test]
    fn test_mask_utilities() {
        assert_eq!(
            tw_merge!(
                "mask-(--foo) mask-[foo] mask-none",
                "mask-linear-1 mask-linear-2",
                "mask-linear-from-[position:test] mask-linear-from-3",
                "mask-linear-to-[position:test] mask-linear-to-3",
                "mask-linear-from-color-red mask-linear-from-color-3",
                "mask-linear-to-color-red mask-linear-to-color-3",
                "mask-t-from-[position:test] mask-t-from-3",
                "mask-t-to-[position:test] mask-t-to-3",
                "mask-t-from-color-red mask-t-from-color-3",
                "mask-radial-(--test) mask-radial-[test]",
                "mask-radial-from-[position:test] mask-radial-from-3",
                "mask-radial-to-[position:test] mask-radial-to-3",
                "mask-radial-from-color-red mask-radial-from-color-3",
            ),
            "mask-none mask-linear-2 mask-linear-from-3 mask-linear-to-3 mask-linear-from-color-3 mask-linear-to-color-3 mask-t-from-3 mask-t-to-3 mask-t-from-color-3 mask-radial-[test] mask-radial-from-3 mask-radial-to-3 mask-radial-from-color-3"
        );
    }

    #[test]
    fn test_mask_position() {
        assert_eq!(
            tw_merge!(
                "mask-(--something) mask-[something]",
                "mask-top-left mask-center mask-(position:--var) mask-[position:1px_1px] mask-position-(--var) mask-position-[1px_1px]",
            ),
            "mask-[something] mask-position-[1px_1px]"
        );
    }

    #[test]
    fn test_mask_size() {
        assert_eq!(
            tw_merge!(
                "mask-(--something) mask-[something]",
                "mask-auto mask-[size:foo] mask-(size:--foo) mask-size-[foo] mask-size-(--foo) mask-cover mask-contain",
            ),
            "mask-[something] mask-contain"
        );
    }

    #[test]
    fn test_mask_type() {
        assert_eq!(
            tw_merge!("mask-type-luminance mask-type-alpha"),
            "mask-type-alpha"
        );
    }

    #[test]
    fn test_shadow_opacity() {
        assert_eq!(
            tw_merge!("shadow-md shadow-lg/25 text-shadow-md text-shadow-lg/25"),
            "shadow-lg/25 text-shadow-lg/25"
        );
    }

    #[test]
    fn test_drop_shadow_variants() {
        assert_eq!(
            tw_merge!(
                "drop-shadow-some-color drop-shadow-[#123456] drop-shadow-lg drop-shadow-[10px_0]"
            ),
            "drop-shadow-[#123456] drop-shadow-[10px_0]"
        );
        assert_eq!(
            tw_merge!("drop-shadow-[#123456] drop-shadow-some-color"),
            "drop-shadow-some-color"
        );
        assert_eq!(
            tw_merge!("drop-shadow-2xl drop-shadow-[shadow:foo]"),
            "drop-shadow-[shadow:foo]"
        );
    }
}

/// Tests for Tailwind CSS v4.1.5 features
#[cfg(test)]
mod v4_1_5_features {
    use super::*;

    #[test]
    fn test_line_height_unit() {
        assert_eq!(tw_merge!("h-12 h-lh"), "h-lh");
        assert_eq!(tw_merge!("min-h-12 min-h-lh"), "min-h-lh");
        assert_eq!(tw_merge!("max-h-12 max-h-lh"), "max-h-lh");
    }
}
