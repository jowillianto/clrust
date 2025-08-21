use clrust::{Color, TerminalNode, TerminalNodes, TextEffect, TextFormat};

#[test]
fn test_text_format_new() {
    let format = TextFormat::new();
    assert_eq!(format.get_bg(), None);
    assert_eq!(format.get_fg(), None);
    assert_eq!(format.len_effects(), 0);
}

#[test]
fn test_format_pattern_taken() {
    let mut format = TextFormat::new().bg(Color::Red).take();

    // Test setting and chaining
    assert_eq!(format.get_bg(), Some(&Color::Red));

    // Test overwriting
    format.bg(Color::Blue);
    assert_eq!(format.get_bg(), Some(&Color::Blue));
}

#[test]
fn test_format_builder_pattern_fg() {
    let format = TextFormat::new().fg(Color::Green).take();
    assert_eq!(format.get_fg(), Some(&Color::Green));
}

#[test]
fn test_format_builder_pattern_effect() {
    let mut format = TextFormat::new().effect(TextEffect::Bold).take();

    assert!(format.has_effect(&TextEffect::Bold));
    assert_eq!(format.len_effects(), 1);

    // Test adding another effect
    format.effect(TextEffect::Italic);
    assert!(format.has_effect(&TextEffect::Bold));
    assert!(format.has_effect(&TextEffect::Italic));
    assert_eq!(format.len_effects(), 2);

    // Test duplicate effect (HashSet behavior)
    format.effect(TextEffect::Bold);
    assert_eq!(format.len_effects(), 2);
}

#[test]
fn test_format_builder_pattern_effects() {
    let mut format = TextFormat::new();

    let effects_vec = vec![TextEffect::Bold, TextEffect::Italic, TextEffect::Underline];
    format.effects(effects_vec);

    assert_eq!(format.len_effects(), 3);

    // Test adding more effects
    format.effects([TextEffect::Strikethrough, TextEffect::Bold]); // Bold is duplicate
    assert_eq!(format.len_effects(), 4);
}

#[test]
fn test_format_method_chaining() {
    let mut format = TextFormat::new();

    // Test full method chaining works
    format
        .bg(Color::Black)
        .fg(Color::White)
        .effect(TextEffect::Bold)
        .effect(TextEffect::Underline)
        .effects(vec![TextEffect::Italic, TextEffect::Strikethrough]);

    assert_eq!(format.get_bg(), Some(&Color::Black));
    assert_eq!(format.get_fg(), Some(&Color::White));
    assert!(format.has_effect(&TextEffect::Bold));
    assert!(format.has_effect(&TextEffect::Underline));
    assert!(format.has_effect(&TextEffect::Italic));
    assert!(format.has_effect(&TextEffect::Strikethrough));
    assert_eq!(format.len_effects(), 4);
}

#[test]
fn test_format_has_effect() {
    let mut format = TextFormat::new();
    format.effect(TextEffect::Bold);
    format.effect(TextEffect::Italic);

    assert!(format.has_effect(&TextEffect::Bold));
    assert!(format.has_effect(&TextEffect::Italic));
    assert!(!format.has_effect(&TextEffect::Underline));
}

#[test]
fn test_format_take() {
    let mut format = TextFormat::new();
    format.bg(Color::Red);
    format.fg(Color::Blue);
    format.effect(TextEffect::Bold);

    let taken = format.take();

    // Original should be reset to default
    assert_eq!(format.get_bg(), None);
    assert_eq!(format.get_fg(), None);
    assert_eq!(format.len_effects(), 0);

    // Taken should have the original values
    assert_eq!(taken.get_bg(), Some(&Color::Red));
    assert_eq!(taken.get_fg(), Some(&Color::Blue));
    assert!(taken.has_effect(&TextEffect::Bold));
    assert_eq!(taken.len_effects(), 1);
}

#[test]
fn test_format_getters() {
    let mut format = TextFormat::new();

    // Initially None
    assert_eq!(format.get_bg(), None);
    assert_eq!(format.get_fg(), None);

    // After setting
    format.bg(Color::Cyan).fg(Color::White);
    assert_eq!(format.get_bg(), Some(&Color::Cyan));
    assert_eq!(format.get_fg(), Some(&Color::White));
}

