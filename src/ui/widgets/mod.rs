//! UI widgets module
//!
//! Reusable widget components for the TUI.

pub mod action_menu;
pub mod dir_chooser;
pub mod help_panel;
pub mod repo_list;
pub mod search_box;

pub use action_menu::{centered_popup, ActionMenu};
pub use dir_chooser::{centered_rect, DirChooser};
pub use help_panel::{centered_help_popup, HelpPanel};
pub use repo_list::RepoList;
pub use search_box::SearchBox;
