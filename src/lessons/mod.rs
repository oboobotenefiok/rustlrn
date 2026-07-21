// File: src/lessons.rs
//! Lesson loading and management module with lazy loading and caching
//!
//! This module handles loading lesson content from embedded files,
//! caching for performance, and providing lesson metadata.

use crate::error::{Result, RustlrnError};
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, Duration};

/// Static lesson directory embedded at compile time
static LESSONS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/lessons");

/// Global lesson cache for lazy loading
static LESSON_CACHE: OnceLock<LessonCache> = OnceLock::new();

/// Cache for loaded lessons
#[derive(Debug, Clone)]
struct LessonCache {
    lessons: Vec<Lesson>,
    loaded_time: SystemTime,
    last_access: SystemTime,
}

/// Individual lesson with metadata
#[derive(Debug, Clone)]
pub struct Lesson {
    /// Lesson content as string
    pub content: String,
    /// Lesson title
    pub title: String,
    /// Lesson number (1-indexed)
    pub number: usize,
    /// Path to the lesson file
    pub path: String,
    /// Number of code blocks in the lesson
    pub code_block_count: usize,
    /// Whether the lesson contains an executable main function
    pub has_main: bool,
    /// Tags for the lesson
    pub tags: Vec<String>,
    /// Difficulty level (1-5)
    pub difficulty: u8,
    /// Estimated time to complete in minutes
    pub estimated_time: u8,
}

impl Lesson {
    /// Create a new lesson from content and metadata
    fn new(content: String, path: String, number: usize) -> Self {
        let title = content.lines()
            .find(|line| line.starts_with("# ") || line.starts_with("## "))
            .map(|line| line.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| format!("Lesson {}", number));
        
        let code_blocks = crate::executor::extract_code_blocks(&content);
        let code_block_count = code_blocks.len();
        let has_main = code_blocks.iter().any(|block| 
            crate::executor::has_main_function(block)
        );
        
        // Extract tags from content
        let tags = Self::extract_tags(&content);
        
        // Extract difficulty
        let difficulty = Self::extract_difficulty(&content);
        
        // Extract estimated time
        let estimated_time = Self::extract_estimated_time(&content);
        
        Self {
            content,
            title,
            number,
            path,
            code_block_count,
            has_main,
            tags,
            difficulty,
            estimated_time,
        }
    }
    
    /// Extract tags from lesson content
    fn extract_tags(content: &str) -> Vec<String> {
        let mut tags = Vec::new();
        for line in content.lines() {
            if line.starts_with("Tags:") || line.starts_with("tags:") {
                let tag_str = line.trim_start_matches("Tags:").trim_start_matches("tags:").trim();
                tags = tag_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                break;
            }
        }
        tags
    }
    
    /// Extract difficulty from lesson content
    fn extract_difficulty(content: &str) -> u8 {
        for line in content.lines() {
            if line.starts_with("Difficulty:") || line.starts_with("difficulty:") {
                let diff_str = line.trim_start_matches("Difficulty:").trim_start_matches("difficulty:").trim();
                if let Ok(diff) = diff_str.parse::<u8>() {
                    return diff.clamp(1, 5);
                }
                break;
            }
        }
        1 // Default difficulty
    }
    
    /// Extract estimated time from lesson content
    fn extract_estimated_time(content: &str) -> u8 {
        for line in content.lines() {
            if line.starts_with("Time:") || line.starts_with("time:") {
                let time_str = line.trim_start_matches("Time:").trim_start_matches("time:").trim();
                if let Ok(time) = time_str.parse::<u8>() {
                    return time.clamp(1, 60);
                }
                break;
            }
        }
        5 // Default time in minutes
    }
}

/// Lesson manager with lazy loading and caching
#[derive(Debug, Clone)]
pub struct LessonManager {
    cache: LessonCache,
}

impl LessonManager {
    /// Create a new lesson manager
    pub fn new() -> Result<Self> {
        let lessons = Self::load_lessons()?;
        let now = SystemTime::now();
        
        Ok(Self {
            cache: LessonCache {
                lessons,
                loaded_time: now,
                last_access: now,
            },
        })
    }
    