#[test]
fn test_format_effects_with_different_iterators() {
    let mut format = TextFormat::new();

    // Test with Vec
    format.effects(vec![TextEffect::Bold, TextEffect::Italic]);
    assert_eq!(format.len_effects(), 2);

    // Test with array
    format.effects([TextEffect::Underline, TextEffect::Strikethrough]);
    assert_eq!(format.len_effects(), 4);

    // Test with empty iterator
    format.effects(std::iter::empty::<TextEffect>());
    assert_eq!(format.len_effects(), 4); // No change
}

#[test]
fn test_terminal_nodes_new() {
    let nodes = TerminalNodes::new(4);
    assert_eq!(nodes.indent(), 4);
    assert_eq!(nodes.len(), 1);

    // Should start with an Indent node - verify through iteration
    let first_node = nodes.iter().next().unwrap();
    match first_node {
        TerminalNode::Indent(indent) => assert_eq!(*indent, 4),
        _ => panic!("Expected Indent node"),
    }
}

#[test]
fn test_terminal_nodes_default() {
    let nodes = TerminalNodes::default();
    assert_eq!(nodes.indent(), 0);
    assert_eq!(nodes.len(), 1);

    let first_node = nodes.iter().next().unwrap();
    match first_node {
        TerminalNode::Indent(indent) => assert_eq!(*indent, 0),
        _ => panic!("Expected Indent(0) node"),
    }
}

#[test]
fn test_terminal_node_from_text_format() {
    let format = TextFormat::new().fg(Color::Red).clone();
    let node = TerminalNode::from(format.clone());

    match node {
        TerminalNode::Begin(fmt) => assert_eq!(fmt, format),
        _ => panic!("Expected Begin node"),
    }
}

