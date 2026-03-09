//! Integration tests for favorites feature

use repotui::config::{self, Config};
use repotui::favorites::FavoritesStore;
use std::path::PathBuf;

#[test]
fn test_favorites_persistence() {
    // Create a test config
    let mut config = Config::default();
    config
        .main_directories
        .push(config::types::MainDirectoryConfig {
            path: PathBuf::from("/tmp/test"),
            display_name: None,
            max_depth: None,
            enabled: true,
        });

    // Add some favorites
    config.favorites.repositories = vec![
        "/home/user/repo1".to_string(),
        "/home/user/repo2".to_string(),
    ];

    // Serialize and deserialize
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();

    assert_eq!(deserialized.favorites.repositories.len(), 2);
    assert_eq!(deserialized.favorites.repositories[0], "/home/user/repo1");
    assert_eq!(deserialized.favorites.repositories[1], "/home/user/repo2");
}

#[test]
fn test_favorites_store_conversion() {
    let mut store = FavoritesStore::new();
    store.add(&PathBuf::from("/home/user/repo1"));
    store.add(&PathBuf::from("/home/user/repo2"));

    // Convert to config
    let config_favorites = config::types::FavoritesConfig::from_store(&store);
    assert_eq!(config_favorites.repositories.len(), 2);

    // Convert back to store
    let restored_store = config_favorites.to_store();
    assert!(restored_store.contains(&PathBuf::from("/home/user/repo1")));
    assert!(restored_store.contains(&PathBuf::from("/home/user/repo2")));
}

#[test]
fn test_favorites_config_default() {
    let config = config::types::FavoritesConfig::default();
    assert!(config.repositories.is_empty());
}
