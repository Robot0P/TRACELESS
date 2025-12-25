//! Scheduled cleanup and custom rules module
//!
//! Provides scheduled task management and custom cleanup rule support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use chrono::{DateTime, Utc, Duration as ChronoDuration, Timelike, Datelike};
use std::path::PathBuf;
use regex::Regex;

/// Schedule frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleFrequency {
    /// Every N minutes
    Minutes(u32),
    /// Every N hours
    Hourly(u32),
    /// Daily at specific hour (0-23)
    Daily { hour: u32 },
    /// Weekly on specific day (0=Sunday, 6=Saturday) at specific hour
    Weekly { day: u32, hour: u32 },
    /// Monthly on specific day (1-28) at specific hour
    Monthly { day: u32, hour: u32 },
}

impl ScheduleFrequency {
    /// Calculate next run time from a given time
    pub fn next_run_from(&self, from: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            ScheduleFrequency::Minutes(n) => from + ChronoDuration::minutes(*n as i64),
            ScheduleFrequency::Hourly(n) => from + ChronoDuration::hours(*n as i64),
            ScheduleFrequency::Daily { hour } => {
                let mut next = from.date_naive().and_hms_opt(*hour, 0, 0).unwrap();
                if next <= from.naive_utc() {
                    next = next + ChronoDuration::days(1);
                }
                DateTime::from_naive_utc_and_offset(next, Utc)
            }
            ScheduleFrequency::Weekly { day, hour } => {
                let current_day = from.weekday().num_days_from_sunday();
                let days_until = if *day >= current_day {
                    *day - current_day
                } else {
                    7 - (current_day - *day)
                };
                let next_date = from.date_naive() + ChronoDuration::days(days_until as i64);
                let next = next_date.and_hms_opt(*hour, 0, 0).unwrap();
                if days_until == 0 && next <= from.naive_utc() {
                    DateTime::from_naive_utc_and_offset(next + ChronoDuration::days(7), Utc)
                } else {
                    DateTime::from_naive_utc_and_offset(next, Utc)
                }
            }
            ScheduleFrequency::Monthly { day, hour } => {
                let current_day = from.day();
                let next_day = (*day).min(28); // Limit to 28 for safety
                let next = if next_day > current_day || (next_day == current_day && from.hour() < *hour) {
                    from.date_naive()
                        .with_day(next_day)
                        .unwrap()
                        .and_hms_opt(*hour, 0, 0)
                        .unwrap()
                } else {
                    // Next month
                    let next_month = if from.month() == 12 {
                        from.date_naive()
                            .with_year(from.year() + 1)
                            .unwrap()
                            .with_month(1)
                            .unwrap()
                    } else {
                        from.date_naive().with_month(from.month() + 1).unwrap()
                    };
                    next_month
                        .with_day(next_day)
                        .unwrap()
                        .and_hms_opt(*hour, 0, 0)
                        .unwrap()
                };
                DateTime::from_naive_utc_and_offset(next, Utc)
            }
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            ScheduleFrequency::Minutes(n) => format!("Every {} minute(s)", n),
            ScheduleFrequency::Hourly(n) => format!("Every {} hour(s)", n),
            ScheduleFrequency::Daily { hour } => format!("Daily at {:02}:00", hour),
            ScheduleFrequency::Weekly { day, hour } => {
                let day_name = match day {
                    0 => "Sunday",
                    1 => "Monday",
                    2 => "Tuesday",
                    3 => "Wednesday",
                    4 => "Thursday",
                    5 => "Friday",
                    _ => "Saturday",
                };
                format!("Every {} at {:02}:00", day_name, hour)
            }
            ScheduleFrequency::Monthly { day, hour } => {
                format!("Monthly on day {} at {:02}:00", day, hour)
            }
        }
    }
}

