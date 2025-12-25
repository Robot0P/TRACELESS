//! Scheduled cleanup commands
//!
//! Tauri commands for managing scheduled tasks and custom cleanup rules.

use crate::modules::{
    scheduled_cleanup::{
        ScheduleFrequency, ScheduledTask, CleanupAction, TaskResult, CustomRule, RuleType,
        RulePattern, RuleAction,
    },
    error_handling::{OperationLog, OperationType, OperationStatus, OperationError, ErrorCode},
};
use serde::{Deserialize, Serialize};

/// Scheduled task data for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskData {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub frequency_type: String,
    pub frequency_value: u32,
    pub frequency_day: Option<u32>,
    pub frequency_hour: Option<u32>,
    pub actions: Vec<String>,
    pub custom_rules: Vec<String>,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub last_success: Option<bool>,
}

/// Custom rule data for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rule_type: String,
    pub patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub age_days: Option<u32>,
    pub size_threshold: Option<u64>,
    pub action: String,
    pub action_method: Option<String>,
}

/// Get all scheduled tasks
#[tauri::command]
pub fn get_scheduled_tasks() -> Result<Vec<ScheduledTaskData>, String> {
    let tasks = crate::modules::get_all_scheduled_tasks();
    Ok(tasks.into_iter().map(|t| task_to_data(&t)).collect())
}

/// Get a single scheduled task
#[tauri::command]
pub fn get_scheduled_task_by_id(id: String) -> Result<Option<ScheduledTaskData>, String> {
    Ok(crate::modules::get_scheduled_task(&id).map(|t| task_to_data(&t)))
}

/// Create a new scheduled task
#[tauri::command]
pub fn create_scheduled_task(data: ScheduledTaskData) -> Result<ScheduledTaskData, String> {
    let frequency = parse_frequency(&data)?;
    let mut task = ScheduledTask::new(&data.id, &data.name, frequency);

    task.enabled = data.enabled;
    task.custom_rules = data.custom_rules.clone();

    // Parse actions
    for action_str in &data.actions {
        if let Some(action) = parse_action(action_str) {
            task.actions.push(action);
        }
    }

    crate::modules::add_scheduled_task(task.clone())?;
    Ok(task_to_data(&task))
}

/// Update an existing scheduled task
#[tauri::command]
pub fn update_scheduled_task_cmd(data: ScheduledTaskData) -> Result<ScheduledTaskData, String> {
    let frequency = parse_frequency(&data)?;
    let existing = crate::modules::get_scheduled_task(&data.id)
        .ok_or_else(|| "Task not found".to_string())?;

    let mut task = ScheduledTask::new(&data.id, &data.name, frequency);
    task.enabled = data.enabled;
    task.custom_rules = data.custom_rules.clone();
    task.last_run = existing.last_run;
    task.last_result = existing.last_result;
    task.created_at = existing.created_at;

    // Parse actions
    for action_str in &data.actions {
        if let Some(action) = parse_action(action_str) {
            task.actions.push(action);
        }
    }

    crate::modules::update_scheduled_task(task.clone())?;
    Ok(task_to_data(&task))
}

/// Delete a scheduled task
#[tauri::command]
pub fn delete_scheduled_task(id: String) -> Result<bool, String> {
    Ok(crate::modules::remove_scheduled_task(&id))
}

/// Toggle scheduled task enabled state
#[tauri::command]
pub fn toggle_scheduled_task(id: String, enabled: bool) -> Result<bool, String> {
    if let Some(mut task) = crate::modules::get_scheduled_task(&id) {
        task.enabled = enabled;
        crate::modules::update_scheduled_task(task)?;
        Ok(true)
    } else {
        Err("Task not found".to_string())
    }
}

/// Get all custom rules
#[tauri::command]
pub fn get_custom_rules() -> Result<Vec<CustomRuleData>, String> {
    let rules = crate::modules::get_all_custom_rules();
    Ok(rules.into_iter().map(|r| rule_to_data(&r)).collect())
}

/// Get a single custom rule
#[tauri::command]
pub fn get_custom_rule_by_id(id: String) -> Result<Option<CustomRuleData>, String> {
    Ok(crate::modules::get_custom_rule(&id).map(|r| rule_to_data(&r)))
}

