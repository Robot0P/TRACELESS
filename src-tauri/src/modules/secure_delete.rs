use rand::Rng;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum WipeMethod {
    Zero,           // 零填充
    Random,         // 随机数据
    DoD5220,        // 美国国防部标准 (7次)
    Gutmann,        // Gutmann 方法 (35次)
}

impl WipeMethod {
    pub fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "zero" => Ok(WipeMethod::Zero),
            "random" => Ok(WipeMethod::Random),
            "dod" | "dod5220" => Ok(WipeMethod::DoD5220),
            "gutmann" => Ok(WipeMethod::Gutmann),
            _ => Err(format!("未知的擦除方法: {}", s)),
        }
    }

    pub fn passes(&self) -> u32 {
        match self {
            WipeMethod::Zero => 1,
            WipeMethod::Random => 3,
            WipeMethod::DoD5220 => 7,
            WipeMethod::Gutmann => 35,
        }
    }
}

/// 进度回调类型定义
pub type ProgressCallback = Box<dyn Fn(String, usize, usize, u32, u32) + Send + Sync>;

/// 安全删除文件或文件夹
pub fn secure_delete(file_path: &str, method: WipeMethod) -> Result<(), String> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("无法获取文件信息: {}", e))?;

    if metadata.is_dir() {
        // 如果是目录,递归删除所有文件
        secure_delete_directory(file_path, method)?;
    } else {
        // 如果是文件,直接删除
        secure_delete_file(file_path, method)?;
    }

    Ok(())
}

/// 带进度回调的安全删除
pub fn secure_delete_with_progress<F>(
    file_path: &str,
    method: WipeMethod,
    progress_callback: F,
) -> Result<(), String>
where
    F: Fn(String, usize, usize, u32, u32) + Send + Sync + 'static,
{
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("无法获取文件信息: {}", e))?;

    if metadata.is_dir() {
        secure_delete_directory_with_progress(file_path, method, &progress_callback)?;
    } else {
        let filename = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);
        secure_delete_file_with_progress(file_path, method, filename, 0, 1, &progress_callback)?;
    }

    Ok(())
}

/// 递归删除目录 - 使用深度优先后序遍历
fn secure_delete_directory(dir_path: &str, method: WipeMethod) -> Result<(), String> {
    let mut file_count = 0usize;

    // 递归函数：深度优先后序遍历
    fn delete_dir_recursive(
        path: &str,
        method: WipeMethod,
        file_count: &mut usize,
    ) -> Result<(), String> {
        // 读取目录内容
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("无法读取目录 {}: {}", path, e))?;

        for entry_result in entries {
            let entry = entry_result.map_err(|e| format!("读取目录项失败: {}", e))?;
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            let metadata = std::fs::symlink_metadata(&entry_path)
                .map_err(|e| format!("获取元数据失败 {}: {}", path_str, e))?;

            if metadata.is_symlink() {
                // 符号链接直接删除
                std::fs::remove_file(&path_str)
                    .map_err(|e| format!("删除符号链接失败 {}: {}", path_str, e))?;
                *file_count += 1;
            } else if metadata.is_dir() {
                // 递归处理子目录
                delete_dir_recursive(&path_str, method, file_count)?;
                // 子目录已清空，删除它
                std::fs::remove_dir(&path_str)
                    .map_err(|e| format!("删除目录失败 {}: {}", path_str, e))?;
            } else if metadata.is_file() {
                // 删除文件
                secure_delete_file(&path_str, method)?;
                *file_count += 1;
            }
        }

        Ok(())
    }

    // 执行递归删除
    delete_dir_recursive(dir_path, method, &mut file_count)?;

    // 删除根目录
    std::fs::remove_dir(dir_path)
        .map_err(|e| format!("删除根目录失败: {}", e))?;

    Ok(())
}

/// 快速统计目录中的文件和符号链接数量（不收集路径）
fn count_files_in_directory(dir_path: &str) -> Result<usize, String> {
    fn count_recursive(path: &str) -> Result<usize, String> {
        let mut count = 0usize;

        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("无法读取目录 {}: {}", path, e))?;

        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue, // 跳过无法读取的项
            };
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            let metadata = match std::fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(_) => continue, // 跳过无法获取元数据的项
            };

            if metadata.is_symlink() {
                count += 1;
            } else if metadata.is_dir() {
                count += count_recursive(&path_str)?;
            } else if metadata.is_file() {
                count += 1;
            }
        }

        Ok(count)
    }

    count_recursive(dir_path)
}