/// Scheduled cleanup task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Whether the task is enabled
    pub enabled: bool,
    /// Schedule frequency
    pub frequency: ScheduleFrequency,
    /// Cleanup actions to perform
    pub actions: Vec<CleanupAction>,
    /// Last run time
    pub last_run: Option<DateTime<Utc>>,
    /// Next scheduled run time
    pub next_run: Option<DateTime<Utc>>,
    /// Last run result
    pub last_result: Option<TaskResult>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Custom rules to apply
    pub custom_rules: Vec<String>, // Rule IDs
}

impl ScheduledTask {
    /// Create a new scheduled task
    pub fn new(id: impl Into<String>, name: impl Into<String>, frequency: ScheduleFrequency) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            enabled: true,
            frequency,
            actions: Vec::new(),
            last_run: None,
            next_run: Some(frequency.next_run_from(now)),
            last_result: None,
            created_at: now,
            custom_rules: Vec::new(),
        }
    }

    /// Add an action
    pub fn with_action(mut self, action: CleanupAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add a custom rule
    pub fn with_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.custom_rules.push(rule_id.into());
        self
    }

    /// Check if task should run now
    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(next_run) = self.next_run {
            Utc::now() >= next_run
        } else {
            false
        }
    }

    /// Update after run
    pub fn mark_run(&mut self, result: TaskResult) {
        let now = Utc::now();
        self.last_run = Some(now);
        self.next_run = Some(self.frequency.next_run_from(now));
        self.last_result = Some(result);
    }
}

/// Cleanup action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupAction {
    /// Clean system logs
    SystemLogs,
    /// Clean memory
    Memory,
    /// Clean network traces
    Network,
    /// Clean registry/privacy traces
    Registry,
    /// Clean browser data
    BrowserData { browsers: Vec<String> },
    /// Clean specific paths
    Paths { paths: Vec<String>, recursive: bool },
    /// Apply custom rules
    CustomRules { rule_ids: Vec<String> },
    /// Clean temporary files
    TempFiles,
    /// Clean DNS cache
    DnsCache,
    /// Clean recent documents
    RecentDocuments,
    /// Clean thumbnail cache
    ThumbnailCache,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub items_cleaned: u64,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
    pub duration_ms: u64,
    pub completed_at: DateTime<Utc>,
}

/// Custom cleanup rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Unique rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Rule type
    pub rule_type: RuleType,
    /// Target patterns
    pub patterns: Vec<RulePattern>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Age threshold for files (in days)
    pub age_days: Option<u32>,
    /// Size threshold (in bytes)
    pub size_threshold: Option<u64>,
    /// Action to take
    pub action: RuleAction,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last modified
    pub modified_at: DateTime<Utc>,
}

/// Rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    /// Match files by extension
    FileExtension,
    /// Match files by name pattern (glob)
    FilePattern,
    /// Match files by regex
    Regex,
    /// Match directories
    Directory,
    /// Match by file content (basic)
    ContentMatch,
}

/// Rule pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePattern {
    /// Pattern string
    pub pattern: String,
    /// Whether pattern is case-sensitive
    pub case_sensitive: bool,
}

impl RulePattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            case_sensitive: false,
        }
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    /// Check if a path matches this pattern
    pub fn matches(&self, path: &str) -> bool {
        let pattern = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };
        let path = if self.case_sensitive {
            path.to_string()
        } else {
            path.to_lowercase()
        };

        // Simple glob matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }

        path.contains(&pattern)
    }
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Securely delete matching files
    SecureDelete { method: String },
    /// Move to trash
    MoveToTrash,
    /// Archive before delete
    Archive { destination: String },
    /// Just report (dry run)
    Report,
}