/// Create a new custom rule
#[tauri::command]
pub fn create_custom_rule(data: CustomRuleData) -> Result<CustomRuleData, String> {
    let rule_type = parse_rule_type(&data.rule_type)?;
    let action = parse_rule_action(&data.action, data.action_method.as_deref())?;

    let mut rule = CustomRule::new(&data.id, &data.name, rule_type);
    rule.description = data.description.clone();
    rule.enabled = data.enabled;
    rule.action = action;
    rule.age_days = data.age_days;
    rule.size_threshold = data.size_threshold;
    rule.exclude_patterns = data.exclude_patterns.clone();

    // Add patterns
    for pattern_str in &data.patterns {
        rule.patterns.push(RulePattern::new(pattern_str));
    }

    crate::modules::add_custom_rule(rule.clone())?;
    Ok(rule_to_data(&rule))
}

/// Update an existing custom rule
#[tauri::command]
pub fn update_custom_rule_cmd(data: CustomRuleData) -> Result<CustomRuleData, String> {
    let rule_type = parse_rule_type(&data.rule_type)?;
    let action = parse_rule_action(&data.action, data.action_method.as_deref())?;

    let existing = crate::modules::get_custom_rule(&data.id)
        .ok_or_else(|| "Rule not found".to_string())?;

    let mut rule = CustomRule::new(&data.id, &data.name, rule_type);
    rule.description = data.description.clone();
    rule.enabled = data.enabled;
    rule.action = action;
    rule.age_days = data.age_days;
    rule.size_threshold = data.size_threshold;
    rule.exclude_patterns = data.exclude_patterns.clone();
    rule.created_at = existing.created_at;

    // Add patterns
    for pattern_str in &data.patterns {
        rule.patterns.push(RulePattern::new(pattern_str));
    }

    crate::modules::update_custom_rule(rule.clone())?;
    Ok(rule_to_data(&rule))
}

/// Delete a custom rule
#[tauri::command]
pub fn delete_custom_rule(id: String) -> Result<bool, String> {
    Ok(crate::modules::remove_custom_rule(&id))
}

/// Toggle custom rule enabled state
#[tauri::command]
pub fn toggle_custom_rule(id: String, enabled: bool) -> Result<bool, String> {
    if let Some(mut rule) = crate::modules::get_custom_rule(&id) {
        rule.enabled = enabled;
        crate::modules::update_custom_rule(rule)?;
        Ok(true)
    } else {
        Err("Rule not found".to_string())
    }
}

/// Get predefined rule templates
#[tauri::command]
pub fn get_rule_templates_cmd() -> Result<Vec<CustomRuleData>, String> {
    let templates = crate::modules::get_rule_templates();
    Ok(templates.into_iter().map(|r| rule_to_data(&r)).collect())
}

/// Get pending tasks that should run
#[tauri::command]
pub fn get_pending_scheduled_tasks() -> Result<Vec<ScheduledTaskData>, String> {
    let tasks = crate::modules::get_pending_tasks();
    Ok(tasks.into_iter().map(|t| task_to_data(&t)).collect())
}

// Helper functions

fn task_to_data(task: &ScheduledTask) -> ScheduledTaskData {
    let (freq_type, freq_value, freq_day, freq_hour) = match task.frequency {
        ScheduleFrequency::Minutes(n) => ("minutes".to_string(), n, None, None),
        ScheduleFrequency::Hourly(n) => ("hourly".to_string(), n, None, None),
        ScheduleFrequency::Daily { hour } => ("daily".to_string(), 1, None, Some(hour)),
        ScheduleFrequency::Weekly { day, hour } => ("weekly".to_string(), 1, Some(day), Some(hour)),
        ScheduleFrequency::Monthly { day, hour } => ("monthly".to_string(), 1, Some(day), Some(hour)),
    };

    let actions: Vec<String> = task.actions.iter().map(|a| match a {
        CleanupAction::SystemLogs => "system_logs".to_string(),
        CleanupAction::Memory => "memory".to_string(),
        CleanupAction::Network => "network".to_string(),
        CleanupAction::Registry => "registry".to_string(),
        CleanupAction::BrowserData { .. } => "browser_data".to_string(),
        CleanupAction::Paths { .. } => "paths".to_string(),
        CleanupAction::CustomRules { .. } => "custom_rules".to_string(),
        CleanupAction::TempFiles => "temp_files".to_string(),
        CleanupAction::DnsCache => "dns_cache".to_string(),
        CleanupAction::RecentDocuments => "recent_documents".to_string(),
        CleanupAction::ThumbnailCache => "thumbnail_cache".to_string(),
    }).collect();

    ScheduledTaskData {
        id: task.id.clone(),
        name: task.name.clone(),
        enabled: task.enabled,
        frequency_type: freq_type,
        frequency_value: freq_value,
        frequency_day: freq_day,
        frequency_hour: freq_hour,
        actions,
        custom_rules: task.custom_rules.clone(),
        last_run: task.last_run.map(|t| t.to_rfc3339()),
        next_run: task.next_run.map(|t| t.to_rfc3339()),
        last_success: task.last_result.as_ref().map(|r| r.success),
    }
}

