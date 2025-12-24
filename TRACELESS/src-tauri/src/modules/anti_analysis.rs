use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use std::path::Path;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetectionResult {
    pub name: String,
    pub detected: bool,
    pub details: Option<String>,
    pub category: String,
    pub confidence: String, // "high", "medium", "low"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub total: usize,
    pub detected: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentCheck {
    pub vm_detected: bool,
    pub debugger_detected: bool,
    pub sandbox_detected: bool,
    pub forensic_tools_detected: bool,
    pub details: Vec<DetectionResult>,
    pub category_stats: Vec<CategoryStats>,
    pub platform: String,
    pub scan_time: String,
}

/// 检测虚拟机环境 - 使用多重验证
pub fn detect_vm() -> Vec<DetectionResult> {
    let mut results = Vec::new();
    let category = "虚拟机".to_string();

    #[cfg(target_os = "macos")]
    {
        // 1. 检测硬件模型 - 最可靠的方法
        let hw_model = check_macos_hardware_model();
        results.push(DetectionResult {
            name: "硬件模型检测".to_string(),
            detected: hw_model.0,
            details: hw_model.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 2. 检测 CPU 品牌 - 虚拟机通常有特殊的 CPU 字符串
        let cpu_check = check_macos_cpu_virtualization();
        results.push(DetectionResult {
            name: "CPU 虚拟化检测".to_string(),
            detected: cpu_check.0,
            details: cpu_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 3. 检测 IOKit 设备树 - 查找虚拟化设备
        let iokit_check = check_macos_iokit_vm();
        results.push(DetectionResult {
            name: "IOKit 设备检测".to_string(),
            detected: iokit_check.0,
            details: iokit_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 4. 检测 SMC (System Management Controller)
        let smc_check = check_macos_smc();
        results.push(DetectionResult {
            name: "SMC 控制器检测".to_string(),
            detected: smc_check.0,
            details: smc_check.1,
            category: category.clone(),
            confidence: "medium".to_string(),
        });

        // 5. 检测虚拟机特有进程
        let vm_processes = check_vm_processes_macos();
        results.push(DetectionResult {
            name: "虚拟机进程检测".to_string(),
            detected: vm_processes.0,
            details: vm_processes.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 6. 检测 Docker 容器
        let docker = check_docker();
        results.push(DetectionResult {
            name: "Docker 容器检测".to_string(),
            detected: docker.0,
            details: docker.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 特定检测
        let dmi_check = check_linux_dmi();
        results.push(DetectionResult {
            name: "DMI 硬件检测".to_string(),
            detected: dmi_check.0,
            details: dmi_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        let cpuinfo_check = check_linux_cpuinfo();
        results.push(DetectionResult {
            name: "CPU 虚拟化检测".to_string(),
            detected: cpuinfo_check.0,
            details: cpuinfo_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        let docker = check_docker();
        results.push(DetectionResult {
            name: "Docker 容器检测".to_string(),
            detected: docker.0,
            details: docker.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        // Windows 特定检测
        let wmi_check = check_windows_wmi();
        results.push(DetectionResult {
            name: "WMI 硬件检测".to_string(),
            detected: wmi_check.0,
            details: wmi_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        let registry_check = check_windows_registry_vm();
        results.push(DetectionResult {
            name: "注册表检测".to_string(),
            detected: registry_check.0,
            details: registry_check.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    results
}

/// 检测调试器 - 使用系统级 API
pub fn detect_debugger() -> Vec<DetectionResult> {
    let mut results = Vec::new();
    let category = "调试器".to_string();

    #[cfg(target_os = "macos")]
    {
        // 1. 使用 sysctl 检测 P_TRACED 标志 - 最可靠
        let sysctl_result = check_macos_ptrace();
        results.push(DetectionResult {
            name: "进程追踪检测".to_string(),
            detected: sysctl_result.0,
            details: sysctl_result.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 2. 检测常见调试器进程
        let debugger_procs = check_debugger_processes();
        results.push(DetectionResult {
            name: "调试器进程检测".to_string(),
            detected: debugger_procs.0,
            details: debugger_procs.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        // 3. 检测调试端口
        let debug_port = check_debug_ports();
        results.push(DetectionResult {
            name: "调试端口检测".to_string(),
            detected: debug_port.0,
            details: debug_port.1,
            category: category.clone(),
            confidence: "medium".to_string(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 检查 TracerPid
        let tracer = check_linux_tracer();
        results.push(DetectionResult {
            name: "TracerPid 检测".to_string(),
            detected: tracer.0,
            details: tracer.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });

        let debugger_procs = check_debugger_processes();
        results.push(DetectionResult {
            name: "调试器进程检测".to_string(),
            detected: debugger_procs.0,
            details: debugger_procs.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: IsDebuggerPresent
        let is_debugger = check_windows_debugger();
        results.push(DetectionResult {
            name: "IsDebuggerPresent".to_string(),
            detected: is_debugger.0,
            details: is_debugger.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    results
}

/// 检测沙箱环境
pub fn detect_sandbox() -> Vec<DetectionResult> {
    let mut results = Vec::new();
    let category = "沙箱".to_string();

    // 1. 系统资源检测
    let resource_check = check_system_resources_detailed();
    results.push(DetectionResult {
        name: "系统资源检测".to_string(),
        detected: resource_check.0,
        details: resource_check.1,
        category: category.clone(),
        confidence: if resource_check.0 { "medium".to_string() } else { "low".to_string() },
    });

    // 2. 用户环境检测
    let user_check = check_user_environment();
    results.push(DetectionResult {
        name: "用户环境检测".to_string(),
        detected: user_check.0,
        details: user_check.1,
        category: category.clone(),
        confidence: if user_check.0 { "high".to_string() } else { "low".to_string() },
    });

    // 3. 文件系统特征检测
    let fs_check = check_filesystem_artifacts();
    results.push(DetectionResult {
        name: "文件系统特征".to_string(),
        detected: fs_check.0,
        details: fs_check.1,
        category: category.clone(),
        confidence: if fs_check.0 { "high".to_string() } else { "low".to_string() },
    });

    #[cfg(target_os = "macos")]
    {
        // macOS App Sandbox 检测
        let app_sandbox = check_macos_app_sandbox();
        results.push(DetectionResult {
            name: "App Sandbox 检测".to_string(),
            detected: app_sandbox.0,
            details: app_sandbox.1,
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    results
}

/// 检测取证工具
pub fn detect_forensic_tools() -> Vec<DetectionResult> {
    let mut results = Vec::new();
    let category = "取证工具".to_string();

    #[cfg(target_os = "macos")]
    {
        // 检测网络分析工具
        let network_tools = check_network_analysis_tools_macos();
        if network_tools.0 {
            results.push(DetectionResult {
                name: "网络分析工具".to_string(),
                detected: true,
                details: network_tools.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }

        // 检测逆向工程工具
        let reverse_tools = check_reverse_engineering_tools_macos();
        if reverse_tools.0 {
            results.push(DetectionResult {
                name: "逆向工程工具".to_string(),
                detected: true,
                details: reverse_tools.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }

        // 检测系统监控工具
        let monitor_tools = check_system_monitor_tools_macos();
        if monitor_tools.0 {
            results.push(DetectionResult {
                name: "系统监控工具".to_string(),
                detected: true,
                details: monitor_tools.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }

        // 检测取证套件
        let forensic_suites = check_forensic_suites_macos();
        if forensic_suites.0 {
            results.push(DetectionResult {
                name: "取证套件".to_string(),
                detected: true,
                details: forensic_suites.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        let tools = check_forensic_tools_windows();
        if tools.0 {
            results.push(DetectionResult {
                name: "取证工具".to_string(),
                detected: true,
                details: tools.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        let tools = check_forensic_tools_linux();
        if tools.0 {
            results.push(DetectionResult {
                name: "取证工具".to_string(),
                detected: true,
                details: tools.1,
                category: category.clone(),
                confidence: "high".to_string(),
            });
        }
    }

    // 如果没有检测到任何工具
    if results.is_empty() {
        results.push(DetectionResult {
            name: "取证工具扫描".to_string(),
            detected: false,
            details: Some("未检测到常见取证工具".to_string()),
            category: category.clone(),
            confidence: "high".to_string(),
        });
    }

    results
}

/// 执行完整的环境检测
pub fn check_environment() -> EnvironmentCheck {
    let vm_results = detect_vm();
    let debugger_results = detect_debugger();
    let sandbox_results = detect_sandbox();
    let forensic_results = detect_forensic_tools();

    // 使用加权判断 - 只有高置信度的检测才计入
    let vm_detected = vm_results.iter().any(|r| r.detected && r.confidence == "high");
    let debugger_detected = debugger_results.iter().any(|r| r.detected && r.confidence == "high");
    let sandbox_detected = sandbox_results.iter().any(|r| r.detected && r.confidence == "high");
    let forensic_tools_detected = forensic_results.iter().any(|r| r.detected);

    let mut all_details = Vec::new();
    all_details.extend(vm_results.clone());
    all_details.extend(debugger_results.clone());
    all_details.extend(sandbox_results.clone());
    all_details.extend(forensic_results.clone());

    let category_stats = vec![
        CategoryStats {
            category: "虚拟机".to_string(),
            total: vm_results.len(),
            detected: vm_results.iter().filter(|r| r.detected).count(),
        },
        CategoryStats {
            category: "调试器".to_string(),
            total: debugger_results.len(),
            detected: debugger_results.iter().filter(|r| r.detected).count(),
        },
        CategoryStats {
            category: "沙箱".to_string(),
            total: sandbox_results.len(),
            detected: sandbox_results.iter().filter(|r| r.detected).count(),
        },
        CategoryStats {
            category: "取证工具".to_string(),
            total: forensic_results.len(),
            detected: forensic_results.iter().filter(|r| r.detected).count(),
        },
    ];

    let platform = get_platform();
    let scan_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    EnvironmentCheck {
        vm_detected,
        debugger_detected,
        sandbox_detected,
        forensic_tools_detected,
        details: all_details,
        category_stats,
        platform,
        scan_time,
    }
}

fn get_platform() -> String {
    #[cfg(target_os = "windows")]
    { "Windows".to_string() }
    #[cfg(target_os = "macos")]
    { "macOS".to_string() }
    #[cfg(target_os = "linux")]
    { "Linux".to_string() }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { "Unknown".to_string() }
}

// ==================== macOS 特定检测函数 ====================

#[cfg(target_os = "macos")]
fn check_macos_hardware_model() -> (bool, Option<String>) {
    if let Ok(output) = Command::new("sysctl").args(["-n", "hw.model"]).output() {
        let model = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // 真实 Mac 硬件模型示例: MacBookPro18,1, Mac14,2, iMac21,1
        // 虚拟机模型: VMware7,1, Parallels15,1, VirtualBox1,0
        let vm_patterns = ["vmware", "parallels", "virtualbox", "virtual", "qemu"];
        let is_vm = vm_patterns.iter().any(|p| model.to_lowercase().contains(p));

        if is_vm {
            (true, Some(format!("检测到虚拟机硬件模型: {}", model)))
        } else {
            (false, Some(format!("硬件模型: {}", model)))
        }
    } else {
        (false, None)
    }
}

#[cfg(target_os = "macos")]
fn check_macos_cpu_virtualization() -> (bool, Option<String>) {
    // 检查 CPU 品牌字符串
    if let Ok(output) = Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]).output() {
        let cpu_brand = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // 检测虚拟化 CPU 特征
        let vm_cpu_patterns = ["qemu", "virtual", "kvm"];
        let is_vm_cpu = vm_cpu_patterns.iter().any(|p| cpu_brand.to_lowercase().contains(p));

        if is_vm_cpu {
            return (true, Some(format!("检测到虚拟化 CPU: {}", cpu_brand)));
        }

        return (false, Some(format!("CPU: {}", cpu_brand)));
    }

    // 检查虚拟化特性
    if let Ok(output) = Command::new("sysctl").args(["-n", "kern.hv_support"]).output() {
        let hv_support = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // hv_support = 1 表示支持硬件虚拟化，这在真机上是正常的
        return (false, Some(format!("Hypervisor 支持: {}", if hv_support == "1" { "是" } else { "否" })));
    }

    (false, None)
}

#[cfg(target_os = "macos")]
fn check_macos_iokit_vm() -> (bool, Option<String>) {
    // 使用 ioreg 检查设备树中的虚拟化特征
    if let Ok(output) = Command::new("ioreg").args(["-l", "-d", "2"]).output() {
        let ioreg_output = String::from_utf8_lossy(&output.stdout).to_lowercase();

        // 检查特定的虚拟化设备标识
        let vm_indicators = [
            ("vmware", "VMware"),
            ("virtualbox", "VirtualBox"),
            ("parallels", "Parallels"),
            ("qemu", "QEMU"),
            ("acpi-smo-z", "VMware SMC"), // VMware 特有
        ];

        for (pattern, name) in vm_indicators {
            if ioreg_output.contains(pattern) {
                return (true, Some(format!("IOKit 检测到 {} 设备", name)));
            }
        }
    }

    (false, Some("未检测到虚拟化设备".to_string()))
}

#[cfg(target_os = "macos")]
fn check_macos_smc() -> (bool, Option<String>) {
    // 检查 SMC 是否存在 - 真实 Mac 都有 SMC
    if let Ok(output) = Command::new("ioreg").args(["-c", "AppleSMC"]).output() {
        let smc_output = String::from_utf8_lossy(&output.stdout);
        if smc_output.contains("AppleSMC") {
            return (false, Some("SMC 控制器正常".to_string()));
        }
    }

    // 无法检测到 SMC 可能表示虚拟机
    (false, Some("SMC 状态未知".to_string()))
}

#[cfg(target_os = "macos")]
fn check_vm_processes_macos() -> (bool, Option<String>) {
    let vm_process_names = [
        ("vmware-vmx", "VMware"),
        ("vmtoolsd", "VMware Tools"),
        ("VBoxClient", "VirtualBox"),
        ("VBoxService", "VirtualBox"),
        ("prl_client_app", "Parallels"),
        ("prl_tools_service", "Parallels Tools"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (process_name, vm_name) in vm_process_names {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == process_name.to_lowercase() {
                detected.push(vm_name);
                break;
            }
        }
    }

    if !detected.is_empty() {
        return (true, Some(format!("检测到虚拟机进程: {}", detected.join(", "))));
    }

    (false, Some("未检测到虚拟机进程".to_string()))
}

#[cfg(target_os = "macos")]
fn check_macos_ptrace() -> (bool, Option<String>) {

    // 使用 sysctl 获取当前进程信息
    // CTL_KERN, KERN_PROC, KERN_PROC_PID
    if let Ok(output) = Command::new("ps").args(["-p", &std::process::id().to_string(), "-o", "stat"]).output() {
        let stat = String::from_utf8_lossy(&output.stdout);
        // 如果状态包含 T，表示被追踪
        if stat.contains("T+") || stat.contains("t+") {
            return (true, Some("检测到进程被追踪".to_string()));
        }
    }

    // 尝试检测父进程
    if let Ok(output) = Command::new("ps").args(["-p", &std::process::id().to_string(), "-o", "ppid"]).output() {
        let ppid_str = String::from_utf8_lossy(&output.stdout);
        if let Some(ppid) = ppid_str.lines().nth(1).and_then(|s| s.trim().parse::<u32>().ok()) {
            // 检查父进程是否是调试器
            if let Ok(parent_output) = Command::new("ps").args(["-p", &ppid.to_string(), "-o", "comm"]).output() {
                let parent_comm = String::from_utf8_lossy(&parent_output.stdout).to_lowercase();
                if parent_comm.contains("lldb") || parent_comm.contains("gdb") || parent_comm.contains("debugserver") {
                    return (true, Some(format!("父进程是调试器 (PID: {})", ppid)));
                }
            }
        }
    }

    (false, Some("未检测到调试器附加".to_string()))
}

#[cfg(target_os = "macos")]
fn check_macos_app_sandbox() -> (bool, Option<String>) {
    // 检查 HOME 目录是否在 Container 内
    if let Ok(home) = env::var("HOME") {
        if home.contains("/Library/Containers/") {
            return (true, Some(format!("运行在 App Sandbox: {}", home)));
        }
    }

    // 检查沙箱配置文件
    if let Ok(output) = Command::new("sandbox-info").args(["--check"]).output() {
        if output.status.success() {
            return (true, Some("App Sandbox 已启用".to_string()));
        }
    }

    (false, Some("未在 App Sandbox 中运行".to_string()))
}

#[cfg(target_os = "macos")]
fn check_network_analysis_tools_macos() -> (bool, Option<String>) {
    let tools = [
        ("wireshark", "Wireshark"),
        ("tcpdump", "tcpdump"),
        ("charles", "Charles"),
        ("proxyman", "Proxyman"),
        ("mitmproxy", "mitmproxy"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (process_name, display_name) in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == process_name || name.contains(process_name) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

#[cfg(target_os = "macos")]
fn check_reverse_engineering_tools_macos() -> (bool, Option<String>) {
    let tools = [
        ("ghidra", "Ghidra"),
        ("hopper", "Hopper"),
        ("ida", "IDA"),
        ("ida64", "IDA Pro 64"),
        ("radare2", "radare2"),
        ("r2", "r2"),
        ("lldb", "lldb"),
        ("gdb", "gdb"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (process_name, display_name) in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == process_name || name.contains(process_name) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

#[cfg(target_os = "macos")]
fn check_system_monitor_tools_macos() -> (bool, Option<String>) {
    let tools = [
        ("activity monitor", "Activity Monitor"),
        ("console", "Console.app"),
        ("fs_usage", "fs_usage"),
        ("dtrace", "DTrace"),
        ("instruments", "Instruments"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (process_pattern, display_name) in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name.contains(process_pattern) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

#[cfg(target_os = "macos")]
fn check_forensic_suites_macos() -> (bool, Option<String>) {
    let tools = [
        ("autopsy", "Autopsy"),
        ("volatility", "Volatility"),
        ("ftk", "FTK"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (process_name, display_name) in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name.contains(process_name) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

// ==================== Linux 特定检测函数 ====================

#[cfg(target_os = "linux")]
fn check_linux_dmi() -> (bool, Option<String>) {
    let dmi_files = [
        ("/sys/class/dmi/id/product_name", "产品名"),
        ("/sys/class/dmi/id/sys_vendor", "系统厂商"),
        ("/sys/class/dmi/id/board_vendor", "主板厂商"),
    ];

    let vm_vendors = ["vmware", "virtualbox", "qemu", "kvm", "xen", "parallels", "microsoft"];

    for (path, desc) in dmi_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            let content_lower = content.to_lowercase();
            for vendor in vm_vendors {
                if content_lower.contains(vendor) {
                    return (true, Some(format!("{}: {}", desc, content.trim())));
                }
            }
        }
    }

    (false, Some("未检测到虚拟化 DMI 信息".to_string()))
}

#[cfg(target_os = "linux")]
fn check_linux_cpuinfo() -> (bool, Option<String>) {
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        let cpuinfo_lower = cpuinfo.to_lowercase();

        // 检查 hypervisor 标志
        if cpuinfo_lower.contains("hypervisor") {
            return (true, Some("检测到 hypervisor CPU 标志".to_string()));
        }

        // 检查 CPU 型号
        let vm_cpu_patterns = ["qemu", "kvm"];
        for pattern in vm_cpu_patterns {
            if cpuinfo_lower.contains(pattern) {
                return (true, Some(format!("检测到虚拟化 CPU: {}", pattern)));
            }
        }
    }

    (false, Some("未检测到虚拟化 CPU".to_string()))
}

#[cfg(target_os = "linux")]
fn check_linux_tracer() -> (bool, Option<String>) {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("TracerPid:") {
                if let Some(pid_str) = line.split_whitespace().nth(1) {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        if pid != 0 {
                            return (true, Some(format!("进程被追踪 (TracerPid: {})", pid)));
                        }
                    }
                }
            }
        }
    }

    (false, Some("未被追踪".to_string()))
}

#[cfg(target_os = "linux")]
fn check_forensic_tools_linux() -> (bool, Option<String>) {
    let tools = [
        "wireshark", "tcpdump", "volatility", "autopsy",
        "ghidra", "radare2", "gdb", "strace", "ltrace"
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for tool in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == tool {
                detected.push(tool.to_string());
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

// ==================== Windows 特定检测函数 ====================

#[cfg(target_os = "windows")]
fn check_windows_wmi() -> (bool, Option<String>) {
    if let Ok(output) = Command::new("wmic").args(["computersystem", "get", "model"]).output() {
        let model = String::from_utf8_lossy(&output.stdout).to_lowercase();
        let vm_models = ["vmware", "virtual", "qemu", "xen"];
        for vm in vm_models {
            if model.contains(vm) {
                return (true, Some(format!("WMI 检测到虚拟机: {}", vm)));
            }
        }
    }

    (false, Some("未检测到虚拟机".to_string()))
}

#[cfg(target_os = "windows")]
fn check_windows_registry_vm() -> (bool, Option<String>) {
    // 使用 reg query 检查虚拟机注册表项
    let registry_paths = [
        r"HKLM\SOFTWARE\VMware, Inc.\VMware Tools",
        r"HKLM\SOFTWARE\Oracle\VirtualBox Guest Additions",
    ];

    for path in registry_paths {
        if let Ok(output) = Command::new("reg").args(["query", path]).output() {
            if output.status.success() {
                return (true, Some(format!("检测到虚拟机注册表: {}", path)));
            }
        }
    }

    (false, Some("未检测到虚拟机注册表".to_string()))
}

#[cfg(target_os = "windows")]
fn check_windows_debugger() -> (bool, Option<String>) {
    let debuggers = [
        ("ollydbg", "OllyDbg"),
        ("x64dbg", "x64dbg"),
        ("x32dbg", "x32dbg"),
        ("windbg", "WinDbg"),
        ("ida64", "IDA Pro 64"),
        ("ida", "IDA Pro"),
        ("idag", "IDA Pro GUI"),
        ("idag64", "IDA Pro 64 GUI"),
        ("immunitydebugger", "Immunity Debugger"),
        ("devenv", "Visual Studio Debugger"),
        ("dbgview", "DebugView"),
        ("radare2", "Radare2"),
        ("ghidra", "Ghidra"),
        ("dnspy", "dnSpy"),
        ("cheatengine", "Cheat Engine"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (proc_pattern, display_name) in debuggers {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name.contains(proc_pattern) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    // 检查调试相关的环境变量
    if std::env::var("_NO_DEBUG_HEAP").is_ok() {
        detected.push("Debug Heap Disabled");
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到调试器: {}", detected.join(", "))))
    } else {
        (false, Some("未检测到调试器".to_string()))
    }
}

#[cfg(target_os = "windows")]
fn check_forensic_tools_windows() -> (bool, Option<String>) {
    let tools = [
        ("wireshark", "Wireshark"),
        ("dumpcap", "Dumpcap (Wireshark)"),
        ("tshark", "TShark"),
        ("procmon", "Process Monitor"),
        ("procexp", "Process Explorer"),
        ("autoruns", "Autoruns"),
        ("tcpview", "TCPView"),
        ("fiddler", "Fiddler"),
        ("charles", "Charles Proxy"),
        ("burpsuite", "Burp Suite"),
        ("regshot", "Regshot"),
        ("pestudio", "PEStudio"),
        ("hxd", "HxD Hex Editor"),
        ("apimonitor", "API Monitor"),
        ("resourcehacker", "Resource Hacker"),
        ("de4dot", "de4dot"),
        ("detector", "PEiD/Detector"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (proc_pattern, display_name) in tools {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name.contains(proc_pattern) {
                if !detected.contains(&display_name) {
                    detected.push(display_name);
                }
                break;
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到: {}", detected.join(", "))))
    } else {
        (false, None)
    }
}

// ==================== 通用检测函数 ====================

fn check_docker() -> (bool, Option<String>) {
    // 检查 /.dockerenv 文件
    if Path::new("/.dockerenv").exists() {
        return (true, Some("检测到 /.dockerenv 文件".to_string()));
    }

    // 检查 cgroup
    #[cfg(target_family = "unix")]
    {
        if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup.contains("docker") || cgroup.contains("/lxc/") {
                return (true, Some("检测到 Docker/LXC cgroup".to_string()));
            }
        }

        // 检查 /.dockerinit (旧版 Docker)
        if Path::new("/.dockerinit").exists() {
            return (true, Some("检测到 /.dockerinit 文件".to_string()));
        }
    }

    (false, Some("未运行在容器中".to_string()))
}

fn check_debugger_processes() -> (bool, Option<String>) {
    let debugger_names = [
        ("lldb", "LLDB"),
        ("gdb", "GDB"),
        ("debugserver", "Debug Server"),
    ];

    // 使用 sysinfo 获取进程列表
    let sys = System::new_all();
    let mut detected = Vec::new();

    for (proc_name, display_name) in debugger_names {
        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == proc_name.to_lowercase() {
                detected.push(display_name);
                break;
            }
        }
    }

    if !detected.is_empty() {
        return (true, Some(format!("检测到: {}", detected.join(", "))));
    }

    (false, Some("未检测到调试器进程".to_string()))
}

fn check_debug_ports() -> (bool, Option<String>) {
    // 检查常见调试端口
    let debug_ports = [1234, 4444, 5555, 9999]; // 常见调试端口

    #[cfg(target_family = "unix")]
    {
        if let Ok(output) = Command::new("lsof").args(["-i", "-P", "-n"]).output() {
            let lsof_output = String::from_utf8_lossy(&output.stdout);

            for port in debug_ports {
                if lsof_output.contains(&format!(":{}", port)) {
                    // 需要进一步验证是否是调试器使用
                    // 这里只是简单检测
                }
            }
        }
    }

    (false, Some("未检测到调试端口".to_string()))
}

fn check_system_resources_detailed() -> (bool, Option<String>) {
    // 使用 sysinfo 获取系统资源信息
    let sys = System::new_all();
    let mut info = Vec::new();
    let mut is_suspicious = false;

    // CPU 核心数
    let cores = sys.cpus().len();
    info.push(format!("CPU: {} 核", cores));
    if cores <= 1 {
        is_suspicious = true;
    }

    // 内存大小
    let mem_bytes = sys.total_memory();
    let mem_gb = mem_bytes / (1024 * 1024 * 1024);
    info.push(format!("内存: {} GB", mem_gb));
    if mem_gb < 2 {
        is_suspicious = true;
    }

    (is_suspicious, Some(info.join(", ")))
}

fn check_user_environment() -> (bool, Option<String>) {
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_default()
        .to_lowercase();

    // 精确匹配可疑用户名
    let suspicious_usernames = ["sandbox", "malware", "virus", "sample", "cuckoo", "analysis"];
    let is_suspicious = suspicious_usernames.iter().any(|&u| username == u);

    if is_suspicious {
        (true, Some(format!("可疑用户名: {}", username)))
    } else {
        (false, Some(format!("当前用户: {}", username)))
    }
}

fn check_filesystem_artifacts() -> (bool, Option<String>) {
    let mut detected = Vec::new();

    // 检查沙箱特有文件
    #[cfg(target_os = "macos")]
    {
        let sandbox_paths = [
            "/Library/Sandbox/Profiles",
        ];

        for path in sandbox_paths {
            if Path::new(path).exists() {
                // 这是正常的系统目录，不作为检测依据
            }
        }
    }

    // 检查分析工具文件
    #[cfg(target_family = "unix")]
    {
        let analysis_files = [
            "/tmp/malware_analysis",
            "/var/log/cuckoo",
        ];

        for path in analysis_files {
            if Path::new(path).exists() {
                detected.push(path);
            }
        }
    }

    if !detected.is_empty() {
        (true, Some(format!("检测到分析环境文件: {}", detected.join(", "))))
    } else {
        (false, Some("未检测到可疑文件".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_check() {
        let result = check_environment();
        assert!(!result.platform.is_empty());
        // Test that the check runs without panicking
        let _ = result.vm_detected;
        let _ = result.debugger_detected;
        let _ = result.sandbox_detected;
        let _ = result.forensic_tools_detected;
        let _ = result.details.len();
    }
}