impl CustomRule {
    /// Create a new custom rule
    pub fn new(id: impl Into<String>, name: impl Into<String>, rule_type: RuleType) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            enabled: true,
            rule_type,
            patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            age_days: None,
            size_threshold: None,
            action: RuleAction::Report,
            created_at: now,
            modified_at: now,
        }
    }

    /// Add a pattern
    pub fn with_pattern(mut self, pattern: RulePattern) -> Self {
        self.patterns.push(pattern);
        self.modified_at = Utc::now();
        self
    }

    /// Set action
    pub fn with_action(mut self, action: RuleAction) -> Self {
        self.action = action;
        self.modified_at = Utc::now();
        self
    }

    /// Set age threshold
    pub fn older_than_days(mut self, days: u32) -> Self {
        self.age_days = Some(days);
        self.modified_at = Utc::now();
        self
    }

    /// Set size threshold
    pub fn larger_than(mut self, bytes: u64) -> Self {
        self.size_threshold = Some(bytes);
        self.modified_at = Utc::now();
        self
    }

    /// Add exclude pattern
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self.modified_at = Utc::now();
        self
    }

    /// Check if a file matches this rule
    pub fn matches_file(&self, path: &PathBuf, metadata: &std::fs::Metadata) -> bool {
        if !self.enabled {
            return false;
        }

        let path_str = path.to_string_lossy();

        // Check exclude patterns first
        for exclude in &self.exclude_patterns {
            let pattern = RulePattern::new(exclude.clone());
            if pattern.matches(&path_str) {
                return false;
            }
        }

        // Check patterns
        let pattern_match = if self.patterns.is_empty() {
            true
        } else {
            self.patterns.iter().any(|p| p.matches(&path_str))
        };

        if !pattern_match {
            return false;
        }

        // Check age
        if let Some(age_days) = self.age_days {
            if let Ok(modified) = metadata.modified() {
                let age = std::time::SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();
                if age.as_secs() < (age_days as u64 * 24 * 60 * 60) {
                    return false;
                }
            }
        }

        // Check size
        if let Some(size_threshold) = self.size_threshold {
            if metadata.len() < size_threshold {
                return false;
            }
        }

        true
    }
}

/// Scheduled tasks storage
static SCHEDULED_TASKS: Lazy<Mutex<HashMap<String, ScheduledTask>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Custom rules storage
static CUSTOM_RULES: Lazy<Mutex<HashMap<String, CustomRule>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Add a scheduled task
pub fn add_scheduled_task(task: ScheduledTask) -> Result<(), String> {
    let mut tasks = SCHEDULED_TASKS.lock().map_err(|e| e.to_string())?;
    tasks.insert(task.id.clone(), task);
    Ok(())
}

/// Get a scheduled task
pub fn get_scheduled_task(id: &str) -> Option<ScheduledTask> {
    SCHEDULED_TASKS.lock().ok()?.get(id).cloned()
}

/// Get all scheduled tasks
pub fn get_all_scheduled_tasks() -> Vec<ScheduledTask> {
    SCHEDULED_TASKS
        .lock()
        .map(|tasks| tasks.values().cloned().collect())
        .unwrap_or_default()
}

/// Remove a scheduled task
pub fn remove_scheduled_task(id: &str) -> bool {
    SCHEDULED_TASKS
        .lock()
        .map(|mut tasks| tasks.remove(id).is_some())
        .unwrap_or(false)
}

/// Update a scheduled task
pub fn update_scheduled_task(task: ScheduledTask) -> Result<(), String> {
    let mut tasks = SCHEDULED_TASKS.lock().map_err(|e| e.to_string())?;
    if !tasks.contains_key(&task.id) {
        return Err("Task not found".to_string());
    }
    tasks.insert(task.id.clone(), task);
    Ok(())
}

