#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use directories::ProjectDirs;

pub fn get_project_dir() -> &'static ProjectDirs {
    static INSTANCE: OnceLock<ProjectDirs> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        ProjectDirs::from("io.github.samunatsu.mihomo-tui", "", "Mihomo TUI").unwrap()
    })
}

pub fn get_data_dir() -> &'static Path {
    static INSTANCE: OnceLock<&Path> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let dir = get_project_dir().data_dir();
        fs::create_dir_all(dir).unwrap();
        dir
    })
}

pub fn get_profiles_dir() -> &'static PathBuf {
    static INSTANCE: OnceLock<PathBuf> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let dir = get_data_dir().join("profiles");
        fs::create_dir_all(&dir).unwrap();
        dir
    })
}
