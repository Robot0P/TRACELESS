// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod modules;

use commands::file_ops::{secure_delete_file, get_file_info};
use commands::system_logs::{clear_system_logs, get_platform, scan_system_logs};
use commands::memory_ops::{clean_memory, get_memory_info, get_detailed_memory_info, get_top_processes, scan_memory_items};
use commands::network_ops::{clean_network, get_network_info, scan_network_items};
use commands::registry_ops::{clean_registry, get_registry_info};
use commands::timestamp_ops::{get_file_timestamps, modify_file_timestamps};
use commands::anti_analysis_ops::check_environment;
use commands::scan_ops::{perform_system_scan, cleanup_scan_items};
use commands::disk_encryption_ops::{check_disk_encryption, enable_disk_encryption, disable_disk_encryption, get_encryption_recommendations};
use commands::system_info_ops::{get_system_info_api, get_network_speed_api, get_disks_info_api};
use commands::permission_ops::{check_admin_permission, get_elevation_guide, requires_admin, request_admin_elevation, open_privacy_settings, check_permission_initialized, initialize_permissions, get_permission_status, check_full_disk_access, open_full_disk_access_settings, open_accessibility_settings, run_with_admin, check_system_privileges, run_as_system, run_as_trustedinstaller};
use commands::settings_ops::{save_settings, load_settings, reset_settings, get_settings_info};
use commands::scheduled_ops::{get_scheduled_tasks, get_scheduled_task_by_id, create_scheduled_task, update_scheduled_task_cmd, delete_scheduled_task, toggle_scheduled_task, get_custom_rules, get_custom_rule_by_id, create_custom_rule, update_custom_rule_cmd, delete_custom_rule, toggle_custom_rule, get_rule_templates_cmd, get_pending_scheduled_tasks};
use commands::platform_ops::{get_platform_info, get_linux_distro_info, check_feature_available, get_feature_reason, get_running_operations, cancel_running_operation, cancel_all_running_operations, get_timeout_settings, set_timeout_settings, get_cpu_info, check_features};
use commands::cleanup_ops::{perform_app_self_clean, get_app_traces_info, preview_deletion, preview_log_clean, preview_browser_clean, preview_shell_clean, preview_dns_clean, preview_trash_clean, preview_all_cleanup};
use commands::license_ops::{get_machine_id, get_full_machine_id, get_license_status, validate_license, activate_license, deactivate_license, get_feature_access, can_access_feature, is_pro_user};
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::Emitter;

fn create_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    // 文件菜单
    let file_menu = Submenu::with_items(
        app,
        "文件",
        true,
        &[
            &MenuItem::with_id(app, "preferences", "偏好设置", true, Some("CmdOrCtrl+,"))?,
            &MenuItem::with_id(app, "quit", "退出", true, Some("CmdOrCtrl+Q"))?,
        ],
    )?;

    // 视图菜单
    let view_menu = Submenu::with_items(
        app,
        "视图",
        true,
        &[
            &MenuItem::with_id(app, "dashboard", "仪表盘", true, Some("CmdOrCtrl+D"))?,
            &MenuItem::with_id(app, "reload", "重新加载", true, Some("CmdOrCtrl+R"))?,
            &MenuItem::with_id(app, "toggle_fullscreen", "全屏", true, Some("F11"))?,
        ],
    )?;

    // 帮助菜单
    let help_menu = Submenu::with_items(
        app,
        "帮助",
        true,
        &[
            &MenuItem::with_id(app, "documentation", "文档", true, None::<&str>)?,
            &MenuItem::with_id(app, "about", "关于", true, None::<&str>)?,
        ],
    )?;

    Menu::with_items(
        app,
        &[
            &file_menu,
            &view_menu,
            &help_menu,
        ],
    )
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let menu = create_menu(app.handle())?;
            app.set_menu(menu)?;

            app.on_menu_event(|app_handle, event| {
                use tauri::Manager;
                match event.id().as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "preferences" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval("window.location.hash = '#/settings'");
                        }
                    }
                    "reload" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval("location.reload()");
                        }
                    }
                    "toggle_fullscreen" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if let Ok(is_fullscreen) = window.is_fullscreen() {
                                let _ = window.set_fullscreen(!is_fullscreen);
                            }
                        }
                    }
                    "dashboard" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval("window.location.hash = '#/dashboard'");
                        }
                    }
                    "about" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.emit("menu-about", ());
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // File operations
            secure_delete_file,
            get_file_info,
            // System logs
            clear_system_logs,
            get_platform,
            scan_system_logs,
            // Memory operations
            clean_memory,
            get_memory_info,
            get_detailed_memory_info,
            get_top_processes,
            scan_memory_items,
            // Network operations
            clean_network,
            get_network_info,
            scan_network_items,
            // Registry operations
            clean_registry,
            get_registry_info,
            // Timestamp operations
            get_file_timestamps,
            modify_file_timestamps,
            // Anti-analysis operations
            check_environment,
            // Scan operations
            perform_system_scan,
            cleanup_scan_items,
            // Disk encryption operations
            check_disk_encryption,
            enable_disk_encryption,
            disable_disk_encryption,
            get_encryption_recommendations,
            // System info operations
            get_system_info_api,
            get_network_speed_api,
            get_disks_info_api,
            // Permission operations
            check_admin_permission,
            get_elevation_guide,
            requires_admin,
            request_admin_elevation,
            open_privacy_settings,
            check_permission_initialized,
            initialize_permissions,
            get_permission_status,
            check_full_disk_access,
            open_full_disk_access_settings,
            open_accessibility_settings,
            run_with_admin,
            check_system_privileges,
            run_as_system,
            run_as_trustedinstaller,
            // Settings operations
            save_settings,
            load_settings,
            reset_settings,
            get_settings_info,
            // Scheduled task operations
            get_scheduled_tasks,
            get_scheduled_task_by_id,
            create_scheduled_task,
            update_scheduled_task_cmd,
            delete_scheduled_task,
            toggle_scheduled_task,
            get_pending_scheduled_tasks,
            // Custom rule operations
            get_custom_rules,
            get_custom_rule_by_id,
            create_custom_rule,
            update_custom_rule_cmd,
            delete_custom_rule,
            toggle_custom_rule,
            get_rule_templates_cmd,
            // Platform operations
            get_platform_info,
            get_linux_distro_info,
            check_feature_available,
            get_feature_reason,
            check_features,
            get_cpu_info,
            // Operation management
            get_running_operations,
            cancel_running_operation,
            cancel_all_running_operations,
            get_timeout_settings,
            set_timeout_settings,
            // Self-cleaning and dry-run operations
            perform_app_self_clean,
            get_app_traces_info,
            preview_deletion,
            preview_log_clean,
            preview_browser_clean,
            preview_shell_clean,
            preview_dns_clean,
            preview_trash_clean,
            preview_all_cleanup,
            // License operations
            get_machine_id,
            get_full_machine_id,
            get_license_status,
            validate_license,
            activate_license,
            deactivate_license,
            get_feature_access,
            can_access_feature,
            is_pro_user,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
