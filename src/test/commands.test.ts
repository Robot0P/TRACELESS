import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Tauri Commands', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('File Operations', () => {
    it('should call secure_delete_file with correct parameters', async () => {
      const mockPath = '/tmp/test.txt';
      const mockMethod = 'dod';
      const mockResult = '文件已安全删除: /tmp/test.txt';

      vi.mocked(invoke).mockResolvedValueOnce(mockResult);

      const result = await invoke('secure_delete_file', {
        path: mockPath,
        method: mockMethod,
      });

      expect(invoke).toHaveBeenCalledWith('secure_delete_file', {
        path: mockPath,
        method: mockMethod,
      });
      expect(result).toBe(mockResult);
    });

    it('should call get_file_info with correct parameters', async () => {
      const mockPath = '/tmp/test.txt';
      const mockFileInfo = {
        path: mockPath,
        size: 1024,
        is_dir: false,
        modified: '2024-01-01T00:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockFileInfo);

      const result = await invoke('get_file_info', { path: mockPath });

      expect(invoke).toHaveBeenCalledWith('get_file_info', { path: mockPath });
      expect(result).toEqual(mockFileInfo);
    });

    it('should handle file operation errors', async () => {
      const mockPath = '/nonexistent/file.txt';
      const errorMessage = '路径验证失败: Path not found';

      vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage));

      await expect(invoke('get_file_info', { path: mockPath })).rejects.toThrow(
        errorMessage
      );
    });
  });

  describe('System Info Operations', () => {
    it('should call get_system_info_api', async () => {
      const mockSystemInfo = {
        os: 'macOS',
        version: '14.0',
        total_memory: 16384,
        used_memory: 8192,
        cpu_usage: 25.5,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockSystemInfo);

      const result = await invoke('get_system_info_api');

      expect(invoke).toHaveBeenCalledWith('get_system_info_api');
      expect(result).toEqual(mockSystemInfo);
    });

    it('should call get_network_speed_api', async () => {
      const mockNetworkSpeed = {
        download: 1.5,
        upload: 0.5,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockNetworkSpeed);

      const result = await invoke('get_network_speed_api');

      expect(invoke).toHaveBeenCalledWith('get_network_speed_api');
      expect(result).toEqual(mockNetworkSpeed);
    });
  });

  describe('Permission Operations', () => {
    it('should call check_admin_permission', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const result = await invoke('check_admin_permission');

      expect(invoke).toHaveBeenCalledWith('check_admin_permission');
      expect(result).toBe(true);
    });

    it('should call get_permission_status', async () => {
      const mockStatus = {
        is_admin: true,
        has_full_disk_access: true,
        has_authorization: true,
        has_system_privileges: false,
        platform: 'macOS',
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockStatus);

      const result = await invoke('get_permission_status');

      expect(invoke).toHaveBeenCalledWith('get_permission_status');
      expect(result).toEqual(mockStatus);
    });

    it('should call check_permission_initialized', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(false);

      const result = await invoke('check_permission_initialized');

      expect(invoke).toHaveBeenCalledWith('check_permission_initialized');
      expect(result).toBe(false);
    });
  });

  describe('Settings Operations', () => {
    it('should call load_settings', async () => {
      const mockSettings = {
        theme: 'dark',
        language: 'zh-CN',
        auto_scan_on_startup: false,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockSettings);

      const result = await invoke('load_settings');

      expect(invoke).toHaveBeenCalledWith('load_settings');
      expect(result).toEqual(mockSettings);
    });

    it('should call save_settings with correct parameters', async () => {
      const mockSettings = {
        theme: 'light',
        language: 'en-US',
        max_scan_depth: 15,
      };

      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await invoke('save_settings', { settings: mockSettings });

      expect(invoke).toHaveBeenCalledWith('save_settings', {
        settings: mockSettings,
      });
    });

    it('should call reset_settings', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await invoke('reset_settings');

      expect(invoke).toHaveBeenCalledWith('reset_settings');
    });
  });

  describe('Scan Operations', () => {
    it('should call perform_system_scan with scan type', async () => {
      const mockScanResult = {
        items: [],
        total_size: 0,
        scan_time: 1000,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockScanResult);

      const result = await invoke('perform_system_scan', { scanType: 'smart' });

      expect(invoke).toHaveBeenCalledWith('perform_system_scan', {
        scanType: 'smart',
      });
      expect(result).toEqual(mockScanResult);
    });

    it('should call cleanup_scan_items with selected items', async () => {
      const mockItems = ['/tmp/cache', '/var/log/old.log'];

      vi.mocked(invoke).mockResolvedValueOnce({ cleaned: 2, failed: 0 });

      const result = await invoke('cleanup_scan_items', { items: mockItems });

      expect(invoke).toHaveBeenCalledWith('cleanup_scan_items', {
        items: mockItems,
      });
    });
  });

  describe('Anti-Analysis Operations', () => {
    it('should call check_environment', async () => {
      const mockResult = {
        vm_detected: false,
        debugger_detected: false,
        sandbox_detected: false,
        forensic_tools_detected: false,
        details: [],
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockResult);

      const result = await invoke('check_environment');

      expect(invoke).toHaveBeenCalledWith('check_environment');
      expect(result).toEqual(mockResult);
    });
  });

  describe('Timestamp Operations', () => {
    it('should call get_file_timestamps', async () => {
      const mockPath = '/tmp/test.txt';
      const mockTimestamps = {
        created: '2024-01-01T00:00:00Z',
        modified: '2024-01-02T00:00:00Z',
        accessed: '2024-01-03T00:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockTimestamps);

      const result = await invoke('get_file_timestamps', { filePath: mockPath });

      expect(invoke).toHaveBeenCalledWith('get_file_timestamps', {
        filePath: mockPath,
      });
      expect(result).toEqual(mockTimestamps);
    });

    it('should call modify_file_timestamps', async () => {
      const mockPath = '/tmp/test.txt';
      const mockTimestamps = {
        modified: '2024-06-01T00:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValueOnce('文件时间戳已成功修改');

      const result = await invoke('modify_file_timestamps', {
        filePath: mockPath,
        timestamps: mockTimestamps,
      });

      expect(invoke).toHaveBeenCalledWith('modify_file_timestamps', {
        filePath: mockPath,
        timestamps: mockTimestamps,
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

      await expect(invoke('get_system_info_api')).rejects.toThrow(
        'Network error'
      );
    });

    it('should handle permission denied errors', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(
        new Error('Permission denied: Administrator privileges required')
      );

      await expect(invoke('clear_system_logs')).rejects.toThrow(
        'Permission denied'
      );
    });

    it('should handle path validation errors', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(
        new Error('路径验证失败: Path traversal detected')
      );

      await expect(
        invoke('secure_delete_file', {
          path: '../../../etc/passwd',
          method: 'dod',
        })
      ).rejects.toThrow('路径验证失败');
    });
  });
});
