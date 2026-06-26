//! Breadcrumbs component.
//!
//! Renders a trail of navigational items separated by a configurable delimiter.

use crate::{
    Component,
    RenderError,
    Rendered,
    theme::{
        Style,
        Theme,
        stylize,
    },
};

/// A non-interactive breadcrumbs trail.
///
/// Renders as `Home > Settings > Account` where the last item is bold and
/// uses the primary text color, earlier items use the secondary text color,
/// and the separator uses the default border color.
pub struct Breadcrumbs {
    items: Vec<String>,
    separator: String,
}

impl Breadcrumbs {
    /// Create a new breadcrumbs component with the given items.
    pub fn new(items: Vec<impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            separator: " > ".to_string(),
        }
    }

    /// Set a custom separator (default is `" > "`).
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl Component for Breadcrumbs {
    fn render(&self, _width: u16) -> Result<Rendered, RenderError> {
        let theme = Theme::palette();
        let primary_style = Style::new().fg(theme.text()).bold();
        let secondary_style = Style::new().fg(theme.text_muted());
        let separator_style = Style::new().fg(theme.border());

        let mut parts: Vec<String> = Vec::new();
        let count = self.items.len();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                parts.push(stylize(&self.separator, &separator_style));
            }

            let styled = if i == count.saturating_sub(1) {
                stylize(item, &primary_style)
            } else {
                stylize(item, &secondary_style)
            };
            parts.push(styled);
        }

        let line = parts.concat();

        Ok(Rendered {
            lines: vec![line],
            cursor: None,
            images: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn renders_single_item_bold() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["Home"]);
            let rendered = bc.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            let line = &rendered.lines[0];
            assert!(line.contains("Home"));
            assert!(line.contains("\x1b[1m"));
        });
    }

    #[test]
    fn renders_multiple_items_with_separator() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["Home", "Settings", "Account"]);
            let rendered = bc.render(80).unwrap();
            let line = &rendered.lines[0];

            assert!(line.contains("Home"));
            assert!(line.contains("Settings"));
            assert!(line.contains("Account"));
            assert!(line.contains(" > "));
        });
    }

    #[test]
    fn custom_separator() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["A", "B"]).separator(" / ");
            let rendered = bc.render(80).unwrap();
            let line = &rendered.lines[0];

            assert!(line.contains(" / "));
            assert!(!line.contains(" > "));
        });
    }

    #[test]
    fn empty_items_renders_empty_line() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(Vec::<String>::new());
            let rendered = bc.render(80).unwrap();
            assert_eq!(rendered.lines.len(), 1);
            assert_eq!(rendered.lines[0], "");
        });
    }

    #[test]
    fn uses_primary_color_for_last_item() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["Home", "Settings"]);
            let rendered = bc.render(80).unwrap();
            let line = &rendered.lines[0];

            // Light theme text_primary is SUNBEAM_BLACK (#1f1f1f = 31,31,31)
            assert!(line.contains("\x1b[38;2;31;31;31m"));
        });
    }

    #[test]
    fn uses_secondary_color_for_earlier_items() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["Home", "Settings"]);
            let rendered = bc.render(80).unwrap();
            let line = &rendered.lines[0];

            // Light theme text_secondary is #666666 = 102,102,102
            assert!(line.contains("\x1b[38;2;102;102;102m"));
        });
    }

    #[test]
    fn last_item_is_bold() {
        Theme::with(Theme::Light, || {
            let bc = Breadcrumbs::new(vec!["Home", "Settings"]);
            let rendered = bc.render(80).unwrap();
            let line = &rendered.lines[0];

            // Bold escape sequence
            assert!(line.contains("\x1b[1m"));
        });
    }
}
