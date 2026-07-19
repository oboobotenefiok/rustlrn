//! Lesson loading and management module

use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::collections::HashMap;

static LESSONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/lessons");

#[derive(Debug, Deserialize, Clone)]
pub struct LessonMetadata {
    pub title: String,
    pub level: String,
    #[serde(default = "default_estimated_time")]
    pub estimated_time: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: u8,
}

fn default_estimated_time() -> String {
    "10 min".to_string()
}

fn default_author() -> String {
    "RustLRN Team".to_string()
}

fn default_difficulty() -> u8 {
    1
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub metadata: LessonMetadata,
    pub content: String,
    pub filename: String,
}

impl Lesson {
    pub fn from_markdown(content: &str, filename: &str) -> Option<Self> {
        let mut lines = content.lines();
        let mut frontmatter = String::new();
        let mut in_frontmatter = false;
        let mut content_start = 0;

        for (idx, line) in lines.clone().enumerate() {
            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else {
                    in_frontmatter = false;
                    content_start = idx + 1;
                    break;
                }
            }
            if in_frontmatter {
                frontmatter.push_str(line);
                frontmatter.push('\n');
            }
        }

        let metadata: LessonMetadata = if frontmatter.is_empty() {
            LessonMetadata {
                title: filename.replace(".md", "").replace('_', " "),
                level: "beginner".to_string(),
                estimated_time: default_estimated_time(),
                prerequisites: Vec::new(),
                tags: Vec::new(),
                author: default_author(),
                difficulty: default_difficulty(),
            }
        } else {
            match serde_yaml::from_str(&frontmatter) {
                Ok(meta) => meta,
                Err(_) => {
                    eprintln!("Warning: Failed to parse frontmatter in {}", filename);
                    LessonMetadata {
                        title: filename.replace(".md", "").replace('_', " "),
                        level: "beginner".to_string(),
                        estimated_time: default_estimated_time(),
                        prerequisites: Vec::new(),
                        tags: Vec::new(),
                        author: default_author(),
                        difficulty: default_difficulty(),
                    }
                }
            }
        };

        let content_body: String = if content_start > 0 {
            lines.skip(content_start).collect::<Vec<&str>>().join("\n")
        } else {
            content.to_string()
        };

        Some(Lesson {
            metadata,
            content: content_body,
            filename: filename.to_string(),
        })
    }
}

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

pub fn load_lesson_content(filename: &str) -> Option<String> {
    if let Some(file) = LESSONS_DIR.get_file(filename) {
        if let Some(content) = file.contents_utf8() {
            return Some(content.to_string());
        }
    }
    None
}

pub fn load_lesson_with_metadata(filename: &str) -> Option<Lesson> {
    if let Some(content) = load_lesson_content(filename) {
        return Lesson::from_markdown(&content, filename);
    }
    None
}

pub fn get_all_lessons_with_metadata() -> Vec<Lesson> {
    let mut lessons = Vec::new();
    let lesson_files = [
        "orientation.md",
        "beginner/setting.md",
        "intermediate/ownership.md",
    ];

    for file in lesson_files {
        if let Some(lesson) = load_lesson_with_metadata(file) {
            lessons.push(lesson);
        }
    }

    lessons
}

pub fn get_lesson_by_title(title: &str) -> Option<Lesson> {
    for lesson in get_all_lessons_with_metadata() {
        if lesson.metadata.title.to_lowercase().contains(&title.to_lowercase()) {
            return Some(lesson);
        }
    }
    None
}

pub fn get_lessons_by_level(level: &str) -> Vec<Lesson> {
    let mut result = Vec::new();
    for lesson in get_all_lessons_with_metadata() {
        if lesson.metadata.level.to_lowercase() == level.to_lowercase() {
            result.push(lesson);
        }
    }
    result
}

pub fn get_lessons_by_tag(tag: &str) -> Vec<Lesson> {
    let mut result = Vec::new();
    for lesson in get_all_lessons_with_metadata() {
        if lesson.metadata.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()) {
            result.push(lesson);
        }
    }
    result
}

pub fn get_lesson_count() -> usize {
    load_all_lessons().len()
}

pub fn get_lesson_titles() -> Vec<String> {
    get_all_lessons_with_metadata()
        .iter()
        .map(|l| l.metadata.title.clone())
        .collect()
}
