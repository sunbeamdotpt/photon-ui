//! UI components provided by photon-ui.
//!
//! Each submodule implements a specific [`Component`] trait for rendering
//! and interacting with a distinct UI element.

/// A box that draws a border around its content.
pub mod box_component;
/// Hierarchical breadcrumb navigation.
pub mod breadcrumbs;
/// Clickable button.
pub mod button;
/// Loading spinner that can be cancelled.
pub mod cancellable_loader;
/// Generic container component.
pub mod container;
/// Layout container that splits space among children.
pub mod div;
/// Horizontal or vertical divider line.
pub mod divider;
/// Multi-line text editor with vim/emacs modes.
pub mod editor;
/// Page header component.
pub mod header;
/// Terminal image display widget.
pub mod image_widget;
/// Single-line text input with vim/emacs modes.
pub mod input;
/// Loading spinner.
pub mod loader;
/// Markdown renderer.
pub mod markdown;
/// Modal dialog overlay.
pub mod modal;
/// Panel container with optional border.
pub mod panel;
/// Progress bar.
pub mod progress_bar;
/// Selectable list of items.
pub mod select_list;
/// Settings key/value list.
pub mod settings_list;
/// Sidebar navigation.
pub mod sidebar;
/// Empty spacer for layout padding.
pub mod spacer;
/// Status bar with segments.
pub mod status_bar;
/// Table with sortable columns and row selection.
pub mod table;
/// Tab bar.
pub mod tabs;
/// Static text label.
pub mod text;
/// Collapsible tree view.
pub mod tree_view;
/// Text truncated to fit a width.
pub mod truncated_text;

pub use box_component::Box;
pub use breadcrumbs::Breadcrumbs;
pub use button::Button;
pub use cancellable_loader::CancellableLoader;
pub use container::Container;
pub use div::Div;
pub use divider::Divider;
pub use editor::Editor;
pub use header::Header;
pub use image_widget::ImageWidget;
pub use input::Input;
pub use loader::Loader;
pub use markdown::Markdown;
pub use modal::Modal;
pub use panel::Panel;
pub use progress_bar::ProgressBar;
pub use select_list::SelectList;
pub use settings_list::SettingsList;
pub use sidebar::{
    Sidebar,
    SidebarItem,
};
pub use spacer::Spacer;
pub use status_bar::{
    Segment,
    StatusBar,
};
pub use table::{
    Column,
    Row,
    Table,
};
pub use tabs::Tabs;
pub use text::Text;
pub use tree_view::{
    TreeNode,
    TreeView,
};
pub use truncated_text::TruncatedText;
