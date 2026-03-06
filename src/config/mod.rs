//! Configuration management module

pub mod load;
pub mod save;
pub mod types;
pub mod validators;

pub use load::{load_config, load_or_create_config, save_config};
pub use types::{Config, EditorConfig, UiConfig};
pub use validators::{validate_config, validate_directory, validate_editor_command};