#[test]
fn test_terminal_node_from_string_types() {
    // Test String
    let node1 = TerminalNode::from(String::from("hello"));
    match node1 {
        TerminalNode::Text(text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Text node"),
    }

    // Test &str
    let node2 = TerminalNode::from("world");
    match node2 {
        TerminalNode::Text(text) => assert_eq!(text, "world"),
        _ => panic!("Expected Text node"),
    }
}

#[test]
fn test_terminal_append_node_basic() {
    let mut nodes = TerminalNodes::new(2);
    let initial_len = nodes.len();

    nodes.append_node("hello");
    assert_eq!(nodes.len(), initial_len + 1);

    // Verify the text was added by checking the last node
    let last_node = nodes.iter().last().unwrap();
    match last_node {
        TerminalNode::Text(text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Text node"),
    }
}

#[test]
fn test_terminal_automatic_indentation_after_newline() {
    let mut nodes = TerminalNodes::new(4);

    nodes
        .append_node("first line")
        .new_line()
        .append_node("second line");

    // Should be: Indent(4), Text("first line"), NewLine, Indent(4), Text("second line")
    assert_eq!(nodes.len(), 5);

    let collected_nodes: Vec<_> = nodes.iter().collect();

    match collected_nodes[0] {
        TerminalNode::Indent(indent) => assert_eq!(*indent, 4),
        _ => panic!("Expected initial Indent"),
    }

    match collected_nodes[1] {
        TerminalNode::Text(text) => assert_eq!(text, "first line"),
        _ => panic!("Expected first Text"),
    }

    match collected_nodes[2] {
        TerminalNode::NewLine => {}
        _ => panic!("Expected NewLine"),
    }

    match collected_nodes[3] {
        TerminalNode::Indent(indent) => assert_eq!(*indent, 4),
        _ => panic!("Expected auto Indent after NewLine"),
    }

    match collected_nodes[4] {
        TerminalNode::Text(text) => assert_eq!(text, "second line"),
        _ => panic!("Expected second Text"),
    }
}

#[test]
fn test_terminal_multiple_newlines_indentation() {
    let mut nodes = TerminalNodes::new(2);

    nodes
        .append_node("line1")
        .new_line()
        .append_node("line2")
        .new_line()
        .new_line() // Multiple newlines
        .append_node("line3");

    // Should auto-indent after each newline before adding content
    let expected_sequence = vec![
        TerminalNode::Indent(2),
        TerminalNode::Text("line1".to_string()),
        TerminalNode::NewLine,
        TerminalNode::Indent(2),
        TerminalNode::Text("line2".to_string()),
        TerminalNode::NewLine,
        TerminalNode::Indent(2),
        TerminalNode::NewLine,
        TerminalNode::Indent(2),
        TerminalNode::Text("line3".to_string()),
    ];

    let collected_nodes: Vec<_> = nodes.iter().collect();
    assert_eq!(collected_nodes.len(), expected_sequence.len());

    for (actual, expected) in collected_nodes.iter().zip(expected_sequence.iter()) {
        match (actual, expected) {
            (TerminalNode::Indent(a), TerminalNode::Indent(e)) => assert_eq!(a, e),
            (TerminalNode::Text(a), TerminalNode::Text(e)) => assert_eq!(a, e),
            (TerminalNode::NewLine, TerminalNode::NewLine) => {}
            _ => panic!("Mismatch: {:?} vs {:?}", actual, expected),
        }
    }
}

#[test]
fn test_terminal_no_indentation_when_not_after_newline() {
    let mut nodes = TerminalNodes::new(3);

    nodes
        .append_node("hello")
        .append_node(" ")
        .append_node("world");

    // Should not add extra indents between regular nodes
    assert_eq!(nodes.len(), 4); // Initial indent + 3 text nodes

    let collected_nodes: Vec<_> = nodes.iter().collect();

    match collected_nodes[1] {
        TerminalNode::Text(text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Text"),
    }

    match collected_nodes[2] {
        TerminalNode::Text(text) => assert_eq!(text, " "),
        _ => panic!("Expected Text"),
    }

    match collected_nodes[3] {
        TerminalNode::Text(text) => assert_eq!(text, "world"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_terminal_append_sub_node() {
    let mut main_nodes = TerminalNodes::new(2);
    let mut sub_nodes = TerminalNodes::new(4);

    sub_nodes.append_node("sub1").append_node("sub2");
    let sub_len = sub_nodes.len();

    let result = main_nodes.append_node("main").append_sub_node(sub_nodes);

    // Should return self for chaining
    assert!(std::ptr::eq(result, &main_nodes));

    // Should have: Indent(2), Text("main"), + all sub_nodes
    assert_eq!(main_nodes.len(), 2 + sub_len);

    let collected_nodes: Vec<_> = main_nodes.iter().collect();

    // Find the sub nodes' initial indent
    let mut found_sub_indent = false;
    let mut found_sub1 = false;
    for node in &collected_nodes {
        match node {
            TerminalNode::Indent(4) => found_sub_indent = true,
            TerminalNode::Text(text) if text == "sub1" => found_sub1 = true,
            _ => {}
        }
    }

    assert!(found_sub_indent, "Expected sub nodes' initial indent");
    assert!(found_sub1, "Expected sub1 text");
}

#[test]
fn test_terminal_append_sub_node_with_newlines() {
    let mut main_nodes = TerminalNodes::new(1);
    let mut sub_nodes = TerminalNodes::new(3);

    sub_nodes.append_node("sub1").new_line().append_node("sub2");

    main_nodes
        .append_node("main")
        .new_line()
        .append_sub_node(sub_nodes);

    // The sub_nodes should maintain their indentation logic when appended
    let text_nodes: Vec<_> = main_nodes
        .iter()
        .filter_map(|node| match node {
            TerminalNode::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(text_nodes, vec!["main", "sub1", "sub2"]);
}

#[test]
fn test_terminal_format_methods() {
    let mut nodes = TerminalNodes::new(0);
    let format = TextFormat::new().fg(Color::Blue).clone();

    nodes
        .begin_format(format.clone())
        .append_node("formatted text")
        .end_format();

    // Should have: Indent(0), Begin(format), Text("formatted text"), End
    assert_eq!(nodes.len(), 4);

    let collected_nodes: Vec<_> = nodes.iter().collect();

    match collected_nodes[1] {
        TerminalNode::Begin(fmt) => assert_eq!(*fmt, format),
        _ => panic!("Expected Begin node"),
    }

    match collected_nodes[3] {
        TerminalNode::End => {}
        _ => panic!("Expected End node"),
    }
}

#[test]
fn test_terminal_with_format_constructor() {
    let format = TextFormat::new().fg(Color::Red).clone();
    let nodes = TerminalNodes::with_format(format.clone(), "hello", 3);

    // Should have: Indent(3), Begin(format), Text("hello"), End
    assert_eq!(nodes.len(), 4);
    assert_eq!(nodes.indent(), 3);

    let collected_nodes: Vec<_> = nodes.iter().collect();

    match collected_nodes[1] {
        TerminalNode::Begin(fmt) => assert_eq!(*fmt, format),
        _ => panic!("Expected Begin node"),
    }

    match collected_nodes[2] {
        TerminalNode::Text(text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Text node"),
    }
}

#[test]
fn test_terminal_method_chaining() {
    let format = TextFormat::new().effect(TextEffect::Bold).clone();

    let mut nodes = TerminalNodes::new(2);
    nodes
        .append_node("start")
        .new_line()
        .begin_format(format)
        .append_node("bold text")
        .end_format()
        .new_line()
        .append_node("end");

    // Verify chaining worked by checking key nodes
    let text_nodes: Vec<_> = nodes
        .iter()
        .filter_map(|node| match node {
            TerminalNode::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(text_nodes, vec!["start", "bold text", "end"]);
}

#[test]
fn test_terminal_len_and_iter() {
    let mut nodes = TerminalNodes::new(1);
    nodes.append_node("test").new_line().append_node("test2");

    assert_eq!(nodes.len(), 5); // Indent(1), Text("test"), NewLine, Indent(1), Text("test2")

    let collected: Vec<_> = nodes.iter().collect();
    assert_eq!(collected.len(), 5);
}

#[test]
fn test_terminal_take() {
    let mut nodes = TerminalNodes::new(3);
    nodes.append_node("original");
    let original_len = nodes.len();

    let taken = nodes.take();

    // Original should be default
    assert_eq!(nodes.indent(), 0);
    assert_eq!(nodes.len(), 1); // Just the default Indent(0)

    // Taken should have original values
    assert_eq!(taken.indent(), 3);
    assert_eq!(taken.len(), original_len);
}

#[test]
fn test_terminal_edge_case_empty_string() {
    let mut nodes = TerminalNodes::new(1);
    nodes.append_node("");

    let last_node = nodes.iter().last().unwrap();
    match last_node {
        TerminalNode::Text(text) => assert_eq!(text, ""),
        _ => panic!("Expected empty text node"),
    }
}

#[test]
fn test_terminal_different_indent_levels() {
    let nodes_0 = TerminalNodes::new(0);
    let nodes_5 = TerminalNodes::new(5);
    let nodes_10 = TerminalNodes::new(10);

    assert_eq!(nodes_0.indent(), 0);
    assert_eq!(nodes_5.indent(), 5);
    assert_eq!(nodes_10.indent(), 10);

    // Each should start with their respective indent
    let first_0 = nodes_0.iter().next().unwrap();
    match first_0 {
        TerminalNode::Indent(i) => assert_eq!(*i, 0),
        _ => panic!("Expected Indent(0)"),
    }

    let first_5 = nodes_5.iter().next().unwrap();
    match first_5 {
        TerminalNode::Indent(i) => assert_eq!(*i, 5),
        _ => panic!("Expected Indent(5)"),
    }

    let first_10 = nodes_10.iter().next().unwrap();
    match first_10 {
        TerminalNode::Indent(i) => assert_eq!(*i, 10),
        _ => panic!("Expected Indent(10)"),
    }
}

#[test]
fn test_terminal_indent_getter() {
    let nodes = TerminalNodes::new(7);
    assert_eq!(nodes.indent(), 7);

    // Test with default
    let default_nodes = TerminalNodes::default();
    assert_eq!(default_nodes.indent(), 0);

    // Test with with_format
    let format = TextFormat::new().fg(Color::Red).clone();
    let formatted_nodes = TerminalNodes::with_format(format, "test", 3);
    assert_eq!(formatted_nodes.indent(), 3);
}
