use photon_ui::layout::Constraint;

#[test]
fn constraint_from_lengths_vec() {
    let c = Constraint::from_lengths(vec![1, 2, 3]);
    assert_eq!(
        c,
        vec![
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3)
        ]
    );
}

#[test]
fn constraint_from_ratios_vec() {
    let c = Constraint::from_ratios(vec![(1, 2), (1, 3)]);
    assert_eq!(c, vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)]);
}

#[test]
fn constraint_from_percentages_vec() {
    let c = Constraint::from_percentages(vec![25, 50, 25]);
    assert_eq!(
        c,
        vec![
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25)
        ]
    );
}

#[test]
fn constraint_all_variants_display() {
    let variants = vec![
        Constraint::Min(1),
        Constraint::Max(2),
        Constraint::Length(3),
        Constraint::Percentage(50),
        Constraint::Ratio(1, 4),
        Constraint::Fill(1),
    ];
    let displays: Vec<String> = variants.iter().map(|c| c.to_string()).collect();
    assert_eq!(
        displays,
        vec![
            "Min(1)",
            "Max(2)",
            "Length(3)",
            "Percentage(50)",
            "Ratio(1, 4)",
            "Fill(1)",
        ]
    );
}

#[test]
fn constraint_from_u16_is_length() {
    let c: Constraint = 42.into();
    assert_eq!(c, Constraint::Length(42));
}

#[test]
fn constraint_as_ref_returns_self() {
    let c = Constraint::Fill(3);
    assert_eq!(c.as_ref(), &Constraint::Fill(3));
}
