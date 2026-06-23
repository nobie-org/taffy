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
fn retained_root_solve_refreshes_stable_sibling_descendant_layout_slots() {
    use taffy::{Overflow, Point};

    fn fixed_leaf(tree: &mut TaffyTree<()>, width: Option<f32>, height: Option<f32>) -> NodeId {
        tree.new_leaf(Style {
            size: Size {
                width: width.map(Dimension::length).unwrap_or(Dimension::AUTO),
                height: height.map(Dimension::length).unwrap_or(Dimension::AUTO),
            },
            ..Default::default()
        })
        .unwrap()
    }

    fn full_leaf(tree: &mut TaffyTree<()>) -> NodeId {
        tree.new_leaf(Style {
            size: Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) },
            ..Default::default()
        })
        .unwrap()
    }

    fn block(tree: &mut TaffyTree<()>, children: &[NodeId]) -> NodeId {
        tree.new_with_children(
            Style {
                display: Display::Block,
                min_size: Size { width: Dimension::length(0.0), height: Dimension::length(0.0) },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
            children,
        )
        .unwrap()
    }

    fn full_block(tree: &mut TaffyTree<()>, children: &[NodeId]) -> NodeId {
        tree.new_with_children(
            Style {
                display: Display::Block,
                size: Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                overflow: Point { x: Overflow::Hidden, y: Overflow::Hidden },
                ..Default::default()
            },
            children,
        )
        .unwrap()
    }

    fn flex_row(tree: &mut TaffyTree<()>, children: &[NodeId]) -> NodeId {
        tree.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                min_size: Size { width: Dimension::length(0.0), height: Dimension::length(0.0) },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
            children,
        )
        .unwrap()
    }

    fn flex_column(tree: &mut TaffyTree<()>, children: &[NodeId]) -> NodeId {
        tree.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                min_size: Size { width: Dimension::length(0.0), height: Dimension::length(0.0) },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
            children,
        )
        .unwrap()
    }

    fn panel(tree: &mut TaffyTree<()>, header: NodeId, child: NodeId) -> NodeId {
        tree.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                overflow: Point { x: Overflow::Hidden, y: Overflow::Hidden },
                border: Rect {
                    left: LengthPercentage::length(2.0),
                    right: LengthPercentage::length(0.0),
                    top: LengthPercentage::length(2.0),
                    bottom: LengthPercentage::length(2.0),
                },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
            &[header, child],
        )
        .unwrap()
    }

    fn set_sidebar_content_height(tree: &mut TaffyTree<()>, content: NodeId, height: f32) {
        tree.set_style(
            content,
            Style { size: Size { width: Dimension::AUTO, height: Dimension::length(height) }, ..Default::default() },
        )
        .unwrap();
    }

    let mut tree = TaffyTree::new();
    tree.disable_rounding();

    let canvas = full_leaf(&mut tree);
    let overlay = tree.new_leaf(Style { position: Position::Absolute, ..Default::default() }).unwrap();
    let canvas_host = full_block(&mut tree, &[canvas, overlay]);
    let canvas_flex_child = block(&mut tree, &[canvas_host]);
    let header = fixed_leaf(&mut tree, None, Some(70.0));
    let panel = panel(&mut tree, header, canvas_flex_child);
    let sidebar_content = fixed_leaf(&mut tree, None, Some(0.0));
    let sidebar = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: Size { width: Dimension::length(0.0), height: Dimension::percent(1.0) },
                ..Default::default()
            },
            &[sidebar_content],
        )
        .unwrap();
    let content_row = flex_row(&mut tree, &[panel, sidebar]);
    let padded_row = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                min_size: Size { width: Dimension::length(0.0), height: Dimension::length(0.0) },
                padding: Rect {
                    left: LengthPercentage::length(21.0),
                    right: LengthPercentage::length(21.0),
                    top: LengthPercentage::length(28.0),
                    bottom: LengthPercentage::length(21.0),
                },
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
            &[content_row],
        )
        .unwrap();
    let footer = fixed_leaf(&mut tree, None, Some(98.0));
    let lower_column = flex_column(&mut tree, &[padded_row, footer]);
    let main_row = flex_row(&mut tree, &[lower_column]);
    let top_chrome = fixed_leaf(&mut tree, None, Some(156.0));
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) },
                ..Default::default()
            },
            &[top_chrome, main_row],
        )
        .unwrap();

    let available_space = Size { width: AvailableSpace::Definite(1100.0), height: AvailableSpace::Definite(760.0) };

    set_sidebar_content_height(&mut tree, sidebar_content, 0.0);
    tree.compute_layout(root, available_space).unwrap();
    assert_eq!(tree.layout(canvas_host).unwrap().size, Size { width: 1056.0, height: 383.0 });
    assert_eq!(tree.layout(canvas).unwrap().size, Size { width: 1056.0, height: 383.0 });

    set_sidebar_content_height(&mut tree, sidebar_content, 1.0);
    tree.compute_layout(root, available_space).unwrap();

    assert_eq!(tree.layout(panel).unwrap().size, Size { width: 1058.0, height: 457.0 });
    assert_eq!(tree.layout(canvas_flex_child).unwrap().size, Size { width: 1056.0, height: 383.0 });
    assert_eq!(tree.layout(canvas_host).unwrap().size, Size { width: 1056.0, height: 383.0 });
    assert_eq!(tree.layout(canvas).unwrap().size, Size { width: 1056.0, height: 383.0 });
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
