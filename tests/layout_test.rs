use photon_ui::layout::{Constraint, Rect, layout::Layout};

#[test]
fn test_dashboard_layout_areas_at_narrow_width() {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(1),
    ]);
    let areas = layout.split(Rect::new(0, 0, 25, 24));
    for (i, area) in areas.iter().enumerate() {
        println!("area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    let body_layout = Layout::horizontal([
        Constraint::Length(16),
        Constraint::Min(10),
    ]);
    let body_areas = body_layout.split(areas[2]);
    for (i, area) in body_areas.iter().enumerate() {
        println!("body_area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(2),
    ]);
    let main_areas = main_layout.split(body_areas[1]);
    for (i, area) in main_areas.iter().enumerate() {
        println!("main_area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    let system_layout = Layout::horizontal([
        Constraint::Length(14),
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Min(5),
    ]);
    let system_areas = system_layout.split(main_areas[14]);
    for (i, area) in system_areas.iter().enumerate() {
        println!("system_area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    // Verify no area exceeds the terminal width
    for area in &areas {
        assert!(area.x + area.width <= 25, "area exceeds terminal width: {:?}", area);
    }
    for area in &body_areas {
        assert!(area.x + area.width <= 25, "body_area exceeds terminal width: {:?}", area);
    }
    for area in &main_areas {
        assert!(area.x + area.width <= 25, "main_area exceeds terminal width: {:?}", area);
    }
    for area in &system_areas {
        assert!(area.x + area.width <= 25, "system_area exceeds terminal width: {:?}", area);
    }
}

#[test]
fn test_dashboard_layout_areas_at_width_80() {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(1),
    ]);
    let areas = layout.split(Rect::new(0, 0, 80, 24));
    println!("=== Width 80, Height 24 ===");
    for (i, area) in areas.iter().enumerate() {
        println!("area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    let body_layout = Layout::horizontal([
        Constraint::Length(16),
        Constraint::Min(10),
    ]);
    let body_areas = body_layout.split(areas[2]);
    for (i, area) in body_areas.iter().enumerate() {
        println!("body_area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
    
    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(2),
    ]);
    let main_areas = main_layout.split(body_areas[1]);
    for (i, area) in main_areas.iter().enumerate() {
        println!("main_area {}: x={}, y={}, w={}, h={}", i, area.x, area.y, area.width, area.height);
    }
}

#[test]
fn test_dashboard_layout_at_various_heights() {
    for height in [24, 26, 28, 30, 32] {
        println!("\n=== Height {} ===", height);
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
        ]);
        let areas = layout.split(Rect::new(0, 0, 80, height));
        let body_layout = Layout::horizontal([
            Constraint::Length(16),
            Constraint::Min(10),
        ]);
        let body_areas = body_layout.split(areas[2]);
        let main_layout = Layout::vertical([
            Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
            Constraint::Length(1), Constraint::Length(4), Constraint::Length(1),
            Constraint::Length(2), Constraint::Length(1), Constraint::Length(3),
            Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
            Constraint::Length(3), Constraint::Length(1), Constraint::Length(2),
        ]);
        let main_areas = main_layout.split(body_areas[1]);
        for (i, area) in main_areas.iter().enumerate() {
            println!("main_area {}: h={}", i, area.height);
        }
    }
}