/// Get tasks that should run now
pub fn get_pending_tasks() -> Vec<ScheduledTask> {
    SCHEDULED_TASKS
        .lock()
        .map(|tasks| {
            tasks
                .values()
                .filter(|t| t.should_run())
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Add a custom rule
pub fn add_custom_rule(rule: CustomRule) -> Result<(), String> {
    let mut rules = CUSTOM_RULES.lock().map_err(|e| e.to_string())?;
    rules.insert(rule.id.clone(), rule);
    Ok(())
}

/// Get a custom rule
pub fn get_custom_rule(id: &str) -> Option<CustomRule> {
    CUSTOM_RULES.lock().ok()?.get(id).cloned()
}

/// Get all custom rules
pub fn get_all_custom_rules() -> Vec<CustomRule> {
    CUSTOM_RULES
        .lock()
        .map(|rules| rules.values().cloned().collect())
        .unwrap_or_default()
}

/// Remove a custom rule
pub fn remove_custom_rule(id: &str) -> bool {
    CUSTOM_RULES
        .lock()
        .map(|mut rules| rules.remove(id).is_some())
        .unwrap_or(false)
}

/// Update a custom rule
pub fn update_custom_rule(rule: CustomRule) -> Result<(), String> {
    let mut rules = CUSTOM_RULES.lock().map_err(|e| e.to_string())?;
    if !rules.contains_key(&rule.id) {
        return Err("Rule not found".to_string());
    }
    rules.insert(rule.id.clone(), rule);
    Ok(())
}

/// Predefined rule templates
pub fn get_rule_templates() -> Vec<CustomRule> {
    vec![
        // Temporary files
        CustomRule::new("temp_files", "Temporary Files", RuleType::FileExtension)
            .with_pattern(RulePattern::new("*.tmp"))
            .with_pattern(RulePattern::new("*.temp"))
            .with_pattern(RulePattern::new("~*"))
            .with_action(RuleAction::SecureDelete { method: "zero".to_string() }),

        // Log files older than 7 days
        CustomRule::new("old_logs", "Old Log Files", RuleType::FileExtension)
            .with_pattern(RulePattern::new("*.log"))
            .older_than_days(7)
            .with_action(RuleAction::SecureDelete { method: "zero".to_string() }),

        // Crash dumps
        CustomRule::new("crash_dumps", "Crash Dumps", RuleType::FileExtension)
            .with_pattern(RulePattern::new("*.dmp"))
            .with_pattern(RulePattern::new("*.crash"))
            .with_pattern(RulePattern::new("*.core"))
            .with_action(RuleAction::SecureDelete { method: "dod".to_string() }),

        // Browser cache
        CustomRule::new("browser_cache", "Browser Cache", RuleType::Directory)
            .with_pattern(RulePattern::new("*/Cache/*"))
            .with_pattern(RulePattern::new("*/cache2/*"))
            .with_action(RuleAction::SecureDelete { method: "zero".to_string() }),

        // Thumbnail cache
        CustomRule::new("thumbnails", "Thumbnail Cache", RuleType::FilePattern)
            .with_pattern(RulePattern::new("thumbs.db"))
            .with_pattern(RulePattern::new("*.thumbcache*"))
            .with_pattern(RulePattern::new("*/.thumbnails/*"))
            .with_action(RuleAction::SecureDelete { method: "zero".to_string() }),

        // Large old files
        CustomRule::new("large_old_files", "Large Old Files", RuleType::FilePattern)
            .with_pattern(RulePattern::new("*"))
            .older_than_days(90)
            .larger_than(100 * 1024 * 1024) // 100MB
            .with_action(RuleAction::Report),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_frequency_minutes() {
        let freq = ScheduleFrequency::Minutes(30);
        let now = Utc::now();
        let next = freq.next_run_from(now);
        assert!(next > now);
        assert!((next - now).num_minutes() == 30);
    }

    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask::new("test", "Test Task", ScheduleFrequency::Daily { hour: 12 })
            .with_action(CleanupAction::TempFiles);

        assert!(task.enabled);
        assert_eq!(task.actions.len(), 1);
        assert!(task.next_run.is_some());
    }

    #[test]
    fn test_custom_rule_pattern_matching() {
        let pattern = RulePattern::new("*.tmp");
        assert!(pattern.matches("/path/to/file.tmp"));
        assert!(!pattern.matches("/path/to/file.txt"));
    }

    #[test]
    fn test_custom_rule_creation() {
        let rule = CustomRule::new("test", "Test Rule", RuleType::FileExtension)
            .with_pattern(RulePattern::new("*.log"))
            .older_than_days(7)
            .with_action(RuleAction::SecureDelete { method: "dod".to_string() });

        assert!(rule.enabled);
        assert_eq!(rule.patterns.len(), 1);
        assert_eq!(rule.age_days, Some(7));
    }
}
