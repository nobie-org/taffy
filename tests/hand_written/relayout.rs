use taffy::prelude::*;
use taffy_test_helpers::new_test_tree;

#[test]
fn relayout() {
    let mut taffy = new_test_tree();
    let node1 = taffy
        .new_leaf(taffy::style::Style {
            size: taffy::geometry::Size { width: length(8.0), height: length(80.0) },
            ..Default::default()
        })
        .unwrap();
    let node0 = taffy
        .new_with_children(
            taffy::style::Style {
                align_self: Some(taffy::prelude::AlignSelf::CENTER),
                size: taffy::geometry::Size { width: Dimension::AUTO, height: Dimension::AUTO },
                // size: taffy::geometry::Size { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) },
                ..Default::default()
            },
            &[node1],
        )
        .unwrap();
    let node = taffy
        .new_with_children(
            taffy::style::Style {
                size: taffy::geometry::Size {
                    width: Dimension::from_percent(1f32),
                    height: Dimension::from_percent(1f32),
                },
                ..Default::default()
            },
            &[node0],
        )
        .unwrap();
    taffy
        .compute_layout(
            node,
            taffy::geometry::Size { width: AvailableSpace::Definite(100f32), height: AvailableSpace::Definite(100f32) },
        )
        .unwrap();
    let initial = taffy.layout(node).unwrap().location;
    let initial0 = taffy.layout(node0).unwrap().location;
    let initial1 = taffy.layout(node1).unwrap().location;
    for _ in 1..10 {
        taffy
            .compute_layout(
                node,
                taffy::geometry::Size {
                    width: AvailableSpace::Definite(100f32),
                    height: AvailableSpace::Definite(100f32),
                },
            )
            .unwrap();
        assert_eq!(taffy.layout(node).unwrap().location, initial);
        assert_eq!(taffy.layout(node0).unwrap().location, initial0);
        assert_eq!(taffy.layout(node1).unwrap().location, initial1);
    }
}

#[test]
fn repeated_root_solve_updates_scaled_list_measure_descendant_after_root_constraint_aba() {
    type TestTree =
        (taffy::TaffyTree<()>, taffy::tree::NodeId, taffy::tree::NodeId, taffy::tree::NodeId, taffy::tree::NodeId);

    fn build_tree() -> TestTree {
        let mut taffy = taffy::TaffyTree::new();
        taffy.disable_rounding();
        let block = || Style { display: Display::Block, ..Default::default() };
        let measured = taffy.new_leaf_with_context(block(), ()).unwrap();
        let fixed_width = taffy
            .new_leaf(Style {
                display: Display::Block,
                size: Size { width: length(0.0), height: auto() },
                ..Default::default()
            })
            .unwrap();
        let inner = taffy
            .new_with_children(
                Style {
                    display: Display::Block,
                    size: Size { width: length(0.0), height: length(0.0) },
                    ..Default::default()
                },
                &[measured, fixed_width],
            )
            .unwrap();
        let root = taffy.new_with_children(block(), &[inner]).unwrap();
        (taffy, root, inner, measured, fixed_width)
    }

    let scale_factor = 2.0_f32;
    let available_one = Size {
        width: AvailableSpace::Definite(1.0 * scale_factor),
        height: AvailableSpace::Definite(1.0 * scale_factor),
    };
    let available_two = Size {
        width: AvailableSpace::Definite(1.0 * scale_factor),
        height: AvailableSpace::Definite(2.0 * scale_factor),
    };
    let measure = move |known_dimensions: Size<Option<f32>>,
                        available_space: Size<AvailableSpace>,
                        _node_id,
                        _context: Option<&mut ()>,
                        _style: &Style| {
        let width = known_dimensions.width.unwrap_or(match available_space.width {
            AvailableSpace::Definite(width) => width / scale_factor,
            AvailableSpace::MinContent | AvailableSpace::MaxContent => 0.0,
        });
        let height = match available_space.height {
            AvailableSpace::Definite(height) => 1.0_f32.min(height / scale_factor),
            AvailableSpace::MinContent | AvailableSpace::MaxContent => 1.0,
        };
        Size { width: (width * scale_factor).ceil(), height: (height * scale_factor).ceil() }
    };

    let (mut retained, retained_root, retained_inner, retained_measured, retained_fixed_width) = build_tree();
    retained.compute_layout_with_measure(retained_root, available_one, measure).unwrap();
    retained.compute_layout_with_measure(retained_root, available_two, measure).unwrap();
    retained.compute_layout_with_measure(retained_root, available_one, measure).unwrap();

    let (mut fresh, fresh_root, fresh_inner, fresh_measured, fresh_fixed_width) = build_tree();
    fresh.compute_layout_with_measure(fresh_root, available_one, measure).unwrap();

    assert_eq!(retained.layout(retained_root).unwrap(), fresh.layout(fresh_root).unwrap());
    assert_eq!(retained.layout(retained_inner).unwrap(), fresh.layout(fresh_inner).unwrap());
    assert_eq!(retained.layout(retained_measured).unwrap(), fresh.layout(fresh_measured).unwrap());
    assert_eq!(retained.layout(retained_fixed_width).unwrap(), fresh.layout(fresh_fixed_width).unwrap());
}