    /// Load all lessons from the embedded directory
    fn load_lessons() -> Result<Vec<Lesson>> {
        let lesson_files = [
            "orientation.md",
            "beginner/setting.md",
            "intermediate/ownership.md",
        ];
        
        let mut lessons = Vec::new();
        
        for (i, file) in lesson_files.iter().enumerate() {
            let content = Self::load_lesson_file(file)?;
            let lesson = Lesson::new(content, file.to_string(), i + 1);
            lessons.push(lesson);
        }
        
        // If no lessons found, try to discover from directory
        if lessons.is_empty() {
            lessons = Self::discover_lessons()?;
        }
        
        Ok(lessons)
    }
    
    /// Load a single lesson file
    fn load_lesson_file(filename: &str) -> Result<String> {
        LESSONS_DIR
            .get_file(filename)
            .and_then(|file| file.contents_utf8())
            .map(|s| s.to_string())
            .ok_or_else(|| RustlrnError::Lesson(format!("Lesson not found: {}", filename)))
    }
    
    /// Discover lessons from the embedded directory
    fn discover_lessons() -> Result<Vec<Lesson>> {
        let mut lessons = Vec::new();
        let mut files: Vec<_> = LESSONS_DIR
            .files()
            .filter(|f| {
                f.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "md" || ext == "rs")
                    .unwrap_or(false)
            })
            .collect();
        
        files.sort_by_key(|f| f.path());
        
        for file in files {
            if let Some(content) = file.contents_utf8() {
                let path = file.path().to_string_lossy().to_string();
                let number = lessons.len() + 1;
                let lesson = Lesson::new(content.to_string(), path, number);
                lessons.push(lesson);
            }
        }
        