fn rule_to_data(rule: &CustomRule) -> CustomRuleData {
    let rule_type = match rule.rule_type {
        RuleType::FileExtension => "file_extension",
        RuleType::FilePattern => "file_pattern",
        RuleType::Regex => "regex",
        RuleType::Directory => "directory",
        RuleType::ContentMatch => "content_match",
    };

    let (action, action_method) = match &rule.action {
        RuleAction::SecureDelete { method } => ("secure_delete".to_string(), Some(method.clone())),
        RuleAction::MoveToTrash => ("move_to_trash".to_string(), None),
        RuleAction::Archive { .. } => ("archive".to_string(), None),
        RuleAction::Report => ("report".to_string(), None),
    };

    CustomRuleData {
        id: rule.id.clone(),
        name: rule.name.clone(),
        description: rule.description.clone(),
        enabled: rule.enabled,
        rule_type: rule_type.to_string(),
        patterns: rule.patterns.iter().map(|p| p.pattern.clone()).collect(),
        exclude_patterns: rule.exclude_patterns.clone(),
        age_days: rule.age_days,
        size_threshold: rule.size_threshold,
        action,
        action_method,
    }
}

fn parse_frequency(data: &ScheduledTaskData) -> Result<ScheduleFrequency, String> {
    match data.frequency_type.as_str() {
        "minutes" => Ok(ScheduleFrequency::Minutes(data.frequency_value)),
        "hourly" => Ok(ScheduleFrequency::Hourly(data.frequency_value)),
        "daily" => Ok(ScheduleFrequency::Daily {
            hour: data.frequency_hour.unwrap_or(0),
        }),
        "weekly" => Ok(ScheduleFrequency::Weekly {
            day: data.frequency_day.unwrap_or(0),
            hour: data.frequency_hour.unwrap_or(0),
        }),
        "monthly" => Ok(ScheduleFrequency::Monthly {
            day: data.frequency_day.unwrap_or(1),
            hour: data.frequency_hour.unwrap_or(0),
        }),
        _ => Err(format!("Unknown frequency type: {}", data.frequency_type)),
    }
}

fn parse_action(action_str: &str) -> Option<CleanupAction> {
    match action_str {
        "system_logs" => Some(CleanupAction::SystemLogs),
        "memory" => Some(CleanupAction::Memory),
        "network" => Some(CleanupAction::Network),
        "registry" => Some(CleanupAction::Registry),
        "browser_data" => Some(CleanupAction::BrowserData { browsers: vec![] }),
        "temp_files" => Some(CleanupAction::TempFiles),
        "dns_cache" => Some(CleanupAction::DnsCache),
        "recent_documents" => Some(CleanupAction::RecentDocuments),
        "thumbnail_cache" => Some(CleanupAction::ThumbnailCache),
        _ => None,
    }
}

fn parse_rule_type(type_str: &str) -> Result<RuleType, String> {
    match type_str {
        "file_extension" => Ok(RuleType::FileExtension),
        "file_pattern" => Ok(RuleType::FilePattern),
        "regex" => Ok(RuleType::Regex),
        "directory" => Ok(RuleType::Directory),
        "content_match" => Ok(RuleType::ContentMatch),
        _ => Err(format!("Unknown rule type: {}", type_str)),
    }
}

fn parse_rule_action(action: &str, method: Option<&str>) -> Result<RuleAction, String> {
    match action {
        "secure_delete" => Ok(RuleAction::SecureDelete {
            method: method.unwrap_or("dod").to_string(),
        }),
        "move_to_trash" => Ok(RuleAction::MoveToTrash),
        "archive" => Ok(RuleAction::Archive {
            destination: String::new(),
        }),
        "report" => Ok(RuleAction::Report),
        _ => Err(format!("Unknown action: {}", action)),
    }
}
