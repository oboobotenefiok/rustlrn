// File: src/state.rs
//! Application state management for RustLrn
//!
//! This module handles the application state, including lesson navigation,
//! code block editing, warning tracking, and user progress persistence.

use crate::error::{Result, RustlrnError};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::time::{Duration, SystemTime};

/// Type-safe wrapper for lesson identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonId(usize);

impl LessonId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    
    pub fn as_usize(&self) -> usize {
        self.0
    }
    
    pub fn next(&self) -> Option<Self> {
        Some(Self(self.0 + 1))
    }
    
    pub fn previous(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }
}

impl From<usize> for LessonId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

/// Type-safe wrapper for code block identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(usize);

impl BlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for BlockId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

/// Composite key for tracking edited code blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodeBlockKey {
    pub lesson_id: LessonId,
    pub block_id: BlockId,
}

impl CodeBlockKey {
    pub fn new(lesson_id: LessonId, block_id: BlockId) -> Self {
        Self { lesson_id, block_id }
    }
}

/// User progress data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    /// Last accessed lesson
    pub last_lesson: LessonId,
    
    /// Completed lessons
    pub completed_lessons: Vec<LessonId>,
    
    /// Code edits made by the user
    pub edited_blocks: HashMap<CodeBlockKey, String>,
    
    /// Timestamp of last activity
    pub last_activity: SystemTime,
    
    /// Total warnings encountered
    pub total_warnings: usize,
    
    /// Time spent per lesson (in seconds)
    pub time_spent: HashMap<LessonId, Duration>,
}

impl Default for UserProgress {
    fn default() -> Self {
        Self {
            last_lesson: LessonId::new(0),
            completed_lessons: Vec::new(),
            edited_blocks: HashMap::new(),
            last_activity: SystemTime::now(),
            total_warnings: 0,
            time_spent: HashMap::new(),
        }
    }
}

/// Application state manager
pub struct AppState {
    /// Current lesson being viewed
    current_lesson: LessonId,
    
    /// Warning count for the current session
    warn_count: usize,
    
    /// Maximum warnings before showing a hint
    max_warnings: usize,
    
    /// Edited code blocks
    edited_blocks: HashMap<CodeBlockKey, String>,
    
    /// Original code blocks (for reset functionality)
    original_blocks: HashMap<CodeBlockKey, String>,
    
    /// Application configuration
    config: Config,
    
    /// User progress (persisted)
    progress: UserProgress,
    
    /// Total number of lessons available
    total_lessons: usize,
}

impl AppState {
    /// Create a new application state
    pub fn new(start_lesson: usize, total_lessons: usize, config: Config) -> Result<Self> {
        let lesson_id = LessonId::new(start_lesson);
        
        if lesson_id.as_usize() >= total_lessons {
            return Err(RustlrnError::State(format!(
                "Lesson {} is out of range (max: {})",
                lesson_id.as_usize(),
                total_lessons
            )));
        }
        
        let progress = Self::load_progress()?;
        
        Ok(Self {
            current_lesson: lesson_id,
            warn_count: 0,
            max_warnings: 3,
            edited_blocks: HashMap::new(),
            original_blocks: HashMap::new(),
            config,
            progress,
            total_lessons,
        })
    }
    
    /// Load progress from disk
    fn load_progress() -> Result<UserProgress> {
        let progress_path = Self::progress_path();
        
        if !progress_path.exists() {
            return Ok(UserProgress::default());
        }
        
        let content = fs::read_to_string(&progress_path)
            .map_err(|e| RustlrnError::Io(e))?;
        
        if content.is_empty() {
            return Ok(UserProgress::default());
        }
        
        serde_json::from_str(&content)
            .map_err(|e| RustlrnError::Config(format!("Failed to parse progress: {}", e)))
    }
    
