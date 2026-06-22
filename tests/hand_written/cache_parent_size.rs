use taffy::prelude::*;

#[test]
fn repeated_root_solve_updates_percent_descendant_after_root_constraint_change() {
    type TestTree = (
        taffy::TaffyTree<Size<f32>>,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
    );

    fn build_tree() -> TestTree {
        let mut taffy = taffy::TaffyTree::new();
        taffy.disable_rounding();

        let measured = taffy.new_leaf_with_context(Style::default(), Size { width: 0.0, height: 1.0 }).unwrap();
        let full = taffy
            .new_leaf(Style { size: Size { width: percent(1.0_f32), height: percent(1.0_f32) }, ..Default::default() })
            .unwrap();
        let parent = taffy.new_with_children(Style::default(), &[measured, full]).unwrap();
        let root = taffy
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    size: Size { width: percent(1.0_f32), height: percent(1.0_f32) },
                    ..Default::default()
                },
                &[parent],
            )
            .unwrap();

        (taffy, root, parent, measured, full)
    }

    let measure = |known_dimensions: Size<Option<f32>>,
                   available_space: Size<AvailableSpace>,
                   _node_id,
                   context: Option<&mut Size<f32>>,
                   _style: &Style| {
        let fallback = context.cloned().unwrap_or(Size::ZERO);
        Size {
            width: known_dimensions.width.unwrap_or_else(|| match available_space.width {
                AvailableSpace::Definite(width) => fallback.width.min(width),
                AvailableSpace::MinContent | AvailableSpace::MaxContent => fallback.width,
            }),
            height: known_dimensions.height.unwrap_or_else(|| match available_space.height {
                AvailableSpace::Definite(height) => fallback.height.min(height),
                AvailableSpace::MinContent | AvailableSpace::MaxContent => fallback.height,
            }),
        }
    };

    let available_one = Size { width: AvailableSpace::Definite(1.0), height: AvailableSpace::Definite(1.0) };
    let available_two = Size { width: AvailableSpace::Definite(1.0), height: AvailableSpace::Definite(2.0) };

    let (mut retained, retained_root, retained_parent, retained_measured, retained_full) = build_tree();
    retained.compute_layout_with_measure(retained_root, available_one, measure).unwrap();
    retained.compute_layout_with_measure(retained_root, available_two, measure).unwrap();

    let (mut fresh, fresh_root, fresh_parent, fresh_measured, fresh_full) = build_tree();
    fresh.compute_layout_with_measure(fresh_root, available_two, measure).unwrap();

    assert_eq!(retained.layout(retained_full).unwrap().size.height, 2.0);
    assert_eq!(fresh.layout(fresh_full).unwrap().size.height, 2.0);
    assert_eq!(retained.layout(retained_root).unwrap(), fresh.layout(fresh_root).unwrap());
    assert_eq!(retained.layout(retained_parent).unwrap(), fresh.layout(fresh_parent).unwrap());
    assert_eq!(retained.layout(retained_measured).unwrap(), fresh.layout(fresh_measured).unwrap());
    assert_eq!(retained.layout(retained_full).unwrap(), fresh.layout(fresh_full).unwrap());
}

#[test]
fn repeated_root_solve_refreshes_container_descendant_layouts() {
    type TestTree = (
        taffy::TaffyTree<Size<f32>>,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
        taffy::tree::NodeId,
    );

    fn build_tree() -> TestTree {
        let mut taffy = taffy::TaffyTree::new();
        taffy.disable_rounding();

        let block = || Style { display: Display::Block, ..Default::default() };

        let measured = taffy.new_leaf_with_context(block(), Size { width: 0.0, height: 1.0 }).unwrap();
        let full = taffy
            .new_leaf(Style {
                display: Display::Block,
                size: Size { width: percent(1.0_f32), height: percent(1.0_f32) },
                ..Default::default()
            })
            .unwrap();
        let parent = taffy.new_with_children(block(), &[measured, full]).unwrap();
        let root = taffy
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::NoWrap,
                    ..Default::default()
                },
                &[parent],
            )
            .unwrap();

        (taffy, root, parent, measured, full)
    }

    let measure = |known_dimensions: Size<Option<f32>>,
                   available_space: Size<AvailableSpace>,
                   _node_id,
                   context: Option<&mut Size<f32>>,
                   _style: &Style| {
        let fallback = context.cloned().unwrap_or(Size::ZERO);
        Size {
            width: known_dimensions.width.unwrap_or_else(|| match available_space.width {
                AvailableSpace::Definite(width) => fallback.width.min(width),
                AvailableSpace::MinContent | AvailableSpace::MaxContent => fallback.width,
            }),
            height: known_dimensions.height.unwrap_or_else(|| match available_space.height {
                AvailableSpace::Definite(height) => fallback.height.min(height),
                AvailableSpace::MinContent | AvailableSpace::MaxContent => fallback.height,
            }),
        }
    };

    let available_one = Size { width: AvailableSpace::Definite(1.0), height: AvailableSpace::Definite(1.0) };
    let available_two = Size { width: AvailableSpace::Definite(1.0), height: AvailableSpace::Definite(2.0) };

    let (mut retained, retained_root, retained_parent, retained_measured, retained_full) = build_tree();
    retained.compute_layout_with_measure_and_cache_events(retained_root, available_one, measure, |_| {}).unwrap();
    retained.compute_layout_with_measure_and_cache_events(retained_root, available_two, measure, |_| {}).unwrap();

    let (mut fresh, fresh_root, fresh_parent, fresh_measured, fresh_full) = build_tree();
    fresh.compute_layout_with_measure(fresh_root, available_two, measure).unwrap();

    assert_eq!(retained.layout(retained_root).unwrap(), fresh.layout(fresh_root).unwrap());
    assert_eq!(retained.layout(retained_parent).unwrap(), fresh.layout(fresh_parent).unwrap());
    assert_eq!(retained.layout(retained_measured).unwrap(), fresh.layout(fresh_measured).unwrap());
    assert_eq!(retained.layout(retained_full).unwrap(), fresh.layout(fresh_full).unwrap());
}
