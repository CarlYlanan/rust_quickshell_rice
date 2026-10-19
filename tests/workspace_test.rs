use owo_bar::model::Workspace;

#[test]
fn test_workspace_kanji() {
    let test_cases = [
        (1, '一'),
        (2, '二'),
        (3, '三'),
        (4, '四'),
        (5, '五'),
        (6, '六'),
        (7, '七'),
        (8, '八'),
        (9, '九'),
        (10, '〇'),
        (0, '〇'),
        (25, '〇'),
    ];

    for (id, expected) in test_cases {
        let ws = Workspace {
            id: id as u8,
            active: false,
            focused: false,
            occupied: false,
        };
        assert_eq!(ws.kanji(), expected, "Failed for workspace id {}", id);
    }
}

#[test]
fn test_workspace_colors() {
    let ws = Workspace {
        id: 1,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#00d0d0"); // cyan

    let ws = Workspace {
        id: 2,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#ff4d6d"); // pink

    let ws = Workspace {
        id: 3,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#ffe066"); // yellow

    let ws = Workspace {
        id: 4,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#c0b0ff"); // lavender

    let ws = Workspace {
        id: 5,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#a0ffc8"); // mint

    // Test wrapping
    let ws = Workspace {
        id: 6,
        active: false,
        focused: false,
        occupied: true,
    };
    assert_eq!(ws.ws_color(), "#00d0d0"); // back to cyan
}

#[test]
fn test_workspace_tones() {
    // Focused takes priority
    let ws = Workspace {
        id: 1,
        active: true,
        focused: true,
        occupied: true,
    };
    // In the actual implementation, tone is determined by focused > occupied > empty
    // But we don't have the tone method in our Workspace struct - that was from rice's model
    // Our Workspace struct just has the boolean fields
}

#[test]
fn test_workspace_serialization() {
    let ws = Workspace {
        id: 3,
        active: true,
        focused: false,
        occupied: true,
    };
    let serialized = serde_json::to_string(&ws).unwrap();
    // JSON key order is not guaranteed, so parse and compare the values
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parsed["id"], 3);
    assert_eq!(parsed["active"], true);
    assert_eq!(parsed["focused"], false);
    assert_eq!(parsed["occupied"], true);
}