    /// Save progress to disk
    pub fn save_progress(&self) -> Result<()> {
        let progress_path = Self::progress_path();
        
        if let Some(parent) = progress_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| RustlrnError::Io(e))?;
            }
        }
        
        let content = serde_json::to_string_pretty(&self.progress)
            .map_err(|e| RustlrnError::Config(format!("Failed to serialize progress: {}", e)))?;
        
        fs::write(&progress_path, content)
            .map_err(|e| RustlrnError::Io(e))
    }
    
    /// Get the progress file path
    fn progress_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".rustlrn").join("progress.json")
    }
    
    /// Get the current lesson ID
    pub fn current_lesson(&self) -> LessonId {
        self.current_lesson
    }
    
    /// Navigate to the next lesson
    pub fn navigate_next(&mut self) -> Result<bool> {
        if let Some(next) = self.current_lesson.next() {
            if next.as_usize() < self.total_lessons {
                self.current_lesson = next;
                self.progress.last_lesson = next;
                self.progress.last_activity = SystemTime::now();
                self.reset_warning();
                return Ok(true);
            }
        }
        Err(RustlrnError::State("Already at the last lesson".to_string()))
    }
    
    /// Navigate to the previous lesson
    pub fn navigate_previous(&mut self) -> Result<bool> {
        if let Some(prev) = self.current_lesson.previous() {
            self.current_lesson = prev;
            self.progress.last_lesson = prev;
            self.progress.last_activity = SystemTime::now();
            self.reset_warning();
            return Ok(true);
        }
        Err(RustlrnError::State("Already at the first lesson".to_string()))
    }
    
    /// Navigate to a specific lesson
    pub fn navigate_to(&mut self, lesson_id: LessonId) -> Result<()> {
        if lesson_id.as_usize() >= self.total_lessons {
            return Err(RustlrnError::State(format!(
                "Lesson {} is out of range (max: {})",
                lesson_id.as_usize(),
                self.total_lessons
            )));
        }
        self.current_lesson = lesson_id;
        self.progress.last_lesson = lesson_id;
        self.progress.last_activity = SystemTime::now();
        self.reset_warning();
        Ok(())
    }
    
    /// Mark the current lesson as completed
    pub fn mark_current_completed(&mut self) {
        if !self.progress.completed_lessons.contains(&self.current_lesson) {
            self.progress.completed_lessons.push(self.current_lesson);
        }
        self.progress.last_activity = SystemTime::now();
    }
    
    /// Check if a lesson is completed
    pub fn is_lesson_completed(&self, lesson_id: LessonId) -> bool {
        self.progress.completed_lessons.contains(&lesson_id)
    }
    
    /// Get the progress percentage
    pub fn progress_percentage(&self) -> f32 {
        if self.total_lessons == 0 {
            0.0
        } else {
            (self.progress.completed_lessons.len() as f32 / self.total_lessons as f32) * 100.0
        }
    }
    
    /// Increment warning count
    pub fn increment_warning(&mut self) -> bool {
        self.warn_count += 1;
        self.progress.total_warnings += 1;
        self.warn_count >= self.max_warnings
    }
    
    /// Reset warning count
    pub fn reset_warning(&mut self) {
        self.warn_count = 0;
    }
    
    /// Check if warnings are at threshold
    pub fn has_warning(&self) -> bool {
        self.warn_count > 0
    }
    
    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warn_count
    }
    
    /// Get a code block (either original or edited)
    pub fn get_code_block(&self, lesson_id: LessonId, block_id: BlockId, original: &str) -> String {
        let key = CodeBlockKey::new(lesson_id, block_id);
        
        // Store original if not already stored
        if !self.original_blocks.contains_key(&key) {
            // We can't modify self here, so this is handled in update_code_block
        }
        
        self.edited_blocks
            .get(&key)
            .cloned()
            .unwrap_or_else(|| original.to_string())
    }
    
    /// Get the original code block
    pub fn get_original_block(&self, lesson_id: LessonId, block_id: BlockId) -> Option<&String> {
        let key = CodeBlockKey::new(lesson_id, block_id);
        self.original_blocks.get(&key)
    }
    
    /// Update a code block
    pub fn update_code_block(&mut self, lesson_id: LessonId, block_id: BlockId, code: String, original: &str) {
        let key = CodeBlockKey::new(lesson_id, block_id);
        
        // Store original if not already stored
        if !self.original_blocks.contains_key(&key) {
            self.original_blocks.insert(key.clone(), original.to_string());
        }
        
        self.edited_blocks.insert(key, code);
        self.progress.edited_blocks = self.edited_blocks.clone();
        self.progress.last_activity = SystemTime::now();
    }
    
    /// Reset a code block to its original state
    pub fn reset_code_block(&mut self, lesson_id: LessonId, block_id: BlockId) -> bool {
        let key = CodeBlockKey::new(lesson_id, block_id);
        let removed = self.edited_blocks.remove(&key);
        self.progress.edited_blocks = self.edited_blocks.clone();
        if removed.is_some() {
            self.progress.last_activity = SystemTime::now();
            true
        } else {
            false
        }
    }
    
    /// Check if a code block has been edited
    pub fn is_block_edited(&self, lesson_id: LessonId, block_id: BlockId) -> bool {
        let key = CodeBlockKey::new(lesson_id, block_id);
        self.edited_blocks.contains_key(&key)
    }
    
    /// Get all edited blocks for a lesson
    pub fn get_edited_blocks_for_lesson(&self, lesson_id: LessonId) -> HashMap<BlockId, String> {
        self.edited_blocks
            .iter()
            .filter_map(|(key, code)| {
                if key.lesson_id == lesson_id {
                    Some((key.block_id, code.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }
    
    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Track time spent on lesson
    pub fn track_time(&mut self, lesson_id: LessonId, duration: Duration) {
        let entry = self.progress.time_spent.entry(lesson_id).or_insert(Duration::from_secs(0));
        *entry += duration;
    }
    
    /// Get time spent on a lesson
    pub fn time_spent(&self, lesson_id: LessonId) -> Duration {
        self.progress.time_spent.get(&lesson_id).cloned().unwrap_or(Duration::from_secs(0))
    }
    
    /// Get total warnings
    pub fn total_warnings(&self) -> usize {
        self.progress.total_warnings
    }
    
    /// Get last activity time
    pub fn last_activity(&self) -> SystemTime {
        self.progress.last_activity
    }
    
    /// Get total lessons
    pub fn total_lessons(&self) -> usize {
        self.total_lessons
    }
    
    /// Get completed lessons count
    pub fn completed_count(&self) -> usize {
        self.progress.completed_lessons.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lesson_navigation() {
        let config = Config::default();
        let mut state = AppState::new(0, 5, config).unwrap();
        
        assert_eq!(state.current_lesson().as_usize(), 0);
        
        assert!(state.navigate_next().is_ok());
        assert_eq!(state.current_lesson().as_usize(), 1);
        
        assert!(state.navigate_previous().is_ok());
        assert_eq!(state.current_lesson().as_usize(), 0);
        
        // Navigate to last lesson
        state.navigate_to(LessonId::new(4)).unwrap();
        assert_eq!(state.current_lesson().as_usize(), 4);
        
        // Cannot navigate past last
        assert!(state.navigate_next().is_err());
    }
    
    #[test]
    fn test_warning_tracking() {
        let config = Config::default();
        let mut state = AppState::new(0, 5, config).unwrap();
        
        assert!(!state.has_warning());
        assert_eq!(state.warning_count(), 0);
        
        state.increment_warning();
        assert!(state.has_warning());
        assert_eq!(state.warning_count(), 1);
        
        state.reset_warning();
        assert!(!state.has_warning());
        assert_eq!(state.warning_count(), 0);
    }
    
    #[test]
    fn test_code_block_editing() {
        let config = Config::default();
        let mut state = AppState::new(0, 5, config).unwrap();
        let lesson = LessonId::new(0);
        let block = BlockId::new(0);
        let original = "fn main() { println!(\"Hello\"); }";
        let edited = "fn main() { println!(\"Modified\"); }";
        
        // Initially not edited
        assert!(!state.is_block_edited(lesson, block));
        assert_eq!(state.get_code_block(lesson, block, original), original);
        
        // Update block
        state.update_code_block(lesson, block, edited.to_string(), original);
        assert!(state.is_block_edited(lesson, block));
        assert_eq!(state.get_code_block(lesson, block, original), edited);
        
        // Reset block
        assert!(state.reset_code_block(lesson, block));
        assert!(!state.is_block_edited(lesson, block));
        assert_eq!(state.get_code_block(lesson, block, original), original);
        
        // Reset non-existent block
        assert!(!state.reset_code_block(lesson, BlockId::new(99)));
    }
    
    #[test]
    fn test_progress_tracking() {
        let config = Config::default();
        let mut state = AppState::new(0, 5, config).unwrap();
        
        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.progress_percentage(), 0.0);
        
        state.mark_current_completed();
        assert_eq!(state.completed_count(), 1);
        assert_eq!(state.progress_percentage(), 20.0);
        
        // Mark same lesson completed again should not duplicate
        state.mark_current_completed();
        assert_eq!(state.completed_count(), 1);
        
        // Navigate and complete another lesson
        state.navigate_next().unwrap();
        state.mark_current_completed();
        assert_eq!(state.completed_count(), 2);
        assert_eq!(state.progress_percentage(), 40.0);
    }
}	
