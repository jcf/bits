use crate::trie::{build_class_group_trie, TrieNode};
use crate::validator::{
    is_any, is_any_non_arbitrary, is_arbitrary_image, is_arbitrary_length, is_arbitrary_number,
    is_arbitrary_position, is_arbitrary_shadow, is_arbitrary_size, is_arbitrary_value,
    is_arbitrary_variable, is_arbitrary_variable_family_name, is_arbitrary_variable_image,
    is_arbitrary_variable_length, is_arbitrary_variable_position, is_arbitrary_variable_shadow,
    is_arbitrary_variable_size, is_fraction, is_integer, is_number, is_percent, is_tshirt_size,
};
use indexmap::IndexMap;
use std::collections::HashMap;

/// A class validator function
pub type ClassValidator = fn(&str) -> bool;

/// Represents a class definition - can be a string literal or a validator
#[derive(Clone)]
pub enum ClassDef {
    Literal(String),
    Validator(ClassValidator),
    Object(HashMap<String, Vec<ClassDef>>),
}

/// Configuration for the Tailwind merge system
pub struct Config {
    pub class_groups: IndexMap<String, Vec<ClassDef>>,
    pub conflicting_class_groups: HashMap<String, Vec<String>>,
    pub class_group_trie: TrieNode,
}

