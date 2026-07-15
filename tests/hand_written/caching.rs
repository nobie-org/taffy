#[cfg(test)]
mod caching {
    use taffy::prelude::*;
    use taffy_test_helpers::{new_test_tree, test_measure_function, TestNodeContext};

    const NODE_CONTEXT: TestNodeContext = TestNodeContext::fixed(50.0, 50.0);

    #[test]
    fn measure_count_flexbox() {
        let mut taffy = new_test_tree();

        let leaf = taffy.new_leaf_with_context(Style::default(), NODE_CONTEXT).unwrap();

        let mut node = taffy.new_with_children(Style::DEFAULT, &[leaf]).unwrap();
        for _ in 0..100 {
            node = taffy.new_with_children(Style::DEFAULT, &[node]).unwrap();
        }

        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, test_measure_function).unwrap();

        // 7 (was 4): a cached measurement now answers only a query with the
        // same known_dimensions — never a query whose known dimension merely
        // equals the cached RESULT size, because a min/max clamp can make that
        // result inconsistent with the content's actual layout width (see
        // Cache::get). The count guards against exponential re-measurement,
        // not against this bounded constant-factor cost.
        assert_eq!(taffy.get_node_context_mut(leaf).unwrap().count, 7);
    }

    #[test]
    #[cfg(feature = "grid")]
    fn measure_count_grid() {
        let mut taffy = new_test_tree();

        let style = || Style { display: Display::Grid, ..Default::default() };
        let leaf = taffy.new_leaf_with_context(style(), NODE_CONTEXT).unwrap();

        let mut node = taffy.new_with_children(Style::DEFAULT, &[leaf]).unwrap();
        for _ in 0..100 {
            node = taffy.new_with_children(Style::DEFAULT, &[node]).unwrap();
        }

        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, test_measure_function).unwrap();
        // 7 (was 4): see measure_count_flexbox — exact-match-only cache hits.
        assert_eq!(taffy.get_node_context_mut(leaf).unwrap().count, 7);
    }

    /// A max-width-clamped flex item must be laid out at its CLAMPED width when
    /// determining the row's cross size — never answered from a cached
    /// max-content measurement whose RESULT width merely equals the clamp.
    ///
    /// The intrinsic pass measures the column's content at unbounded width
    /// (text on one line) and stores a result whose width is clamped to the
    /// max-width. Matching a later `known_dimensions == cached result size`
    /// query against that entry reuses the single-line height for the very
    /// layout (at the clamped width) that would have re-wrapped the text, so
    /// the row ends up one line tall while the text inside it wraps.
    #[test]
    fn max_width_clamped_flex_item_re_measures_at_clamped_width() {
        use taffy_test_helpers::WritingMode;

        let mut taffy = new_test_tree();

        // Ten 10-char words (Ahem: 10px/char): max-content 1000px. At the
        // 600px clamp six words fit per line -> 2 lines of 10px.
        let words: Vec<&str> = core::iter::repeat("AAAAAAAAAA").take(10).collect();
        let text_content = words.join("\u{200B}");
        let text = taffy
            .new_leaf_with_context(
                Style::default(),
                TestNodeContext::ahem_text(text_content, WritingMode::Horizontal),
            )
            .unwrap();

        let column = taffy
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: Dimension::from_percent(0.0),
                    min_size: Size { width: Dimension::from_length(0.0), height: Dimension::auto() },
                    max_size: Size { width: Dimension::from_length(600.0), height: Dimension::auto() },
                    ..Default::default()
                },
                &[text],
            )
            .unwrap();
        let chip = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_length(70.0), height: Dimension::auto() },
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..Default::default()
            })
            .unwrap();
        let row = taffy
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::FLEX_START),
                    gap: Size {
                        width: LengthPercentage::from_length(12.0),
                        height: LengthPercentage::from_length(12.0),
                    },
                    ..Default::default()
                },
                &[chip, column],
            )
            .unwrap();
        let root = taffy
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    size: Size { width: Dimension::from_length(900.0), height: Dimension::auto() },
                    ..Default::default()
                },
                &[row],
            )
            .unwrap();

        taffy.compute_layout_with_measure(root, Size::MAX_CONTENT, test_measure_function).unwrap();

        // Column flex share (818) exceeds the 600 max-width, so the clamp binds.
        assert_eq!(taffy.layout(column).unwrap().size.width, 600.0, "column width");
        assert_eq!(taffy.layout(text).unwrap().size.height, 20.0, "text wraps at the clamped width");
        assert_eq!(taffy.layout(column).unwrap().size.height, 20.0, "column includes the wrapped text");
        assert_eq!(taffy.layout(row).unwrap().size.height, 20.0, "row includes the wrapped text");
    }
}