#[test]
fn toggle_root_display_none() {
    let hidden_style = Style {
        display: Display::None,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    let flex_style = Style {
        display: Display::Flex,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    // Setup
    let mut taffy = new_test_tree();
    let node = taffy.new_leaf(hidden_style.clone()).unwrap();

    // Layout 1 (None)
    taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);

    // Layout 2 (Flex)
    taffy.set_style(node, flex_style).unwrap();
    taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 100.0);

    // Layout 3 (None)
    taffy.set_style(node, hidden_style).unwrap();
    taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);
}

#[test]
fn toggle_root_display_none_with_children() {
    use taffy::prelude::*;

    let mut taffy = new_test_tree();

    let child = taffy
        .new_leaf(Style { size: Size { width: length(800.0), height: length(100.0) }, ..Default::default() })
        .unwrap();

    let parent = taffy
        .new_with_children(
            Style { size: Size { width: length(800.0), height: length(100.0) }, ..Default::default() },
            &[child],
        )
        .unwrap();

    let root = taffy.new_with_children(Style::default(), &[parent]).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(child).unwrap().size.width, 800.0);
    assert_eq!(taffy.layout(child).unwrap().size.height, 100.0);

    taffy.set_style(root, Style { display: Display::None, ..Default::default() }).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(child).unwrap().size.width, 0.0);
    assert_eq!(taffy.layout(child).unwrap().size.height, 0.0);

    taffy.set_style(root, Style::default()).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(taffy.layout(parent).unwrap().size.width, 800.0);
    assert_eq!(taffy.layout(parent).unwrap().size.height, 100.0);
    assert_eq!(taffy.layout(child).unwrap().size.width, 800.0);
    assert_eq!(taffy.layout(child).unwrap().size.height, 100.0);
}