impl Config {
    /// Get the default Tailwind v4 configuration
    pub fn default_v4() -> Self {
        let mut class_groups = IndexMap::new();
        let mut conflicting_class_groups = HashMap::new();

        // --------------
        // --- Layout ---
        // --------------

        // Aspect Ratio
        // @see https://tailwindcss.com/docs/aspect-ratio
        add_class_group(
            &mut class_groups,
            "aspect",
            vec![obj(
                "aspect",
                vec![
                    lit("auto"),
                    lit("square"),
                    validator(is_fraction),
                    validator(is_arbitrary_value),
                    validator(is_arbitrary_variable),
                    validator(is_tshirt_size),
                ],
            )],
        );

        // Container
        // @see https://tailwindcss.com/docs/container
        // @deprecated since Tailwind CSS v4.0.0
        add_class_group(&mut class_groups, "container", vec![lit("container")]);

        // Columns
        // @see https://tailwindcss.com/docs/columns
        add_class_group(
            &mut class_groups,
            "columns",
            vec![obj(
                "columns",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_value),
                    validator(is_arbitrary_variable),
                    validator(is_tshirt_size),
                ],
            )],
        );

        // Break After
        // @see https://tailwindcss.com/docs/break-after
        add_class_group(
            &mut class_groups,
            "break-after",
            vec![obj(
                "break-after",
                vec![
                    lit("auto"),
                    lit("avoid"),
                    lit("all"),
                    lit("avoid-page"),
                    lit("page"),
                    lit("left"),
                    lit("right"),
                    lit("column"),
                ],
            )],
        );

        // Break Before
        // @see https://tailwindcss.com/docs/break-before
        add_class_group(
            &mut class_groups,
            "break-before",
            vec![obj(
                "break-before",
                vec![
                    lit("auto"),
                    lit("avoid"),
                    lit("all"),
                    lit("avoid-page"),
                    lit("page"),
                    lit("left"),
                    lit("right"),
                    lit("column"),
                ],
            )],
        );

        // Break Inside
        // @see https://tailwindcss.com/docs/break-inside
        add_class_group(
            &mut class_groups,
            "break-inside",
            vec![obj(
                "break-inside",
                vec![
                    lit("auto"),
                    lit("avoid"),
                    lit("avoid-page"),
                    lit("avoid-column"),
                ],
            )],
        );

        // Box Decoration Break
        // @see https://tailwindcss.com/docs/box-decoration-break
        add_class_group(
            &mut class_groups,
            "box-decoration",
            vec![obj("box-decoration", vec![lit("slice"), lit("clone")])],
        );

        // Box Sizing
        // @see https://tailwindcss.com/docs/box-sizing
        add_class_group(
            &mut class_groups,
            "box",
            vec![obj("box", vec![lit("border"), lit("content")])],
        );

        // Display
        // @see https://tailwindcss.com/docs/display
        add_class_group(
            &mut class_groups,
            "display",
            vec![
                lit("block"),
                lit("inline-block"),
                lit("inline"),
                lit("flex"),
                lit("inline-flex"),
                lit("table"),
                lit("inline-table"),
                lit("table-caption"),
                lit("table-cell"),
                lit("table-column"),
                lit("table-column-group"),
                lit("table-footer-group"),
                lit("table-header-group"),
                lit("table-row-group"),
                lit("table-row"),
                lit("flow-root"),
                lit("grid"),
                lit("inline-grid"),
                lit("contents"),
                lit("list-item"),
                lit("hidden"),
            ],
        );

        // Screen Reader Only
        // @see https://tailwindcss.com/docs/display#screen-reader-only
        add_class_group(
            &mut class_groups,
            "sr",
            vec![lit("sr-only"), lit("not-sr-only")],
        );

        // Floats
        // @see https://tailwindcss.com/docs/float
        add_class_group(
            &mut class_groups,
            "float",
            vec![obj(
                "float",
                vec![
                    lit("right"),
                    lit("left"),
                    lit("none"),
                    lit("start"),
                    lit("end"),
                ],
            )],
        );

        // Clear
        // @see https://tailwindcss.com/docs/clear
        add_class_group(
            &mut class_groups,
            "clear",
            vec![obj(
                "clear",
                vec![
                    lit("left"),
                    lit("right"),
                    lit("both"),
                    lit("none"),
                    lit("start"),
                    lit("end"),
                ],
            )],
        );

        // Isolation
        // @see https://tailwindcss.com/docs/isolation
        add_class_group(
            &mut class_groups,
            "isolation",
            vec![lit("isolate"), lit("isolation-auto")],
        );

        // Object Fit
        // @see https://tailwindcss.com/docs/object-fit
        add_class_group(
            &mut class_groups,
            "object-fit",
            vec![obj(
                "object",
                vec![
                    lit("contain"),
                    lit("cover"),
                    lit("fill"),
                    lit("none"),
                    lit("scale-down"),
                ],
            )],
        );

        // Object Position
        // @see https://tailwindcss.com/docs/object-position
        add_class_group(
            &mut class_groups,
            "object-position",
            vec![obj("object", scale_position_with_arbitrary())],
        );

        // Overflow
        // @see https://tailwindcss.com/docs/overflow
        add_class_group(
            &mut class_groups,
            "overflow",
            vec![obj("overflow", scale_overflow())],
        );

        // Overflow X
        // @see https://tailwindcss.com/docs/overflow
        add_class_group(
            &mut class_groups,
            "overflow-x",
            vec![obj("overflow-x", scale_overflow())],
        );

        // Overflow Y
        // @see https://tailwindcss.com/docs/overflow
        add_class_group(
            &mut class_groups,
            "overflow-y",
            vec![obj("overflow-y", scale_overflow())],
        );

        // Overscroll Behavior
        // @see https://tailwindcss.com/docs/overscroll-behavior
        add_class_group(
            &mut class_groups,
            "overscroll",
            vec![obj("overscroll", scale_overscroll())],
        );

        // Overscroll Behavior X
        // @see https://tailwindcss.com/docs/overscroll-behavior
        add_class_group(
            &mut class_groups,
            "overscroll-x",
            vec![obj("overscroll-x", scale_overscroll())],
        );

        // Overscroll Behavior Y
        // @see https://tailwindcss.com/docs/overscroll-behavior
        add_class_group(
            &mut class_groups,
            "overscroll-y",
            vec![obj("overscroll-y", scale_overscroll())],
        );

        // Position
        // @see https://tailwindcss.com/docs/position
        add_class_group(
            &mut class_groups,
            "position",
            vec![
                lit("static"),
                lit("fixed"),
                lit("absolute"),
                lit("relative"),
                lit("sticky"),
            ],
        );

        // Top / Right / Bottom / Left
        // @see https://tailwindcss.com/docs/top-right-bottom-left
        add_class_group(
            &mut class_groups,
            "inset",
            vec![obj("inset", scale_inset())],
        );
        add_class_group(
            &mut class_groups,
            "inset-x",
            vec![obj("inset-x", scale_inset())],
        );
        add_class_group(
            &mut class_groups,
            "inset-y",
            vec![obj("inset-y", scale_inset())],
        );
        add_class_group(
            &mut class_groups,
            "start",
            vec![obj("start", scale_inset())],
        );
        add_class_group(&mut class_groups, "end", vec![obj("end", scale_inset())]);
        add_class_group(&mut class_groups, "top", vec![obj("top", scale_inset())]);
        add_class_group(
            &mut class_groups,
            "right",
            vec![obj("right", scale_inset())],
        );
        add_class_group(
            &mut class_groups,
            "bottom",
            vec![obj("bottom", scale_inset())],
        );
        add_class_group(&mut class_groups, "left", vec![obj("left", scale_inset())]);

        // Visibility
        // @see https://tailwindcss.com/docs/visibility
        add_class_group(
            &mut class_groups,
            "visibility",
            vec![lit("visible"), lit("invisible"), lit("collapse")],
        );

        // Z-Index
        // @see https://tailwindcss.com/docs/z-index
        add_class_group(
            &mut class_groups,
            "z",
            vec![obj(
                "z",
                vec![
                    validator(is_integer),
                    lit("auto"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // ------------------------
        // --- Flexbox and Grid ---
        // ------------------------

        // Flex Basis
        // @see https://tailwindcss.com/docs/flex-basis
        add_class_group(
            &mut class_groups,
            "basis",
            vec![obj("basis", {
                let mut v = vec![
                    validator(is_fraction),
                    lit("full"),
                    lit("auto"),
                    validator(is_tshirt_size),
                ];
                v.extend(scale_unambiguous_spacing());
                v
            })],
        );

        // Flex Direction
        // @see https://tailwindcss.com/docs/flex-direction
        add_class_group(
            &mut class_groups,
            "flex-direction",
            vec![obj(
                "flex",
                vec![
                    lit("row"),
                    lit("row-reverse"),
                    lit("col"),
                    lit("col-reverse"),
                ],
            )],
        );

        // Flex Wrap
        // @see https://tailwindcss.com/docs/flex-wrap
        add_class_group(
            &mut class_groups,
            "flex-wrap",
            vec![obj(
                "flex",
                vec![lit("nowrap"), lit("wrap"), lit("wrap-reverse")],
            )],
        );

        // Flex
        // @see https://tailwindcss.com/docs/flex
        add_class_group(
            &mut class_groups,
            "flex",
            vec![obj(
                "flex",
                vec![
                    validator(is_number),
                    validator(is_fraction),
                    lit("auto"),
                    lit("initial"),
                    lit("none"),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Flex Grow
        // @see https://tailwindcss.com/docs/flex-grow
        add_class_group(
            &mut class_groups,
            "grow",
            vec![obj(
                "grow",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Flex Shrink
        // @see https://tailwindcss.com/docs/flex-shrink
        add_class_group(
            &mut class_groups,
            "shrink",
            vec![obj(
                "shrink",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Order
        // @see https://tailwindcss.com/docs/order
        add_class_group(
            &mut class_groups,
            "order",
            vec![obj(
                "order",
                vec![
                    validator(is_integer),
                    lit("first"),
                    lit("last"),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Grid Template Columns
        // @see https://tailwindcss.com/docs/grid-template-columns
        add_class_group(
            &mut class_groups,
            "grid-cols",
            vec![obj("grid-cols", scale_grid_template_cols_rows())],
        );

        // Grid Column Start / End
        // @see https://tailwindcss.com/docs/grid-column
        add_class_group(
            &mut class_groups,
            "col-start-end",
            vec![obj("col", scale_grid_col_row_start_and_end())],
        );

        // Grid Column Start
        // @see https://tailwindcss.com/docs/grid-column
        add_class_group(
            &mut class_groups,
            "col-start",
            vec![obj("col-start", scale_grid_col_row_start_or_end())],
        );

        // Grid Column End
        // @see https://tailwindcss.com/docs/grid-column
        add_class_group(
            &mut class_groups,
            "col-end",
            vec![obj("col-end", scale_grid_col_row_start_or_end())],
        );

        // Grid Template Rows
        // @see https://tailwindcss.com/docs/grid-template-rows
        add_class_group(
            &mut class_groups,
            "grid-rows",
            vec![obj("grid-rows", scale_grid_template_cols_rows())],
        );

        // Grid Row Start / End
        // @see https://tailwindcss.com/docs/grid-row
        add_class_group(
            &mut class_groups,
            "row-start-end",
            vec![obj("row", scale_grid_col_row_start_and_end())],
        );

        // Grid Row Start
        // @see https://tailwindcss.com/docs/grid-row
        add_class_group(
            &mut class_groups,
            "row-start",
            vec![obj("row-start", scale_grid_col_row_start_or_end())],
        );

        // Grid Row End
        // @see https://tailwindcss.com/docs/grid-row
        add_class_group(
            &mut class_groups,
            "row-end",
            vec![obj("row-end", scale_grid_col_row_start_or_end())],
        );

        // Grid Auto Flow
        // @see https://tailwindcss.com/docs/grid-auto-flow
        add_class_group(
            &mut class_groups,
            "grid-flow",
            vec![obj(
                "grid-flow",
                vec![
                    lit("row"),
                    lit("col"),
                    lit("dense"),
                    lit("row-dense"),
                    lit("col-dense"),
                ],
            )],
        );

        // Grid Auto Columns
        // @see https://tailwindcss.com/docs/grid-auto-columns
        add_class_group(
            &mut class_groups,
            "auto-cols",
            vec![obj("auto-cols", scale_grid_auto_cols_rows())],
        );

        // Grid Auto Rows
        // @see https://tailwindcss.com/docs/grid-auto-rows
        add_class_group(
            &mut class_groups,
            "auto-rows",
            vec![obj("auto-rows", scale_grid_auto_cols_rows())],
        );

        // Gap
        // @see https://tailwindcss.com/docs/gap
        add_class_group(
            &mut class_groups,
            "gap",
            vec![obj("gap", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "gap-x",
            vec![obj("gap-x", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "gap-y",
            vec![obj("gap-y", scale_unambiguous_spacing())],
        );

        // Justify Content
        // @see https://tailwindcss.com/docs/justify-content
        add_class_group(
            &mut class_groups,
            "justify-content",
            vec![obj("justify", {
                let mut v = scale_align_primary_axis();
                v.push(lit("normal"));
                v
            })],
        );

        // Justify Items
        // @see https://tailwindcss.com/docs/justify-items
        add_class_group(
            &mut class_groups,
            "justify-items",
            vec![obj("justify-items", {
                let mut v = scale_align_secondary_axis();
                v.push(lit("normal"));
                v
            })],
        );

        // Justify Self
        // @see https://tailwindcss.com/docs/justify-self
        add_class_group(
            &mut class_groups,
            "justify-self",
            vec![obj("justify-self", {
                let mut v = vec![lit("auto")];
                v.extend(scale_align_secondary_axis());
                v
            })],
        );

        // Align Content
        // @see https://tailwindcss.com/docs/align-content
        add_class_group(
            &mut class_groups,
            "align-content",
            vec![obj("content", {
                let mut v = vec![lit("normal")];
                v.extend(scale_align_primary_axis());
                v
            })],
        );

        // Align Items
        // @see https://tailwindcss.com/docs/align-items
        add_class_group(
            &mut class_groups,
            "align-items",
            vec![obj("items", {
                let mut v = scale_align_secondary_axis();
                v.push(obj("baseline", vec![lit(""), lit("last")]));
                v
            })],
        );

        // Align Self
        // @see https://tailwindcss.com/docs/align-self
        add_class_group(
            &mut class_groups,
            "align-self",
            vec![obj("self", {
                let mut v = vec![lit("auto")];
                v.extend(scale_align_secondary_axis());
                v.push(obj("baseline", vec![lit(""), lit("last")]));
                v
            })],
        );

        // Place Content
        // @see https://tailwindcss.com/docs/place-content
        add_class_group(
            &mut class_groups,
            "place-content",
            vec![obj("place-content", scale_align_primary_axis())],
        );

        // Place Items
        // @see https://tailwindcss.com/docs/place-items
        add_class_group(
            &mut class_groups,
            "place-items",
            vec![obj("place-items", {
                let mut v = scale_align_secondary_axis();
                v.push(lit("baseline"));
                v
            })],
        );

        // Place Self
        // @see https://tailwindcss.com/docs/place-self
        add_class_group(
            &mut class_groups,
            "place-self",
            vec![obj("place-self", {
                let mut v = vec![lit("auto")];
                v.extend(scale_align_secondary_axis());
                v
            })],
        );

        // Spacing - Padding
        // @see https://tailwindcss.com/docs/padding
        add_class_group(
            &mut class_groups,
            "p",
            vec![obj("p", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "px",
            vec![obj("px", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "py",
            vec![obj("py", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "ps",
            vec![obj("ps", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "pe",
            vec![obj("pe", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "pt",
            vec![obj("pt", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "pr",
            vec![obj("pr", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "pb",
            vec![obj("pb", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "pl",
            vec![obj("pl", scale_unambiguous_spacing())],
        );

        // Spacing - Margin
        // @see https://tailwindcss.com/docs/margin
        add_class_group(&mut class_groups, "m", vec![obj("m", scale_margin())]);
        add_class_group(&mut class_groups, "mx", vec![obj("mx", scale_margin())]);
        add_class_group(&mut class_groups, "my", vec![obj("my", scale_margin())]);
        add_class_group(&mut class_groups, "ms", vec![obj("ms", scale_margin())]);
        add_class_group(&mut class_groups, "me", vec![obj("me", scale_margin())]);
        add_class_group(&mut class_groups, "mt", vec![obj("mt", scale_margin())]);
        add_class_group(&mut class_groups, "mr", vec![obj("mr", scale_margin())]);
        add_class_group(&mut class_groups, "mb", vec![obj("mb", scale_margin())]);
        add_class_group(&mut class_groups, "ml", vec![obj("ml", scale_margin())]);

        // Space Between X
        // @see https://tailwindcss.com/docs/margin#adding-space-between-children
        add_class_group(
            &mut class_groups,
            "space-x",
            vec![obj("space-x", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "space-x-reverse",
            vec![lit("space-x-reverse")],
        );

        // Space Between Y
        // @see https://tailwindcss.com/docs/margin#adding-space-between-children
        add_class_group(
            &mut class_groups,
            "space-y",
            vec![obj("space-y", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "space-y-reverse",
            vec![lit("space-y-reverse")],
        );

        // --------------
        // --- Sizing ---
        // --------------

        // Size
        // @see https://tailwindcss.com/docs/width#setting-both-width-and-height
        add_class_group(&mut class_groups, "size", vec![obj("size", scale_sizing())]);

        // Width
        // @see https://tailwindcss.com/docs/width
        add_class_group(
            &mut class_groups,
            "w",
            vec![obj("w", {
                let mut v = vec![validator(is_tshirt_size), lit("screen")];
                v.extend(scale_sizing());
                v
            })],
        );

        // Min-Width
        // @see https://tailwindcss.com/docs/min-width
        add_class_group(
            &mut class_groups,
            "min-w",
            vec![obj("min-w", {
                let mut v = vec![
                    validator(is_tshirt_size),
                    lit("screen"),
                    lit("none"), // Deprecated
                ];
                v.extend(scale_sizing());
                v
            })],
        );

        // Max-Width
        // @see https://tailwindcss.com/docs/max-width
        add_class_group(
            &mut class_groups,
            "max-w",
            vec![obj("max-w", {
                let mut v = vec![
                    validator(is_tshirt_size),
                    lit("screen"),
                    lit("none"),
                    lit("prose"),                                   // Deprecated
                    obj("screen", vec![validator(is_tshirt_size)]), // Deprecated
                ];
                v.extend(scale_sizing());
                v
            })],
        );

        // Height
        // @see https://tailwindcss.com/docs/height
        add_class_group(
            &mut class_groups,
            "h",
            vec![obj("h", {
                let mut v = vec![lit("screen"), lit("lh")];
                v.extend(scale_sizing());
                v
            })],
        );

        // Min-Height
        // @see https://tailwindcss.com/docs/min-height
        add_class_group(
            &mut class_groups,
            "min-h",
            vec![obj("min-h", {
                let mut v = vec![lit("screen"), lit("lh"), lit("none")];
                v.extend(scale_sizing());
                v
            })],
        );

        // Max-Height
        // @see https://tailwindcss.com/docs/max-height
        add_class_group(
            &mut class_groups,
            "max-h",
            vec![obj("max-h", {
                let mut v = vec![lit("screen"), lit("lh")];
                v.extend(scale_sizing());
                v
            })],
        );

        // ------------------
        // --- Typography ---
        // ------------------

        // Font Size
        // @see https://tailwindcss.com/docs/font-size
        add_class_group(
            &mut class_groups,
            "font-size",
            vec![obj(
                "text",
                vec![
                    lit("base"),
                    validator(is_tshirt_size),
                    validator(is_arbitrary_variable_length),
                    validator(is_arbitrary_length),
                ],
            )],
        );

        // Font Smoothing
        // @see https://tailwindcss.com/docs/font-smoothing
        add_class_group(
            &mut class_groups,
            "font-smoothing",
            vec![lit("antialiased"), lit("subpixel-antialiased")],
        );

        // Font Style
        // @see https://tailwindcss.com/docs/font-style
        add_class_group(
            &mut class_groups,
            "font-style",
            vec![lit("italic"), lit("not-italic")],
        );

        // Font Weight
        // @see https://tailwindcss.com/docs/font-weight
        add_class_group(
            &mut class_groups,
            "font-weight",
            vec![obj(
                "font",
                vec![
                    lit("thin"),
                    lit("extralight"),
                    lit("light"),
                    lit("normal"),
                    lit("medium"),
                    lit("semibold"),
                    lit("bold"),
                    lit("extrabold"),
                    lit("black"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_number),
                ],
            )],
        );

        // Font Stretch
        // @see https://tailwindcss.com/docs/font-stretch
        add_class_group(
            &mut class_groups,
            "font-stretch",
            vec![obj(
                "font-stretch",
                vec![
                    lit("ultra-condensed"),
                    lit("extra-condensed"),
                    lit("condensed"),
                    lit("semi-condensed"),
                    lit("normal"),
                    lit("semi-expanded"),
                    lit("expanded"),
                    lit("extra-expanded"),
                    lit("ultra-expanded"),
                    validator(is_percent),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Font Family
        // @see https://tailwindcss.com/docs/font-family
        add_class_group(
            &mut class_groups,
            "font-family",
            vec![obj(
                "font",
                vec![
                    validator(is_arbitrary_variable_family_name),
                    validator(is_arbitrary_value),
                    validator(is_any_non_arbitrary),
                ],
            )],
        );

        // Font Variant Numeric
        // @see https://tailwindcss.com/docs/font-variant-numeric
        add_class_group(&mut class_groups, "fvn-normal", vec![lit("normal-nums")]);
        add_class_group(&mut class_groups, "fvn-ordinal", vec![lit("ordinal")]);
        add_class_group(
            &mut class_groups,
            "fvn-slashed-zero",
            vec![lit("slashed-zero")],
        );
        add_class_group(
            &mut class_groups,
            "fvn-figure",
            vec![lit("lining-nums"), lit("oldstyle-nums")],
        );
        add_class_group(
            &mut class_groups,
            "fvn-spacing",
            vec![lit("proportional-nums"), lit("tabular-nums")],
        );
        add_class_group(
            &mut class_groups,
            "fvn-fraction",
            vec![lit("diagonal-fractions"), lit("stacked-fractions")],
        );

        // Letter Spacing
        // @see https://tailwindcss.com/docs/letter-spacing
        add_class_group(
            &mut class_groups,
            "tracking",
            vec![obj(
                "tracking",
                vec![
                    lit("tighter"),
                    lit("tight"),
                    lit("normal"),
                    lit("wide"),
                    lit("wider"),
                    lit("widest"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Line Clamp
        // @see https://tailwindcss.com/docs/line-clamp
        add_class_group(
            &mut class_groups,
            "line-clamp",
            vec![obj(
                "line-clamp",
                vec![
                    validator(is_number),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_number),
                ],
            )],
        );

        // Line Height
        // @see https://tailwindcss.com/docs/line-height
        add_class_group(
            &mut class_groups,
            "leading",
            vec![obj("leading", {
                let mut v = vec![
                    lit("none"),
                    lit("tight"),
                    lit("snug"),
                    lit("normal"),
                    lit("relaxed"),
                    lit("loose"),
                ];
                v.extend(scale_unambiguous_spacing());
                v
            })],
        );

        // List Style Image
        // @see https://tailwindcss.com/docs/list-style-image
        add_class_group(
            &mut class_groups,
            "list-image",
            vec![obj(
                "list-image",
                vec![
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // List Style Position
        // @see https://tailwindcss.com/docs/list-style-position
        add_class_group(
            &mut class_groups,
            "list-style-position",
            vec![obj("list", vec![lit("inside"), lit("outside")])],
        );

        // List Style Type
        // @see https://tailwindcss.com/docs/list-style-type
        add_class_group(
            &mut class_groups,
            "list-style-type",
            vec![obj(
                "list",
                vec![
                    lit("disc"),
                    lit("decimal"),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Text Alignment
        // @see https://tailwindcss.com/docs/text-align
        add_class_group(
            &mut class_groups,
            "text-alignment",
            vec![obj(
                "text",
                vec![
                    lit("left"),
                    lit("center"),
                    lit("right"),
                    lit("justify"),
                    lit("start"),
                    lit("end"),
                ],
            )],
        );

        // Placeholder Color
        // @deprecated since Tailwind CSS v3.0.0
        // @see https://v3.tailwindcss.com/docs/placeholder-color
        add_class_group(
            &mut class_groups,
            "placeholder-color",
            vec![obj("placeholder", scale_color())],
        );

        // Text Color
        // @see https://tailwindcss.com/docs/text-color
        add_class_group(
            &mut class_groups,
            "text-color",
            vec![obj("text", scale_color())],
        );

        // Text Decoration
        // @see https://tailwindcss.com/docs/text-decoration
        add_class_group(
            &mut class_groups,
            "text-decoration",
            vec![
                lit("underline"),
                lit("overline"),
                lit("line-through"),
                lit("no-underline"),
            ],
        );

        // Text Decoration Style
        // @see https://tailwindcss.com/docs/text-decoration-style
        add_class_group(
            &mut class_groups,
            "text-decoration-style",
            vec![obj("decoration", {
                let mut v = scale_line_style();
                v.push(lit("wavy"));
                v
            })],
        );

        // Text Decoration Thickness
        // @see https://tailwindcss.com/docs/text-decoration-thickness
        add_class_group(
            &mut class_groups,
            "text-decoration-thickness",
            vec![obj(
                "decoration",
                vec![
                    validator(is_number),
                    lit("from-font"),
                    lit("auto"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_length),
                ],
            )],
        );

        // Text Decoration Color
        // @see https://tailwindcss.com/docs/text-decoration-color
        add_class_group(
            &mut class_groups,
            "text-decoration-color",
            vec![obj("decoration", scale_color())],
        );

        // Text Underline Offset
        // @see https://tailwindcss.com/docs/text-underline-offset
        add_class_group(
            &mut class_groups,
            "underline-offset",
            vec![obj(
                "underline-offset",
                vec![
                    validator(is_number),
                    lit("auto"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Text Transform
        // @see https://tailwindcss.com/docs/text-transform
        add_class_group(
            &mut class_groups,
            "text-transform",
            vec![
                lit("uppercase"),
                lit("lowercase"),
                lit("capitalize"),
                lit("normal-case"),
            ],
        );

        // Text Overflow
        // @see https://tailwindcss.com/docs/text-overflow
        add_class_group(
            &mut class_groups,
            "text-overflow",
            vec![lit("truncate"), lit("text-ellipsis"), lit("text-clip")],
        );

        // Text Wrap
        // @see https://tailwindcss.com/docs/text-wrap
        add_class_group(
            &mut class_groups,
            "text-wrap",
            vec![obj(
                "text",
                vec![lit("wrap"), lit("nowrap"), lit("balance"), lit("pretty")],
            )],
        );

        // Text Indent
        // @see https://tailwindcss.com/docs/text-indent
        add_class_group(
            &mut class_groups,
            "indent",
            vec![obj("indent", scale_unambiguous_spacing())],
        );

        // Vertical Alignment
        // @see https://tailwindcss.com/docs/vertical-align
        add_class_group(
            &mut class_groups,
            "vertical-align",
            vec![obj(
                "align",
                vec![
                    lit("baseline"),
                    lit("top"),
                    lit("middle"),
                    lit("bottom"),
                    lit("text-top"),
                    lit("text-bottom"),
                    lit("sub"),
                    lit("super"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Whitespace
        // @see https://tailwindcss.com/docs/whitespace
        add_class_group(
            &mut class_groups,
            "whitespace",
            vec![obj(
                "whitespace",
                vec![
                    lit("normal"),
                    lit("nowrap"),
                    lit("pre"),
                    lit("pre-line"),
                    lit("pre-wrap"),
                    lit("break-spaces"),
                ],
            )],
        );

        // Word Break
        // @see https://tailwindcss.com/docs/word-break
        add_class_group(
            &mut class_groups,
            "break",
            vec![obj(
                "break",
                vec![lit("normal"), lit("words"), lit("all"), lit("keep")],
            )],
        );

        // Overflow Wrap
        // @see https://tailwindcss.com/docs/overflow-wrap
        add_class_group(
            &mut class_groups,
            "wrap",
            vec![obj(
                "wrap",
                vec![lit("break-word"), lit("anywhere"), lit("normal")],
            )],
        );

        // Hyphens
        // @see https://tailwindcss.com/docs/hyphens
        add_class_group(
            &mut class_groups,
            "hyphens",
            vec![obj(
                "hyphens",
                vec![lit("none"), lit("manual"), lit("auto")],
            )],
        );

        // Content
        // @see https://tailwindcss.com/docs/content
        add_class_group(
            &mut class_groups,
            "content",
            vec![obj(
                "content",
                vec![
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // -------------------
        // --- Backgrounds ---
        // -------------------

        // Background Attachment
        // @see https://tailwindcss.com/docs/background-attachment
        add_class_group(
            &mut class_groups,
            "bg-attachment",
            vec![obj("bg", vec![lit("fixed"), lit("local"), lit("scroll")])],
        );

        // Background Clip
        // @see https://tailwindcss.com/docs/background-clip
        add_class_group(
            &mut class_groups,
            "bg-clip",
            vec![obj(
                "bg-clip",
                vec![lit("border"), lit("padding"), lit("content"), lit("text")],
            )],
        );

        // Background Origin
        // @see https://tailwindcss.com/docs/background-origin
        add_class_group(
            &mut class_groups,
            "bg-origin",
            vec![obj(
                "bg-origin",
                vec![lit("border"), lit("padding"), lit("content")],
            )],
        );

        // Background Position
        // @see https://tailwindcss.com/docs/background-position
        add_class_group(
            &mut class_groups,
            "bg-position",
            vec![obj("bg", scale_bg_position())],
        );

        // Background Repeat
        // @see https://tailwindcss.com/docs/background-repeat
        add_class_group(
            &mut class_groups,
            "bg-repeat",
            vec![obj("bg", scale_bg_repeat())],
        );

        // Background Size
        // @see https://tailwindcss.com/docs/background-size
        add_class_group(
            &mut class_groups,
            "bg-size",
            vec![obj("bg", scale_bg_size())],
        );

        // Background Image
        // @see https://tailwindcss.com/docs/background-image
        add_class_group(
            &mut class_groups,
            "bg-image",
            vec![obj(
                "bg",
                vec![
                    lit("none"),
                    obj(
                        "linear",
                        vec![
                            obj(
                                "to",
                                vec![
                                    lit("t"),
                                    lit("tr"),
                                    lit("r"),
                                    lit("br"),
                                    lit("b"),
                                    lit("bl"),
                                    lit("l"),
                                    lit("tl"),
                                ],
                            ),
                            validator(is_integer),
                            validator(is_arbitrary_variable),
                            validator(is_arbitrary_value),
                        ],
                    ),
                    obj(
                        "radial",
                        vec![
                            lit(""),
                            validator(is_arbitrary_variable),
                            validator(is_arbitrary_value),
                        ],
                    ),
                    obj(
                        "conic",
                        vec![
                            validator(is_integer),
                            validator(is_arbitrary_variable),
                            validator(is_arbitrary_value),
                        ],
                    ),
                    validator(is_arbitrary_variable_image),
                    validator(is_arbitrary_image),
                ],
            )],
        );

        // Background Color
        // @see https://tailwindcss.com/docs/background-color
        add_class_group(
            &mut class_groups,
            "bg-color",
            vec![obj("bg", scale_color())],
        );

        // Gradient Color Stops From Position
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-from-pos",
            vec![obj("from", scale_gradient_stop_position())],
        );

        // Gradient Color Stops Via Position
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-via-pos",
            vec![obj("via", scale_gradient_stop_position())],
        );

        // Gradient Color Stops To Position
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-to-pos",
            vec![obj("to", scale_gradient_stop_position())],
        );

        // Gradient Color Stops From
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-from",
            vec![obj("from", scale_color())],
        );

        // Gradient Color Stops Via
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-via",
            vec![obj("via", scale_color())],
        );

        // Gradient Color Stops To
        // @see https://tailwindcss.com/docs/gradient-color-stops
        add_class_group(
            &mut class_groups,
            "gradient-to",
            vec![obj("to", scale_color())],
        );

        // ---------------
        // --- Borders ---
        // ---------------

        // Border Radius
        // @see https://tailwindcss.com/docs/border-radius
        add_class_group(
            &mut class_groups,
            "rounded",
            vec![obj("rounded", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-s",
            vec![obj("rounded-s", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-e",
            vec![obj("rounded-e", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-t",
            vec![obj("rounded-t", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-r",
            vec![obj("rounded-r", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-b",
            vec![obj("rounded-b", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-l",
            vec![obj("rounded-l", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-ss",
            vec![obj("rounded-ss", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-se",
            vec![obj("rounded-se", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-ee",
            vec![obj("rounded-ee", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-es",
            vec![obj("rounded-es", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-tl",
            vec![obj("rounded-tl", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-tr",
            vec![obj("rounded-tr", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-br",
            vec![obj("rounded-br", scale_radius())],
        );
        add_class_group(
            &mut class_groups,
            "rounded-bl",
            vec![obj("rounded-bl", scale_radius())],
        );

        // Border Width
        // @see https://tailwindcss.com/docs/border-width
        add_class_group(
            &mut class_groups,
            "border-w",
            vec![obj("border", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-x",
            vec![obj("border-x", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-y",
            vec![obj("border-y", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-s",
            vec![obj("border-s", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-e",
            vec![obj("border-e", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-t",
            vec![obj("border-t", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-r",
            vec![obj("border-r", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-b",
            vec![obj("border-b", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "border-w-l",
            vec![obj("border-l", scale_border_width())],
        );

        // Divide Width X
        // @see https://tailwindcss.com/docs/border-width#between-children
        add_class_group(
            &mut class_groups,
            "divide-x",
            vec![obj("divide-x", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "divide-x-reverse",
            vec![lit("divide-x-reverse")],
        );

        // Divide Width Y
        // @see https://tailwindcss.com/docs/border-width#between-children
        add_class_group(
            &mut class_groups,
            "divide-y",
            vec![obj("divide-y", scale_border_width())],
        );
        add_class_group(
            &mut class_groups,
            "divide-y-reverse",
            vec![lit("divide-y-reverse")],
        );

        // Border Style
        // @see https://tailwindcss.com/docs/border-style
        add_class_group(
            &mut class_groups,
            "border-style",
            vec![obj("border", {
                let mut v = scale_line_style();
                v.push(lit("hidden"));
                v.push(lit("none"));
                v
            })],
        );

        // Divide Style
        // @see https://tailwindcss.com/docs/border-style#setting-the-divider-style
        add_class_group(
            &mut class_groups,
            "divide-style",
            vec![obj("divide", {
                let mut v = scale_line_style();
                v.push(lit("hidden"));
                v.push(lit("none"));
                v
            })],
        );

        // Border Color
        // @see https://tailwindcss.com/docs/border-color
        add_class_group(
            &mut class_groups,
            "border-color",
            vec![obj("border", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-x",
            vec![obj("border-x", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-y",
            vec![obj("border-y", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-s",
            vec![obj("border-s", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-e",
            vec![obj("border-e", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-t",
            vec![obj("border-t", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-r",
            vec![obj("border-r", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-b",
            vec![obj("border-b", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "border-color-l",
            vec![obj("border-l", scale_color())],
        );

        // Divide Color
        // @see https://tailwindcss.com/docs/divide-color
        add_class_group(
            &mut class_groups,
            "divide-color",
            vec![obj("divide", scale_color())],
        );

        // Outline Style
        // @see https://tailwindcss.com/docs/outline-style
        add_class_group(
            &mut class_groups,
            "outline-style",
            vec![obj("outline", {
                let mut v = scale_line_style();
                v.push(lit("none"));
                v.push(lit("hidden"));
                v
            })],
        );

        // Outline Offset
        // @see https://tailwindcss.com/docs/outline-offset
        add_class_group(
            &mut class_groups,
            "outline-offset",
            vec![obj(
                "outline-offset",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Outline Width
        // @see https://tailwindcss.com/docs/outline-width
        add_class_group(
            &mut class_groups,
            "outline-w",
            vec![obj(
                "outline",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable_length),
                    validator(is_arbitrary_length),
                ],
            )],
        );

        // Outline Color
        // @see https://tailwindcss.com/docs/outline-color
        add_class_group(
            &mut class_groups,
            "outline-color",
            vec![obj("outline", scale_color())],
        );

        // ---------------
        // --- Effects ---
        // ---------------

        // Box Shadow
        // @see https://tailwindcss.com/docs/box-shadow
        add_class_group(
            &mut class_groups,
            "shadow",
            vec![obj(
                "shadow",
                vec![
                    lit(""),
                    lit("none"),
                    validator(is_tshirt_size),
                    validator(is_arbitrary_variable_shadow),
                    validator(is_arbitrary_shadow),
                ],
            )],
        );

        // Box Shadow Color
        // @see https://tailwindcss.com/docs/box-shadow#setting-the-shadow-color
        add_class_group(
            &mut class_groups,
            "shadow-color",
            vec![obj("shadow", scale_color())],
        );

        // Inset Box Shadow
        // @see https://tailwindcss.com/docs/box-shadow#adding-an-inset-shadow
        add_class_group(
            &mut class_groups,
            "inset-shadow",
            vec![obj(
                "inset-shadow",
                vec![
                    lit("none"),
                    validator(is_tshirt_size),
                    validator(is_arbitrary_variable_shadow),
                    validator(is_arbitrary_shadow),
                ],
            )],
        );

        // Inset Box Shadow Color
        // @see https://tailwindcss.com/docs/box-shadow#setting-the-inset-shadow-color
        add_class_group(
            &mut class_groups,
            "inset-shadow-color",
            vec![obj("inset-shadow", scale_color())],
        );

        // Ring Width
        // @see https://tailwindcss.com/docs/box-shadow#adding-a-ring
        add_class_group(
            &mut class_groups,
            "ring-w",
            vec![obj("ring", scale_border_width())],
        );

        // Ring Width Inset
        // @see https://v3.tailwindcss.com/docs/ring-width#inset-rings
        // @deprecated since Tailwind CSS v4.0.0
        add_class_group(&mut class_groups, "ring-w-inset", vec![lit("ring-inset")]);

        // Ring Color
        // @see https://tailwindcss.com/docs/box-shadow#setting-the-ring-color
        add_class_group(
            &mut class_groups,
            "ring-color",
            vec![obj("ring", scale_color())],
        );

        // Ring Offset Width
        // @see https://v3.tailwindcss.com/docs/ring-offset-width
        // @deprecated since Tailwind CSS v4.0.0
        add_class_group(
            &mut class_groups,
            "ring-offset-w",
            vec![obj(
                "ring-offset",
                vec![validator(is_number), validator(is_arbitrary_length)],
            )],
        );

        // Ring Offset Color
        // @see https://v3.tailwindcss.com/docs/ring-offset-color
        // @deprecated since Tailwind CSS v4.0.0
        add_class_group(
            &mut class_groups,
            "ring-offset-color",
            vec![obj("ring-offset", scale_color())],
        );

        // Inset Ring Width
        // @see https://tailwindcss.com/docs/box-shadow#adding-an-inset-ring
        add_class_group(
            &mut class_groups,
            "inset-ring-w",
            vec![obj("inset-ring", scale_border_width())],
        );

        // Inset Ring Color
        // @see https://tailwindcss.com/docs/box-shadow#setting-the-inset-ring-color
        add_class_group(
            &mut class_groups,
            "inset-ring-color",
            vec![obj("inset-ring", scale_color())],
        );

        // Text Shadow
        // @see https://tailwindcss.com/docs/text-shadow
        add_class_group(
            &mut class_groups,
            "text-shadow",
            vec![obj(
                "text-shadow",
                vec![
                    lit("none"),
                    validator(is_tshirt_size),
                    validator(is_arbitrary_variable_shadow),
                    validator(is_arbitrary_shadow),
                ],
            )],
        );

        // Text Shadow Color
        // @see https://tailwindcss.com/docs/text-shadow#setting-the-shadow-color
        add_class_group(
            &mut class_groups,
            "text-shadow-color",
            vec![obj("text-shadow", scale_color())],
        );

        // Opacity
        // @see https://tailwindcss.com/docs/opacity
        add_class_group(
            &mut class_groups,
            "opacity",
            vec![obj(
                "opacity",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Mix Blend Mode
        // @see https://tailwindcss.com/docs/mix-blend-mode
        add_class_group(
            &mut class_groups,
            "mix-blend",
            vec![obj("mix-blend", {
                let mut v = scale_blend_mode();
                v.push(lit("plus-darker"));
                v.push(lit("plus-lighter"));
                v
            })],
        );

        // Background Blend Mode
        // @see https://tailwindcss.com/docs/background-blend-mode
        add_class_group(
            &mut class_groups,
            "bg-blend",
            vec![obj("bg-blend", scale_blend_mode())],
        );

        // Mask Clip
        // @see https://tailwindcss.com/docs/mask-clip
        add_class_group(
            &mut class_groups,
            "mask-clip",
            vec![
                obj(
                    "mask-clip",
                    vec![
                        lit("border"),
                        lit("padding"),
                        lit("content"),
                        lit("fill"),
                        lit("stroke"),
                        lit("view"),
                    ],
                ),
                lit("mask-no-clip"),
            ],
        );

        // Mask Composite
        // @see https://tailwindcss.com/docs/mask-composite
        add_class_group(
            &mut class_groups,
            "mask-composite",
            vec![obj(
                "mask",
                vec![
                    lit("add"),
                    lit("subtract"),
                    lit("intersect"),
                    lit("exclude"),
                ],
            )],
        );

        // Mask Image (linear gradients)
        // @see https://tailwindcss.com/docs/mask-image
        add_class_group(
            &mut class_groups,
            "mask-image-linear-pos",
            vec![obj("mask-linear", vec![validator(is_number)])],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-linear-from-pos",
            vec![obj("mask-linear-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-linear-to-pos",
            vec![obj("mask-linear-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-linear-from-color",
            vec![obj("mask-linear-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-linear-to-color",
            vec![obj("mask-linear-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-t-from-pos",
            vec![obj("mask-t-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-t-to-pos",
            vec![obj("mask-t-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-t-from-color",
            vec![obj("mask-t-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-t-to-color",
            vec![obj("mask-t-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-r-from-pos",
            vec![obj("mask-r-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-r-to-pos",
            vec![obj("mask-r-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-r-from-color",
            vec![obj("mask-r-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-r-to-color",
            vec![obj("mask-r-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-b-from-pos",
            vec![obj("mask-b-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-b-to-pos",
            vec![obj("mask-b-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-b-from-color",
            vec![obj("mask-b-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-b-to-color",
            vec![obj("mask-b-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-l-from-pos",
            vec![obj("mask-l-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-l-to-pos",
            vec![obj("mask-l-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-l-from-color",
            vec![obj("mask-l-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-l-to-color",
            vec![obj("mask-l-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-x-from-pos",
            vec![obj("mask-x-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-x-to-pos",
            vec![obj("mask-x-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-x-from-color",
            vec![obj("mask-x-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-x-to-color",
            vec![obj("mask-x-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-y-from-pos",
            vec![obj("mask-y-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-y-to-pos",
            vec![obj("mask-y-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-y-from-color",
            vec![obj("mask-y-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-y-to-color",
            vec![obj("mask-y-to", scale_color())],
        );

        // Mask Image (radial gradients)
        add_class_group(
            &mut class_groups,
            "mask-image-radial",
            vec![obj(
                "mask-radial",
                vec![
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-from-pos",
            vec![obj("mask-radial-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-to-pos",
            vec![obj("mask-radial-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-from-color",
            vec![obj("mask-radial-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-to-color",
            vec![obj("mask-radial-to", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-shape",
            vec![obj("mask-radial", vec![lit("circle"), lit("ellipse")])],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-size",
            vec![obj(
                "mask-radial",
                vec![
                    obj("closest", vec![lit("side"), lit("corner")]),
                    obj("farthest", vec![lit("side"), lit("corner")]),
                ],
            )],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-radial-pos",
            vec![obj("mask-radial-at", scale_position())],
        );

        // Mask Image (conic gradients)
        add_class_group(
            &mut class_groups,
            "mask-image-conic-pos",
            vec![obj("mask-conic", vec![validator(is_number)])],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-conic-from-pos",
            vec![obj("mask-conic-from", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-conic-to-pos",
            vec![obj("mask-conic-to", scale_mask_image_position())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-conic-from-color",
            vec![obj("mask-conic-from", scale_color())],
        );
        add_class_group(
            &mut class_groups,
            "mask-image-conic-to-color",
            vec![obj("mask-conic-to", scale_color())],
        );

        // Mask Mode
        // @see https://tailwindcss.com/docs/mask-mode
        add_class_group(
            &mut class_groups,
            "mask-mode",
            vec![obj(
                "mask",
                vec![lit("alpha"), lit("luminance"), lit("match")],
            )],
        );

        // Mask Origin
        // @see https://tailwindcss.com/docs/mask-origin
        add_class_group(
            &mut class_groups,
            "mask-origin",
            vec![obj(
                "mask-origin",
                vec![
                    lit("border"),
                    lit("padding"),
                    lit("content"),
                    lit("fill"),
                    lit("stroke"),
                    lit("view"),
                ],
            )],
        );

        // Mask Position
        // @see https://tailwindcss.com/docs/mask-position
        add_class_group(
            &mut class_groups,
            "mask-position",
            vec![obj("mask", scale_bg_position())],
        );

        // Mask Repeat
        // @see https://tailwindcss.com/docs/mask-repeat
        add_class_group(
            &mut class_groups,
            "mask-repeat",
            vec![obj("mask", scale_bg_repeat())],
        );

        // Mask Size
        // @see https://tailwindcss.com/docs/mask-size
        add_class_group(
            &mut class_groups,
            "mask-size",
            vec![obj("mask", scale_bg_size())],
        );

        // Mask Type
        // @see https://tailwindcss.com/docs/mask-type
        add_class_group(
            &mut class_groups,
            "mask-type",
            vec![obj("mask-type", vec![lit("alpha"), lit("luminance")])],
        );

        // Mask Image
        // @see https://tailwindcss.com/docs/mask-image
        add_class_group(
            &mut class_groups,
            "mask-image",
            vec![obj(
                "mask",
                vec![
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // ---------------
        // --- Filters ---
        // ---------------

        // Filter
        // @see https://tailwindcss.com/docs/filter
        add_class_group(
            &mut class_groups,
            "filter",
            vec![obj(
                "filter",
                vec![
                    lit(""),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Blur
        // @see https://tailwindcss.com/docs/blur
        add_class_group(&mut class_groups, "blur", vec![obj("blur", scale_blur())]);

        // Brightness
        // @see https://tailwindcss.com/docs/brightness
        add_class_group(
            &mut class_groups,
            "brightness",
            vec![obj(
                "brightness",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Contrast
        // @see https://tailwindcss.com/docs/contrast
        add_class_group(
            &mut class_groups,
            "contrast",
            vec![obj(
                "contrast",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Drop Shadow
        // @see https://tailwindcss.com/docs/drop-shadow
        add_class_group(
            &mut class_groups,
            "drop-shadow",
            vec![obj(
                "drop-shadow",
                vec![
                    lit(""),
                    lit("none"),
                    validator(is_tshirt_size),
                    validator(is_arbitrary_variable_shadow),
                    validator(is_arbitrary_shadow),
                ],
            )],
        );

        // Drop Shadow Color
        // @see https://tailwindcss.com/docs/filter-drop-shadow#setting-the-shadow-color
        add_class_group(
            &mut class_groups,
            "drop-shadow-color",
            vec![obj("drop-shadow", scale_color())],
        );

        // Grayscale
        // @see https://tailwindcss.com/docs/grayscale
        add_class_group(
            &mut class_groups,
            "grayscale",
            vec![obj(
                "grayscale",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Hue Rotate
        // @see https://tailwindcss.com/docs/hue-rotate
        add_class_group(
            &mut class_groups,
            "hue-rotate",
            vec![obj(
                "hue-rotate",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Invert
        // @see https://tailwindcss.com/docs/invert
        add_class_group(
            &mut class_groups,
            "invert",
            vec![obj(
                "invert",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Saturate
        // @see https://tailwindcss.com/docs/saturate
        add_class_group(
            &mut class_groups,
            "saturate",
            vec![obj(
                "saturate",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Sepia
        // @see https://tailwindcss.com/docs/sepia
        add_class_group(
            &mut class_groups,
            "sepia",
            vec![obj(
                "sepia",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Filter
        // @see https://tailwindcss.com/docs/backdrop-filter
        add_class_group(
            &mut class_groups,
            "backdrop-filter",
            vec![obj(
                "backdrop-filter",
                vec![
                    lit(""),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Blur
        // @see https://tailwindcss.com/docs/backdrop-blur
        add_class_group(
            &mut class_groups,
            "backdrop-blur",
            vec![obj("backdrop-blur", scale_blur())],
        );

        // Backdrop Brightness
        // @see https://tailwindcss.com/docs/backdrop-brightness
        add_class_group(
            &mut class_groups,
            "backdrop-brightness",
            vec![obj(
                "backdrop-brightness",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Contrast
        // @see https://tailwindcss.com/docs/backdrop-contrast
        add_class_group(
            &mut class_groups,
            "backdrop-contrast",
            vec![obj(
                "backdrop-contrast",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Grayscale
        // @see https://tailwindcss.com/docs/backdrop-grayscale
        add_class_group(
            &mut class_groups,
            "backdrop-grayscale",
            vec![obj(
                "backdrop-grayscale",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Hue Rotate
        // @see https://tailwindcss.com/docs/backdrop-hue-rotate
        add_class_group(
            &mut class_groups,
            "backdrop-hue-rotate",
            vec![obj(
                "backdrop-hue-rotate",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Invert
        // @see https://tailwindcss.com/docs/backdrop-invert
        add_class_group(
            &mut class_groups,
            "backdrop-invert",
            vec![obj(
                "backdrop-invert",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Opacity
        // @see https://tailwindcss.com/docs/backdrop-opacity
        add_class_group(
            &mut class_groups,
            "backdrop-opacity",
            vec![obj(
                "backdrop-opacity",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Saturate
        // @see https://tailwindcss.com/docs/backdrop-saturate
        add_class_group(
            &mut class_groups,
            "backdrop-saturate",
            vec![obj(
                "backdrop-saturate",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Backdrop Sepia
        // @see https://tailwindcss.com/docs/backdrop-sepia
        add_class_group(
            &mut class_groups,
            "backdrop-sepia",
            vec![obj(
                "backdrop-sepia",
                vec![
                    lit(""),
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // --------------
        // --- Tables ---
        // --------------

        // Border Collapse
        // @see https://tailwindcss.com/docs/border-collapse
        add_class_group(
            &mut class_groups,
            "border-collapse",
            vec![obj("border", vec![lit("collapse"), lit("separate")])],
        );

        // Border Spacing
        // @see https://tailwindcss.com/docs/border-spacing
        add_class_group(
            &mut class_groups,
            "border-spacing",
            vec![obj("border-spacing", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "border-spacing-x",
            vec![obj("border-spacing-x", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "border-spacing-y",
            vec![obj("border-spacing-y", scale_unambiguous_spacing())],
        );

        // Table Layout
        // @see https://tailwindcss.com/docs/table-layout
        add_class_group(
            &mut class_groups,
            "table-layout",
            vec![obj("table", vec![lit("auto"), lit("fixed")])],
        );

        // Caption Side
        // @see https://tailwindcss.com/docs/caption-side
        add_class_group(
            &mut class_groups,
            "caption",
            vec![obj("caption", vec![lit("top"), lit("bottom")])],
        );

        // ---------------------------------
        // --- Transitions and Animation ---
        // ---------------------------------

        // Transition Property
        // @see https://tailwindcss.com/docs/transition-property
        add_class_group(
            &mut class_groups,
            "transition",
            vec![obj(
                "transition",
                vec![
                    lit(""),
                    lit("all"),
                    lit("colors"),
                    lit("opacity"),
                    lit("shadow"),
                    lit("transform"),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Transition Behavior
        // @see https://tailwindcss.com/docs/transition-behavior
        add_class_group(
            &mut class_groups,
            "transition-behavior",
            vec![obj("transition", vec![lit("normal"), lit("discrete")])],
        );

        // Transition Duration
        // @see https://tailwindcss.com/docs/transition-duration
        add_class_group(
            &mut class_groups,
            "duration",
            vec![obj(
                "duration",
                vec![
                    validator(is_number),
                    lit("initial"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Transition Timing Function
        // @see https://tailwindcss.com/docs/transition-timing-function
        add_class_group(
            &mut class_groups,
            "ease",
            vec![obj(
                "ease",
                vec![
                    lit("linear"),
                    lit("initial"),
                    lit("in"),
                    lit("out"),
                    lit("in-out"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Transition Delay
        // @see https://tailwindcss.com/docs/transition-delay
        add_class_group(
            &mut class_groups,
            "delay",
            vec![obj(
                "delay",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Animation
        // @see https://tailwindcss.com/docs/animation
        add_class_group(
            &mut class_groups,
            "animate",
            vec![obj(
                "animate",
                vec![
                    lit("none"),
                    lit("spin"),
                    lit("ping"),
                    lit("pulse"),
                    lit("bounce"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // ------------------
        // --- Transforms ---
        // ------------------

        // Backface Visibility
        // @see https://tailwindcss.com/docs/backface-visibility
        add_class_group(
            &mut class_groups,
            "backface",
            vec![obj("backface", vec![lit("hidden"), lit("visible")])],
        );

        // Perspective
        // @see https://tailwindcss.com/docs/perspective
        add_class_group(
            &mut class_groups,
            "perspective",
            vec![obj(
                "perspective",
                vec![
                    lit("dramatic"),
                    lit("near"),
                    lit("normal"),
                    lit("midrange"),
                    lit("distant"),
                    lit("none"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Perspective Origin
        // @see https://tailwindcss.com/docs/perspective-origin
        add_class_group(
            &mut class_groups,
            "perspective-origin",
            vec![obj("perspective-origin", scale_position_with_arbitrary())],
        );

        // Rotate
        // @see https://tailwindcss.com/docs/rotate
        add_class_group(
            &mut class_groups,
            "rotate",
            vec![obj("rotate", scale_rotate())],
        );
        add_class_group(
            &mut class_groups,
            "rotate-x",
            vec![obj("rotate-x", scale_rotate())],
        );
        add_class_group(
            &mut class_groups,
            "rotate-y",
            vec![obj("rotate-y", scale_rotate())],
        );
        add_class_group(
            &mut class_groups,
            "rotate-z",
            vec![obj("rotate-z", scale_rotate())],
        );

        // Scale
        // @see https://tailwindcss.com/docs/scale
        add_class_group(
            &mut class_groups,
            "scale",
            vec![obj("scale", scale_scale())],
        );
        add_class_group(
            &mut class_groups,
            "scale-x",
            vec![obj("scale-x", scale_scale())],
        );
        add_class_group(
            &mut class_groups,
            "scale-y",
            vec![obj("scale-y", scale_scale())],
        );
        add_class_group(
            &mut class_groups,
            "scale-z",
            vec![obj("scale-z", scale_scale())],
        );

        // Scale 3D
        // @see https://tailwindcss.com/docs/scale
        add_class_group(&mut class_groups, "scale-3d", vec![lit("scale-3d")]);

        // Skew
        // @see https://tailwindcss.com/docs/skew
        add_class_group(&mut class_groups, "skew", vec![obj("skew", scale_skew())]);
        add_class_group(
            &mut class_groups,
            "skew-x",
            vec![obj("skew-x", scale_skew())],
        );
        add_class_group(
            &mut class_groups,
            "skew-y",
            vec![obj("skew-y", scale_skew())],
        );

        // Transform
        // @see https://tailwindcss.com/docs/transform
        add_class_group(
            &mut class_groups,
            "transform",
            vec![obj(
                "transform",
                vec![
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                    lit(""),
                    lit("none"),
                    lit("gpu"),
                    lit("cpu"),
                ],
            )],
        );

        // Transform Origin
        // @see https://tailwindcss.com/docs/transform-origin
        add_class_group(
            &mut class_groups,
            "transform-origin",
            vec![obj("origin", scale_position_with_arbitrary())],
        );

        // Transform Style
        // @see https://tailwindcss.com/docs/transform-style
        add_class_group(
            &mut class_groups,
            "transform-style",
            vec![obj("transform", vec![lit("3d"), lit("flat")])],
        );

        // Translate
        // @see https://tailwindcss.com/docs/translate
        add_class_group(
            &mut class_groups,
            "translate",
            vec![obj("translate", scale_translate())],
        );
        add_class_group(
            &mut class_groups,
            "translate-x",
            vec![obj("translate-x", scale_translate())],
        );
        add_class_group(
            &mut class_groups,
            "translate-y",
            vec![obj("translate-y", scale_translate())],
        );
        add_class_group(
            &mut class_groups,
            "translate-z",
            vec![obj("translate-z", scale_translate())],
        );

        // Translate None
        // @see https://tailwindcss.com/docs/translate
        add_class_group(
            &mut class_groups,
            "translate-none",
            vec![lit("translate-none")],
        );

        // ---------------------
        // --- Interactivity ---
        // ---------------------

        // Accent Color
        // @see https://tailwindcss.com/docs/accent-color
        add_class_group(
            &mut class_groups,
            "accent",
            vec![obj("accent", scale_color())],
        );

        // Appearance
        // @see https://tailwindcss.com/docs/appearance
        add_class_group(
            &mut class_groups,
            "appearance",
            vec![obj("appearance", vec![lit("none"), lit("auto")])],
        );

        // Caret Color
        // @see https://tailwindcss.com/docs/just-in-time-mode#caret-color-utilities
        add_class_group(
            &mut class_groups,
            "caret-color",
            vec![obj("caret", scale_color())],
        );

        // Color Scheme
        // @see https://tailwindcss.com/docs/color-scheme
        add_class_group(
            &mut class_groups,
            "color-scheme",
            vec![obj(
                "scheme",
                vec![
                    lit("normal"),
                    lit("dark"),
                    lit("light"),
                    lit("light-dark"),
                    lit("only-dark"),
                    lit("only-light"),
                ],
            )],
        );

        // Cursor
        // @see https://tailwindcss.com/docs/cursor
        add_class_group(
            &mut class_groups,
            "cursor",
            vec![obj(
                "cursor",
                vec![
                    lit("auto"),
                    lit("default"),
                    lit("pointer"),
                    lit("wait"),
                    lit("text"),
                    lit("move"),
                    lit("help"),
                    lit("not-allowed"),
                    lit("none"),
                    lit("context-menu"),
                    lit("progress"),
                    lit("cell"),
                    lit("crosshair"),
                    lit("vertical-text"),
                    lit("alias"),
                    lit("copy"),
                    lit("no-drop"),
                    lit("grab"),
                    lit("grabbing"),
                    lit("all-scroll"),
                    lit("col-resize"),
                    lit("row-resize"),
                    lit("n-resize"),
                    lit("e-resize"),
                    lit("s-resize"),
                    lit("w-resize"),
                    lit("ne-resize"),
                    lit("nw-resize"),
                    lit("se-resize"),
                    lit("sw-resize"),
                    lit("ew-resize"),
                    lit("ns-resize"),
                    lit("nesw-resize"),
                    lit("nwse-resize"),
                    lit("zoom-in"),
                    lit("zoom-out"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // Field Sizing
        // @see https://tailwindcss.com/docs/field-sizing
        add_class_group(
            &mut class_groups,
            "field-sizing",
            vec![obj("field-sizing", vec![lit("fixed"), lit("content")])],
        );

        // Pointer Events
        // @see https://tailwindcss.com/docs/pointer-events
        add_class_group(
            &mut class_groups,
            "pointer-events",
            vec![obj("pointer-events", vec![lit("auto"), lit("none")])],
        );

        // Resize
        // @see https://tailwindcss.com/docs/resize
        add_class_group(
            &mut class_groups,
            "resize",
            vec![obj(
                "resize",
                vec![lit("none"), lit(""), lit("y"), lit("x")],
            )],
        );

        // Scroll Behavior
        // @see https://tailwindcss.com/docs/scroll-behavior
        add_class_group(
            &mut class_groups,
            "scroll-behavior",
            vec![obj("scroll", vec![lit("auto"), lit("smooth")])],
        );

        // Scroll Margin
        // @see https://tailwindcss.com/docs/scroll-margin
        add_class_group(
            &mut class_groups,
            "scroll-m",
            vec![obj("scroll-m", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-mx",
            vec![obj("scroll-mx", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-my",
            vec![obj("scroll-my", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-ms",
            vec![obj("scroll-ms", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-me",
            vec![obj("scroll-me", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-mt",
            vec![obj("scroll-mt", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-mr",
            vec![obj("scroll-mr", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-mb",
            vec![obj("scroll-mb", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-ml",
            vec![obj("scroll-ml", scale_unambiguous_spacing())],
        );

        // Scroll Padding
        // @see https://tailwindcss.com/docs/scroll-padding
        add_class_group(
            &mut class_groups,
            "scroll-p",
            vec![obj("scroll-p", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-px",
            vec![obj("scroll-px", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-py",
            vec![obj("scroll-py", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-ps",
            vec![obj("scroll-ps", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-pe",
            vec![obj("scroll-pe", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-pt",
            vec![obj("scroll-pt", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-pr",
            vec![obj("scroll-pr", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-pb",
            vec![obj("scroll-pb", scale_unambiguous_spacing())],
        );
        add_class_group(
            &mut class_groups,
            "scroll-pl",
            vec![obj("scroll-pl", scale_unambiguous_spacing())],
        );

        // Scroll Snap Align
        // @see https://tailwindcss.com/docs/scroll-snap-align
        add_class_group(
            &mut class_groups,
            "snap-align",
            vec![obj(
                "snap",
                vec![lit("start"), lit("end"), lit("center"), lit("align-none")],
            )],
        );

        // Scroll Snap Stop
        // @see https://tailwindcss.com/docs/scroll-snap-stop
        add_class_group(
            &mut class_groups,
            "snap-stop",
            vec![obj("snap", vec![lit("normal"), lit("always")])],
        );

        // Scroll Snap Type
        // @see https://tailwindcss.com/docs/scroll-snap-type
        add_class_group(
            &mut class_groups,
            "snap-type",
            vec![obj(
                "snap",
                vec![lit("none"), lit("x"), lit("y"), lit("both")],
            )],
        );

        // Scroll Snap Type Strictness
        // @see https://tailwindcss.com/docs/scroll-snap-type
        add_class_group(
            &mut class_groups,
            "snap-strictness",
            vec![obj("snap", vec![lit("mandatory"), lit("proximity")])],
        );

        // Touch Action
        // @see https://tailwindcss.com/docs/touch-action
        add_class_group(
            &mut class_groups,
            "touch",
            vec![obj(
                "touch",
                vec![lit("auto"), lit("none"), lit("manipulation")],
            )],
        );
        add_class_group(
            &mut class_groups,
            "touch-x",
            vec![obj("touch-pan", vec![lit("x"), lit("left"), lit("right")])],
        );
        add_class_group(
            &mut class_groups,
            "touch-y",
            vec![obj("touch-pan", vec![lit("y"), lit("up"), lit("down")])],
        );
        add_class_group(&mut class_groups, "touch-pz", vec![lit("touch-pinch-zoom")]);

        // User Select
        // @see https://tailwindcss.com/docs/user-select
        add_class_group(
            &mut class_groups,
            "select",
            vec![obj(
                "select",
                vec![lit("none"), lit("text"), lit("all"), lit("auto")],
            )],
        );

        // Will Change
        // @see https://tailwindcss.com/docs/will-change
        add_class_group(
            &mut class_groups,
            "will-change",
            vec![obj(
                "will-change",
                vec![
                    lit("auto"),
                    lit("scroll"),
                    lit("contents"),
                    lit("transform"),
                    validator(is_arbitrary_variable),
                    validator(is_arbitrary_value),
                ],
            )],
        );

        // -----------
        // --- SVG ---
        // -----------

        // Fill
        // @see https://tailwindcss.com/docs/fill
        add_class_group(
            &mut class_groups,
            "fill",
            vec![obj("fill", {
                let mut v = vec![lit("none")];
                v.extend(scale_color());
                v
            })],
        );

        // Stroke Width
        // @see https://tailwindcss.com/docs/stroke-width
        add_class_group(
            &mut class_groups,
            "stroke-w",
            vec![obj(
                "stroke",
                vec![
                    validator(is_number),
                    validator(is_arbitrary_variable_length),
                    validator(is_arbitrary_length),
                    validator(is_arbitrary_number),
                ],
            )],
        );

        // Stroke
        // @see https://tailwindcss.com/docs/stroke
        add_class_group(
            &mut class_groups,
            "stroke",
            vec![obj("stroke", {
                let mut v = vec![lit("none")];
                v.extend(scale_color());
                v
            })],
        );

        // ---------------------
        // --- Accessibility ---
        // ---------------------

        // Forced Color Adjust
        // @see https://tailwindcss.com/docs/forced-color-adjust
        add_class_group(
            &mut class_groups,
            "forced-color-adjust",
            vec![obj("forced-color-adjust", vec![lit("auto"), lit("none")])],
        );

        // -------------------------
        // --- Conflicting Groups ---
        // -------------------------

        add_conflict(
            &mut conflicting_class_groups,
            "overflow",
            vec!["overflow-x", "overflow-y"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "overscroll",
            vec!["overscroll-x", "overscroll-y"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "inset",
            vec![
                "inset-x", "inset-y", "start", "end", "top", "right", "bottom", "left",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "inset-x",
            vec!["right", "left"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "inset-y",
            vec!["top", "bottom"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "flex",
            vec!["basis", "grow", "shrink"],
        );
        add_conflict(&mut conflicting_class_groups, "gap", vec!["gap-x", "gap-y"]);
        add_conflict(
            &mut conflicting_class_groups,
            "p",
            vec!["px", "py", "ps", "pe", "pt", "pr", "pb", "pl"],
        );
        add_conflict(&mut conflicting_class_groups, "px", vec!["pr", "pl"]);
        add_conflict(&mut conflicting_class_groups, "py", vec!["pt", "pb"]);
        add_conflict(
            &mut conflicting_class_groups,
            "m",
            vec!["mx", "my", "ms", "me", "mt", "mr", "mb", "ml"],
        );
        add_conflict(&mut conflicting_class_groups, "mx", vec!["mr", "ml"]);
        add_conflict(&mut conflicting_class_groups, "my", vec!["mt", "mb"]);
        add_conflict(&mut conflicting_class_groups, "size", vec!["w", "h"]);
        add_conflict(&mut conflicting_class_groups, "font-size", vec!["leading"]);
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-normal",
            vec![
                "fvn-ordinal",
                "fvn-slashed-zero",
                "fvn-figure",
                "fvn-spacing",
                "fvn-fraction",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-ordinal",
            vec!["fvn-normal"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-slashed-zero",
            vec!["fvn-normal"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-figure",
            vec!["fvn-normal"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-spacing",
            vec!["fvn-normal"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "fvn-fraction",
            vec!["fvn-normal"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "line-clamp",
            vec!["display", "overflow"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded",
            vec![
                "rounded-s",
                "rounded-e",
                "rounded-t",
                "rounded-r",
                "rounded-b",
                "rounded-l",
                "rounded-ss",
                "rounded-se",
                "rounded-ee",
                "rounded-es",
                "rounded-tl",
                "rounded-tr",
                "rounded-br",
                "rounded-bl",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-s",
            vec!["rounded-ss", "rounded-es"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-e",
            vec!["rounded-se", "rounded-ee"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-t",
            vec!["rounded-tl", "rounded-tr"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-r",
            vec!["rounded-tr", "rounded-br"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-b",
            vec!["rounded-br", "rounded-bl"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "rounded-l",
            vec!["rounded-tl", "rounded-bl"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-spacing",
            vec!["border-spacing-x", "border-spacing-y"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-w",
            vec![
                "border-w-x",
                "border-w-y",
                "border-w-s",
                "border-w-e",
                "border-w-t",
                "border-w-r",
                "border-w-b",
                "border-w-l",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-w-x",
            vec!["border-w-r", "border-w-l"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-w-y",
            vec!["border-w-t", "border-w-b"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-color",
            vec![
                "border-color-x",
                "border-color-y",
                "border-color-s",
                "border-color-e",
                "border-color-t",
                "border-color-r",
                "border-color-b",
                "border-color-l",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-color-x",
            vec!["border-color-r", "border-color-l"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "border-color-y",
            vec!["border-color-t", "border-color-b"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "translate",
            vec!["translate-x", "translate-y", "translate-none"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "translate-none",
            vec!["translate", "translate-x", "translate-y", "translate-z"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-m",
            vec![
                "scroll-mx",
                "scroll-my",
                "scroll-ms",
                "scroll-me",
                "scroll-mt",
                "scroll-mr",
                "scroll-mb",
                "scroll-ml",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-mx",
            vec!["scroll-mr", "scroll-ml"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-my",
            vec!["scroll-mt", "scroll-mb"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-p",
            vec![
                "scroll-px",
                "scroll-py",
                "scroll-ps",
                "scroll-pe",
                "scroll-pt",
                "scroll-pr",
                "scroll-pb",
                "scroll-pl",
            ],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-px",
            vec!["scroll-pr", "scroll-pl"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "scroll-py",
            vec!["scroll-pt", "scroll-pb"],
        );
        add_conflict(
            &mut conflicting_class_groups,
            "touch",
            vec!["touch-x", "touch-y", "touch-pz"],
        );
        add_conflict(&mut conflicting_class_groups, "touch-x", vec!["touch"]);
        add_conflict(&mut conflicting_class_groups, "touch-y", vec!["touch"]);
        add_conflict(&mut conflicting_class_groups, "touch-pz", vec!["touch"]);

        let config = Config {
            class_groups,
            conflicting_class_groups,
            class_group_trie: TrieNode::default(),
        };

        // Build the trie for fast lookups
        let class_group_trie = build_class_group_trie(&config);

        Config {
            class_groups: config.class_groups,
            conflicting_class_groups: config.conflicting_class_groups,
            class_group_trie,
        }
    }
}

// Helper functions for building class definitions

fn lit(s: &str) -> ClassDef {
    ClassDef::Literal(s.to_string())
}

fn validator(f: ClassValidator) -> ClassDef {
    ClassDef::Validator(f)
}

fn obj(key: &str, values: Vec<ClassDef>) -> ClassDef {
    let mut map = HashMap::new();
    map.insert(key.to_string(), values);
    ClassDef::Object(map)
}

fn add_class_group(groups: &mut IndexMap<String, Vec<ClassDef>>, name: &str, defs: Vec<ClassDef>) {
    groups.insert(name.to_string(), defs);
}

fn add_conflict(conflicts: &mut HashMap<String, Vec<String>>, name: &str, conflicting: Vec<&str>) {
    conflicts.insert(
        name.to_string(),
        conflicting.iter().map(|s| s.to_string()).collect(),
    );
}

// Scale helper functions matching the JavaScript version

fn scale_position() -> Vec<ClassDef> {
    vec![
        lit("center"),
        lit("top"),
        lit("bottom"),
        lit("left"),
        lit("right"),
        lit("top-left"),
        lit("left-top"), // Deprecated
        lit("top-right"),
        lit("right-top"), // Deprecated
        lit("bottom-right"),
        lit("right-bottom"), // Deprecated
        lit("bottom-left"),
        lit("left-bottom"), // Deprecated
    ]
}

fn scale_position_with_arbitrary() -> Vec<ClassDef> {
    let mut v = scale_position();
    v.push(validator(is_arbitrary_variable));
    v.push(validator(is_arbitrary_value));
    v
}

fn scale_overflow() -> Vec<ClassDef> {
    vec![
        lit("auto"),
        lit("hidden"),
        lit("clip"),
        lit("visible"),
        lit("scroll"),
    ]
}

fn scale_overscroll() -> Vec<ClassDef> {
    vec![lit("auto"), lit("contain"), lit("none")]
}

fn scale_unambiguous_spacing() -> Vec<ClassDef> {
    vec![
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
        lit("px"),
        validator(is_number),
    ]
}

fn scale_inset() -> Vec<ClassDef> {
    let mut v = vec![validator(is_fraction), lit("full"), lit("auto")];
    v.extend(scale_unambiguous_spacing());
    v
}

fn scale_grid_template_cols_rows() -> Vec<ClassDef> {
    vec![
        validator(is_integer),
        lit("none"),
        lit("subgrid"),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_grid_col_row_start_and_end() -> Vec<ClassDef> {
    vec![
        lit("auto"),
        obj(
            "span",
            vec![
                lit("full"),
                validator(is_integer),
                validator(is_arbitrary_variable),
                validator(is_arbitrary_value),
            ],
        ),
        validator(is_integer),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_grid_col_row_start_or_end() -> Vec<ClassDef> {
    vec![
        validator(is_integer),
        lit("auto"),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_grid_auto_cols_rows() -> Vec<ClassDef> {
    vec![
        lit("auto"),
        lit("min"),
        lit("max"),
        lit("fr"),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_align_primary_axis() -> Vec<ClassDef> {
    vec![
        lit("start"),
        lit("end"),
        lit("center"),
        lit("between"),
        lit("around"),
        lit("evenly"),
        lit("stretch"),
        lit("baseline"),
        lit("center-safe"),
        lit("end-safe"),
    ]
}

fn scale_align_secondary_axis() -> Vec<ClassDef> {
    vec![
        lit("start"),
        lit("end"),
        lit("center"),
        lit("stretch"),
        lit("center-safe"),
        lit("end-safe"),
    ]
}

fn scale_margin() -> Vec<ClassDef> {
    let mut v = vec![lit("auto")];
    v.extend(scale_unambiguous_spacing());
    v
}

fn scale_sizing() -> Vec<ClassDef> {
    let mut v = vec![
        validator(is_fraction),
        lit("auto"),
        lit("full"),
        lit("dvw"),
        lit("dvh"),
        lit("lvw"),
        lit("lvh"),
        lit("svw"),
        lit("svh"),
        lit("min"),
        lit("max"),
        lit("fit"),
    ];
    v.extend(scale_unambiguous_spacing());
    v
}

fn scale_color() -> Vec<ClassDef> {
    vec![
        validator(is_any),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_bg_position() -> Vec<ClassDef> {
    let mut v = scale_position();
    v.push(validator(is_arbitrary_variable_position));
    v.push(validator(is_arbitrary_position));
    v.push(obj(
        "position",
        vec![
            validator(is_arbitrary_variable),
            validator(is_arbitrary_value),
        ],
    ));
    v
}

fn scale_bg_repeat() -> Vec<ClassDef> {
    vec![
        lit("no-repeat"),
        obj(
            "repeat",
            vec![lit(""), lit("x"), lit("y"), lit("space"), lit("round")],
        ),
    ]
}

fn scale_bg_size() -> Vec<ClassDef> {
    vec![
        lit("auto"),
        lit("cover"),
        lit("contain"),
        validator(is_arbitrary_variable_size),
        validator(is_arbitrary_size),
        obj(
            "size",
            vec![
                validator(is_arbitrary_variable),
                validator(is_arbitrary_value),
            ],
        ),
    ]
}

fn scale_gradient_stop_position() -> Vec<ClassDef> {
    vec![
        validator(is_percent),
        validator(is_arbitrary_variable_length),
        validator(is_arbitrary_length),
    ]
}

fn scale_radius() -> Vec<ClassDef> {
    vec![
        lit(""),
        lit("none"),
        lit("full"),
        validator(is_tshirt_size),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_border_width() -> Vec<ClassDef> {
    vec![
        lit(""),
        validator(is_number),
        validator(is_arbitrary_variable_length),
        validator(is_arbitrary_length),
    ]
}

fn scale_line_style() -> Vec<ClassDef> {
    vec![lit("solid"), lit("dashed"), lit("dotted"), lit("double")]
}

fn scale_blend_mode() -> Vec<ClassDef> {
    vec![
        lit("normal"),
        lit("multiply"),
        lit("screen"),
        lit("overlay"),
        lit("darken"),
        lit("lighten"),
        lit("color-dodge"),
        lit("color-burn"),
        lit("hard-light"),
        lit("soft-light"),
        lit("difference"),
        lit("exclusion"),
        lit("hue"),
        lit("saturation"),
        lit("color"),
        lit("luminosity"),
    ]
}

fn scale_mask_image_position() -> Vec<ClassDef> {
    vec![
        validator(is_number),
        validator(is_percent),
        validator(is_arbitrary_variable_position),
        validator(is_arbitrary_position),
    ]
}

fn scale_blur() -> Vec<ClassDef> {
    vec![
        lit(""),
        lit("none"),
        validator(is_tshirt_size),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_rotate() -> Vec<ClassDef> {
    vec![
        lit("none"),
        validator(is_number),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_scale() -> Vec<ClassDef> {
    vec![
        lit("none"),
        validator(is_number),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_skew() -> Vec<ClassDef> {
    vec![
        validator(is_number),
        validator(is_arbitrary_variable),
        validator(is_arbitrary_value),
    ]
}

fn scale_translate() -> Vec<ClassDef> {
    let mut v = vec![validator(is_fraction), lit("full")];
    v.extend(scale_unambiguous_spacing());
    v
}