        Ok(lessons)
    }
    
    /// Get a lesson by number (1-indexed)
    pub fn get_lesson(&mut self, number: usize) -> Result<&Lesson> {
        self.cache.last_access = SystemTime::now();
        
        if number == 0 || number > self.cache.lessons.len() {
            return Err(RustlrnError::Lesson(format!(
                "Lesson {} not found (1-{})",
                number,
                self.cache.lessons.len()
            )));
        }
        
        Ok(&self.cache.lessons[number - 1])
    }
    
    /// Get a lesson by index (0-indexed)
    pub fn get_lesson_by_index(&mut self, index: usize) -> Result<&Lesson> {
        self.cache.last_access = SystemTime::now();
        
        if index >= self.cache.lessons.len() {
            return Err(RustlrnError::Lesson(format!(
                "Lesson index {} out of range (0-{})",
                index,
                self.cache.lessons.len() - 1
            )));
        }
        
        Ok(&self.cache.lessons[index])
    }
    
    /// Get all lessons
    pub fn get_all_lessons(&mut self) -> &[Lesson] {
        self.cache.last_access = SystemTime::now();
        &self.cache.lessons
    }
    
    /// Get the number of lessons
    pub fn lesson_count(&self) -> usize {
        self.cache.lessons.len()
    }
    
    /// Search lessons by tag
    pub fn search_by_tag(&mut self, tag: &str) -> Vec<&Lesson> {
        self.cache.last_access = SystemTime::now();
        self.cache.lessons
            .iter()
            .filter(|lesson| lesson.tags.iter().any(|t| t.contains(tag)))
            .collect()
    }
    
    /// Search lessons by title
    pub fn search_by_title(&mut self, query: &str) -> Vec<&Lesson> {
        self.cache.last_access = SystemTime::now();
        let query = query.to_lowercase();
        self.cache.lessons
            .iter()
            .filter(|lesson| lesson.title.to_lowercase().contains(&query))
            .collect()
    }
    
    /// Get lessons by difficulty
    pub fn get_by_difficulty(&mut self, difficulty: u8) -> Vec<&Lesson> {
        self.cache.last_access = SystemTime::now();
        self.cache.lessons
            .iter()
            .filter(|lesson| lesson.difficulty == difficulty)
            .collect()
    }
    
    /// Get lessons with executable code
    pub fn get_executable_lessons(&mut self) -> Vec<&Lesson> {
        self.cache.last_access = SystemTime::now();
        self.cache.lessons
            .iter()
            .filter(|lesson| lesson.has_main)
            .collect()
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let now = SystemTime::now();
        let age = self.cache.loaded_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0));
        let last_access_age = self.cache.last_access
            .elapsed()
            .unwrap_or(Duration::from_secs(0));
        
        CacheStats {
            lesson_count: self.cache.lessons.len(),
            cache_age: age,
            last_access_age: last_access_age,
            total_code_blocks: self.cache.lessons.iter().map(|l| l.code_block_count).sum(),
            executable_lessons: self.cache.lessons.iter().filter(|l| l.has_main).count(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub lesson_count: usize,
    pub cache_age: Duration,
    pub last_access_age: Duration,
    pub total_code_blocks: usize,
    pub executable_lessons: usize,
}

/// Load all lessons (legacy function for backward compatibility)
pub fn load_all_lessons() -> Result<Vec<String>> {
    let manager = LessonManager::new()?;
    Ok(manager.get_all_lessons()
        .iter()
        .map(|lesson| lesson.content.clone())
        .collect())
}

/// Load a single lesson (legacy function)
pub fn load_lesson_content(filename: &str) -> Option<String> {
    LessonManager::load_lesson_file(filename).ok()
}

/// Get a lesson by number (convenience function)
pub fn get_lesson(number: usize) -> Result<Lesson> {
    let mut manager = LessonManager::new()?;
    manager.get_lesson(number).cloned()
}

/// Get all lessons with metadata
pub fn get_all_lessons_with_metadata() -> Result<Vec<Lesson>> {
    let manager = LessonManager::new()?;
    Ok(manager.get_all_lessons().to_vec())
}

/// Search lessons (convenience function)
pub fn search_lessons(query: &str) -> Result<Vec<Lesson>> {
    let mut manager = LessonManager::new()?;
    let results = if query.contains('@') {
        // Search by tag
        let tag = query.trim_start_matches('@');
        manager.search_by_tag(tag)
    } else {
        // Search by title
        manager.search_by_title(query)
    };
    Ok(results.iter().map(|l| (*l).clone()).collect())
}

/// Get lesson count
pub fn get_lesson_count() -> Result<usize> {
    let manager = LessonManager::new()?;
    Ok(manager.lesson_count())
}

/// Get cache statistics
pub fn get_cache_stats() -> Result<CacheStats> {
    let manager = LessonManager::new()?;
    Ok(manager.cache_stats())
}

/// Validate all lessons are valid
pub fn validate_lessons() -> Result<Vec<String>> {
    let manager = LessonManager::new()?;
    let mut errors = Vec::new();
    
    for lesson in manager.get_all_lessons() {
        // Check for empty content
        if lesson.content.trim().is_empty() {
            errors.push(format!("Lesson {} has empty content", lesson.number));
        }
        
        // Check for executable blocks that might be incomplete
        if lesson.has_main {
            let blocks = crate::executor::extract_code_blocks(&lesson.content);
            if blocks.is_empty() {
                errors.push(format!("Lesson {} has main but no code blocks", lesson.number));
            }
        }
    }
    
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lesson_manager_creation() {
        let manager = LessonManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(manager.lesson_count() > 0);
    }
    
    #[test]
    fn test_get_lesson() {
        let mut manager = LessonManager::new().unwrap();
        let lesson = manager.get_lesson(1);
        assert!(lesson.is_ok());
        
        let lesson = lesson.unwrap();
        assert!(!lesson.content.is_empty());
        assert!(!lesson.title.is_empty());
    }
    
    #[test]
    fn test_get_lesson_out_of_range() {
        let mut manager = LessonManager::new().unwrap();
        let lesson = manager.get_lesson(99);
        assert!(lesson.is_err());
    }
    
    #[test]
    fn test_lesson_metadata_extraction() {
        let content = r#"
# Test Lesson

Tags: rust, beginner
Difficulty: 2
Time: 10

```rust
fn main() {
    println!("Hello");
}
