use crate::modules::memory_cleaner;
use crate::commands::permission_ops::require_admin_for_operation;

#[tauri::command]
pub async fn get_memory_info() -> Result<memory_cleaner::MemoryInfo, String> {
    memory_cleaner::get_memory_info()
}

#[tauri::command]
pub async fn get_detailed_memory_info() -> Result<memory_cleaner::DetailedMemoryInfo, String> {
    memory_cleaner::get_detailed_memory_info()
}

#[tauri::command]
pub async fn get_top_processes() -> Result<Vec<memory_cleaner::ProcessInfo>, String> {
    memory_cleaner::get_top_processes()
}

#[tauri::command]
pub async fn scan_memory_items() -> Result<memory_cleaner::MemoryScanResult, String> {
    memory_cleaner::scan_memory_items()
}

#[tauri::command]
pub async fn clean_memory(types: Vec<String>) -> Result<String, String> {
    if types.is_empty() {
        return Err("INVALID_INPUT:selectMemoryTypes".to_string());
    }

    // Check permission for operations that require elevation
    for memory_type in &types {
        let op_name = format!("memory_{}", memory_type);
        require_admin_for_operation(&op_name)?;
    }

    match memory_cleaner::clean_memory(types.clone()) {
        Ok(cleaned) => Ok(format!(
            "OK:{}:{}",
            cleaned.len(),
            cleaned.join(",")
        )),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_memory_info() {
        let result = get_memory_info().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_detailed_memory_info() {
        let result = get_detailed_memory_info().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_top_processes() {
        let result = get_top_processes().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clean_memory_empty() {
        let result = clean_memory(vec![]).await;
        assert!(result.is_err());
    }
}