#[test]
fn toggle_flex_child_display_none() {
    let hidden_style = Style {
        display: Display::None,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    let flex_style = Style {
        display: Display::Flex,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    // Setup
    let mut taffy = new_test_tree();
    let node = taffy.new_leaf(hidden_style.clone()).unwrap();
    let root = taffy.new_with_children(flex_style.clone(), &[node]).unwrap();

    // Layout 1 (None)
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);

    // Layout 2 (Flex)
    taffy.set_style(node, flex_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 100.0);

    // Layout 3 (None)
    taffy.set_style(node, hidden_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);
}

#[test]
fn toggle_flex_container_display_none() {
    let hidden_style = Style {
        display: Display::None,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    let flex_style = Style {
        display: Display::Flex,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    // Setup
    let mut taffy = new_test_tree();
    let node = taffy.new_leaf(hidden_style.clone()).unwrap();
    let root = taffy.new_with_children(hidden_style.clone(), &[node]).unwrap();

    // Layout 1 (None)
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);

    // Layout 2 (Flex)
    taffy.set_style(root, flex_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 100.0);

    // Layout 3 (None)
    taffy.set_style(root, hidden_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);
}

#[test]
fn toggle_grid_child_display_none() {
    let hidden_style = Style {
        display: Display::None,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    let grid_style = Style {
        display: Display::Grid,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    // Setup
    let mut taffy = new_test_tree();
    let node = taffy.new_leaf(hidden_style.clone()).unwrap();
    let root = taffy.new_with_children(grid_style.clone(), &[node]).unwrap();

    // Layout 1 (None)
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);

    // Layout 2 (Flex)
    taffy.set_style(node, grid_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 100.0);

    // Layout 3 (None)
    taffy.set_style(node, hidden_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(node).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);
}

#[test]
fn toggle_grid_container_display_none() {
    let hidden_style = Style {
        display: Display::None,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    let grid_style = Style {
        display: Display::Grid,
        size: Size { width: length(100.0), height: length(100.0) },
        ..Default::default()
    };

    // Setup
    let mut taffy = new_test_tree();
    let node = taffy.new_leaf(hidden_style.clone()).unwrap();
    let root = taffy.new_with_children(hidden_style.clone(), &[node]).unwrap();

    // Layout 1 (None)
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);

    // Layout 2 (Flex)
    taffy.set_style(root, grid_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 100.0);
    assert_eq!(layout.size.height, 100.0);

    // Layout 3 (None)
    taffy.set_style(root, hidden_style).unwrap();
    taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();
    let layout = taffy.layout(root).unwrap();
    assert_eq!(layout.location.x, 0.0);
    assert_eq!(layout.location.y, 0.0);
    assert_eq!(layout.size.width, 0.0);
    assert_eq!(layout.size.height, 0.0);
}

#[test]
fn relayout_is_stable_with_rounding() {
    let mut taffy = new_test_tree();
    taffy.enable_rounding();

    // <div style="width: 1920px; height: 1080px">
    //     <div style="width: 100%; left: 1.5px">
    //         <div style="width: 150px; justify-content: end">
    //             <div style="min-width: 300px" />
    //         </div>
    //     </div>
    // </div>

    let inner =
        taffy.new_leaf(Style { min_size: Size { width: length(300.), height: auto() }, ..Default::default() }).unwrap();
    let wrapper = taffy
        .new_with_children(
            Style {
                size: Size { width: length(150.), height: auto() },
                justify_content: Some(JustifyContent::END),
                ..Default::default()
            },
            &[inner],
        )
        .unwrap();
    let outer = taffy
        .new_with_children(
            Style {
                size: Size { width: percent(1.), height: auto() },
                inset: Rect { left: length(1.5), right: auto(), top: auto(), bottom: auto() },
                ..Default::default()
            },
            &[wrapper],
        )
        .unwrap();
    let root = taffy
        .new_with_children(
            Style { size: Size { width: length(1920.), height: length(1080.) }, ..Default::default() },
            &[outer],
        )
        .unwrap();

    // Compute and assert initial layout.

    taffy.compute_layout(root, Size::MAX_CONTENT).ok();
    taffy.print_tree(root);

    let initial_root_layout = taffy.layout(root).unwrap().clone();
    assert_eq!(initial_root_layout.location.x, 0.0);
    assert_eq!(initial_root_layout.location.y, 0.0);
    assert_eq!(initial_root_layout.size.width, 1920.0);
    assert_eq!(initial_root_layout.size.height, 1080.0);

    let initial_outer_layout = taffy.layout(outer).unwrap().clone();
    assert_eq!(initial_outer_layout.location.x, 2.0);
    assert_eq!(initial_outer_layout.location.y, 0.0);
    assert_eq!(initial_outer_layout.size.width, 1920.0);
    assert_eq!(initial_outer_layout.size.height, 1080.0);

    let initial_wrapper_layout = taffy.layout(wrapper).unwrap().clone();
    assert_eq!(initial_wrapper_layout.location.x, 0.0);
    assert_eq!(initial_wrapper_layout.location.y, 0.0);
    assert_eq!(initial_wrapper_layout.size.width, 150.0);
    assert_eq!(initial_wrapper_layout.size.height, 1080.0);

    let initial_inner_layout = taffy.layout(inner).unwrap().clone();
    assert_eq!(initial_inner_layout.location.x, -150.0);
    assert_eq!(initial_inner_layout.location.y, 0.0);
    assert_eq!(initial_inner_layout.size.width, 300.0);
    assert_eq!(initial_inner_layout.size.height, 1080.0);

    // Recompute and assert that new layout marks initial layout each time
    for _ in 0..5 {
        taffy.mark_dirty(root).ok();
        taffy.compute_layout(root, Size::MAX_CONTENT).ok();
        taffy.print_tree(root);

        let root_layout = taffy.layout(root).unwrap();
        assert_eq!(initial_root_layout.location.x, root_layout.location.x);
        assert_eq!(initial_root_layout.location.y, root_layout.location.y);
        assert_eq!(initial_root_layout.size.width, root_layout.size.width);
        assert_eq!(initial_root_layout.size.height, root_layout.size.height);
        let outer_layout = taffy.layout(outer).unwrap();
        assert_eq!(initial_outer_layout.location.x, outer_layout.location.x);
        assert_eq!(initial_outer_layout.location.y, outer_layout.location.y);
        assert_eq!(initial_outer_layout.size.width, outer_layout.size.width);
        assert_eq!(initial_outer_layout.size.height, outer_layout.size.height);
        let wrapper_layout = taffy.layout(wrapper).unwrap();
        assert_eq!(initial_wrapper_layout.location.x, wrapper_layout.location.x);
        assert_eq!(initial_wrapper_layout.location.x, wrapper_layout.location.y);
        assert_eq!(initial_wrapper_layout.size.width, wrapper_layout.size.width);
        assert_eq!(initial_wrapper_layout.size.height, wrapper_layout.size.height);
        let inner_layout = taffy.layout(inner).unwrap();
        assert_eq!(initial_inner_layout.location.x, inner_layout.location.x);
        assert_eq!(initial_inner_layout.location.y, inner_layout.location.y);
        assert_eq!(initial_inner_layout.size.width, inner_layout.size.width);
        assert_eq!(initial_inner_layout.size.height, inner_layout.size.height);
    }
}
