//! Lesson loading and management module

use include_dir::{include_dir, Dir};

static LESSONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/lessons");

/// Load all lesson content as strings
pub fn load_all_lessons() -> Vec<String> {
    let mut lessons = Vec::new();

    let lesson_files = [
        "orientation.md",
        "beginner/setting.md",
        "intermediate/ownership.md",
    ];

    for file in lesson_files {
        if let Some(content) = load_lesson_content(file) {
            lessons.push(content);
        } else {
            eprintln!("Warning: Could not load lesson: {}", file);
        }
    }

    lessons
}

/// Load a single lesson file content
pub fn load_lesson_content(filename: &str) -> Option<String> {
    LESSONS_DIR
        .get_file(filename)
        .and_then(|file| file.contents_utf8().map(|s| s.to_string()))
}