/// 递归删除目录(带进度) - 使用深度优先后序遍历
fn secure_delete_directory_with_progress<F>(
    dir_path: &str,
    method: WipeMethod,
    progress_callback: &F,
) -> Result<(), String>
where
    F: Fn(String, usize, usize, u32, u32) + Send + Sync,
{
    // 先快速统计文件总数
    let total_files = count_files_in_directory(dir_path)?;

    let mut completed_count = 0usize;

    // 递归函数：深度优先后序遍历
    fn delete_dir_recursive<F>(
        path: &str,
        method: WipeMethod,
        completed_count: &mut usize,
        total_files: usize,
        progress_callback: &F,
    ) -> Result<(), String>
    where
        F: Fn(String, usize, usize, u32, u32) + Send + Sync,
    {
        // 读取目录内容
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("无法读取目录 {}: {}", path, e))?;

        for entry_result in entries {
            let entry = entry_result.map_err(|e| format!("读取目录项失败: {}", e))?;
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            let metadata = std::fs::symlink_metadata(&entry_path)
                .map_err(|e| format!("获取元数据失败 {}: {}", path_str, e))?;

            if metadata.is_symlink() {
                // 符号链接直接删除
                let filename = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path_str);
                progress_callback(filename.to_string(), *completed_count, total_files, 1, 1);
                std::fs::remove_file(&path_str)
                    .map_err(|e| format!("删除符号链接失败 {}: {}", path_str, e))?;
                *completed_count += 1;
            } else if metadata.is_dir() {
                // 递归处理子目录
                delete_dir_recursive(&path_str, method, completed_count, total_files, progress_callback)?;
                // 子目录已清空，删除它
                std::fs::remove_dir(&path_str)
                    .map_err(|e| format!("删除目录失败 {}: {}", path_str, e))?;
            } else if metadata.is_file() {
                // 删除文件
                let filename = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path_str);

                secure_delete_file_with_progress(
                    &path_str,
                    method,
                    filename,
                    *completed_count,
                    total_files,
                    progress_callback,
                )?;
                *completed_count += 1;
            }
        }

        Ok(())
    }

    // 执行递归删除
    delete_dir_recursive(dir_path, method, &mut completed_count, total_files, progress_callback)?;

    // 删除根目录
    std::fs::remove_dir(dir_path)
        .map_err(|e| format!("删除根目录失败: {}", e))?;

    Ok(())
}

/// 安全删除单个文件
fn secure_delete_file(file_path: &str, method: WipeMethod) -> Result<(), String> {
    // 获取文件大小
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("无法获取文件信息: {}", e))?;

    let file_size = metadata.len();
    let passes = method.passes();

    // 执行多次覆写
    for pass in 0..passes {
        overwrite_file(file_path, file_size, pass, &method)?;
    }

    // 删除文件
    std::fs::remove_file(file_path)
        .map_err(|e| format!("删除文件失败: {}", e))?;

    Ok(())
}

/// 安全删除单个文件(带进度)
fn secure_delete_file_with_progress<F>(
    file_path: &str,
    method: WipeMethod,
    filename: &str,
    file_index: usize,
    total_files: usize,
    progress_callback: &F,
) -> Result<(), String>
where
    F: Fn(String, usize, usize, u32, u32) + Send + Sync,
{
    // 获取文件大小
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("无法获取文件信息: {}", e))?;

    let file_size = metadata.len();
    let passes = method.passes();

    // 执行多次覆写,每次调用进度回调
    for pass in 0..passes {
        progress_callback(filename.to_string(), file_index, total_files, pass + 1, passes);
        overwrite_file(file_path, file_size, pass, &method)?;
    }

    // 删除文件
    std::fs::remove_file(file_path)
        .map_err(|e| format!("删除文件失败: {}", e))?;

    Ok(())
}

fn overwrite_file(path: &str, size: u64, pass: u32, method: &WipeMethod) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("打开文件失败: {}", e))?;

    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("文件定位失败: {}", e))?;

    let mut rng = rand::thread_rng();
    let chunk_size = 4096;
    let mut buffer = vec![0u8; chunk_size];

    let mut remaining = size;
    while remaining > 0 {
        let write_size = std::cmp::min(remaining, chunk_size as u64) as usize;

        // 根据方法和当前轮次生成数据
        match method {
            WipeMethod::Zero => {
                buffer[..write_size].fill(0);
            }
            WipeMethod::Random => {
                rng.fill(&mut buffer[..write_size]);
            }
            WipeMethod::DoD5220 => {
                let pattern = match pass % 3 {
                    0 => 0x00,      // 第一次: 全零
                    1 => 0xFF,      // 第二次: 全一
                    _ => rng.gen(), // 其他: 随机
                };
                buffer[..write_size].fill(pattern);
            }
            WipeMethod::Gutmann => {
                // Gutmann 方法使用复杂的模式序列
                if pass < 4 {
                    rng.fill(&mut buffer[..write_size]);
                } else if pass < 31 {
                    // 使用特定模式
                    let patterns = [
                        0x55, 0xAA, 0x92, 0x49, 0x24,
                        0x00, 0x11, 0x22, 0x33, 0x44,
                        0x55, 0x66, 0x77, 0x88, 0x99,
                        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
                    ];
                    let pattern = patterns[(pass as usize - 4) % patterns.len()];
                    buffer[..write_size].fill(pattern);
                } else {
                    rng.fill(&mut buffer[..write_size]);
                }
            }
        }

        file.write_all(&buffer[..write_size])
            .map_err(|e| format!("写入数据失败: {}", e))?;

        remaining -= write_size as u64;
    }

    file.sync_all()
        .map_err(|e| format!("同步数据失败: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_secure_delete() {
        // 创建临时测试文件
        let test_file = "test_secure_delete.txt";
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"This is a test file for secure deletion").unwrap();
        drop(file);

        // 测试安全删除
        let result = secure_delete(test_file, WipeMethod::Random);
        assert!(result.is_ok());

        // 验证文件已被删除
        assert!(!std::path::Path::new(test_file).exists());
    }

    #[test]
    fn test_wipe_method_from_string() {
        assert!(matches!(WipeMethod::from_string("zero"), Ok(WipeMethod::Zero)));
        assert!(matches!(WipeMethod::from_string("random"), Ok(WipeMethod::Random)));
        assert!(matches!(WipeMethod::from_string("dod"), Ok(WipeMethod::DoD5220)));
        assert!(matches!(WipeMethod::from_string("gutmann"), Ok(WipeMethod::Gutmann)));
        assert!(WipeMethod::from_string("invalid").is_err());
    }
}
