//! UI widgets module
//!
//! Reusable widget components for the TUI.

pub mod action_menu;
pub mod clone_dialog;
pub mod dir_chooser;
pub mod help_panel;
pub mod main_dir_manager;
pub mod move_target_selector;
pub mod path_bar;
pub mod repo_list;
pub mod search_box;
pub mod status_bar;
pub mod theme_selector;
pub mod title_bar;
pub mod toast;

pub use action_menu::{centered_popup, ActionMenu};
pub use clone_dialog::{clone_dialog_rect, CloneDialog};
pub use dir_chooser::{centered_rect, DirectoryChooser, DirectoryChooserState};
pub use help_panel::{centered_help_popup, HelpPanel};
pub use main_dir_manager::{centered_rect as main_dir_centered_rect, MainDirManager};
pub use move_target_selector::{move_target_centered_rect, MoveTargetSelector, MoveTargetSelectorState};
pub use path_bar::PathBar;
pub use repo_list::RepoList;
pub use search_box::SearchBox;
pub use status_bar::StatusBar;
pub use theme_selector::ThemeSelector;
pub use title_bar::TitleBar;
pub use toast::{calculate_toast_rect, ToastManager, ToastWidget};
