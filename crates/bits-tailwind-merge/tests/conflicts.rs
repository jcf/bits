use bits_tailwind_merge::tw_merge;

/// Tests for conflicts across class groups
/// Ported from tailwind-merge/tests/conflicts-across-class-groups.test.ts
#[cfg(test)]
mod cross_group_conflicts {
    use super::*;

    #[test]
    fn test_inset_conflicts() {
        assert_eq!(tw_merge!("inset-1 inset-x-1"), "inset-1 inset-x-1");
        assert_eq!(tw_merge!("inset-x-1 inset-1"), "inset-1");
        assert_eq!(tw_merge!("inset-x-1 left-1 inset-1"), "inset-1");
        assert_eq!(tw_merge!("inset-x-1 inset-1 left-1"), "inset-1 left-1");
        assert_eq!(tw_merge!("inset-x-1 right-1 inset-1"), "inset-1");
        assert_eq!(tw_merge!("inset-x-1 right-1 inset-x-1"), "inset-x-1");
        assert_eq!(
            tw_merge!("inset-x-1 right-1 inset-y-1"),
            "inset-x-1 right-1 inset-y-1"
        );
        assert_eq!(
            tw_merge!("right-1 inset-x-1 inset-y-1"),
            "inset-x-1 inset-y-1"
        );
        assert_eq!(
            tw_merge!("inset-x-1 hover:left-1 inset-1"),
            "hover:left-1 inset-1"
        );
    }

    #[test]
    fn test_ring_and_shadow_no_conflict() {
        assert_eq!(tw_merge!("ring shadow"), "ring shadow");
        assert_eq!(tw_merge!("ring-2 shadow-md"), "ring-2 shadow-md");
        assert_eq!(tw_merge!("shadow ring"), "shadow ring");
        assert_eq!(tw_merge!("shadow-md ring-2"), "shadow-md ring-2");
    }

    #[test]
    fn test_touch_action_conflicts() {
        assert_eq!(tw_merge!("touch-pan-x touch-pan-right"), "touch-pan-right");
        assert_eq!(tw_merge!("touch-none touch-pan-x"), "touch-pan-x");
        assert_eq!(tw_merge!("touch-pan-x touch-none"), "touch-none");
        assert_eq!(
            tw_merge!("touch-pan-x touch-pan-y touch-pinch-zoom"),
            "touch-pan-x touch-pan-y touch-pinch-zoom"
        );
        assert_eq!(
            tw_merge!("touch-manipulation touch-pan-x touch-pan-y touch-pinch-zoom"),
            "touch-pan-x touch-pan-y touch-pinch-zoom"
        );
        assert_eq!(
            tw_merge!("touch-pan-x touch-pan-y touch-pinch-zoom touch-auto"),
            "touch-auto"
        );
    }

    #[test]
    fn test_line_clamp_conflicts() {
        assert_eq!(
            tw_merge!("overflow-auto inline line-clamp-1"),
            "line-clamp-1"
        );
        assert_eq!(
            tw_merge!("line-clamp-1 overflow-auto inline"),
            "line-clamp-1 overflow-auto inline"
        );
    }
}
