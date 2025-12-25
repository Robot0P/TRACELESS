import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import * as tauriCore from '@tauri-apps/api/core';
import i18n from '../i18n/config';

// 设置类型定义
export interface AppSettings {
  language: string;
  theme: string;
  notifications: boolean;
  auto_cleanup: boolean;
  secure_delete: boolean;
  delete_method: string;
  confirm_before_delete: boolean;
  auto_clean_logs: boolean;
  log_retention_days: number;
  auto_clean_memory: boolean;
  memory_clean_interval: number;
  default_memory_types: string[];
  auto_clear_dns_cache: boolean;
  auto_clear_network_history: boolean;
  auto_clean_registry: boolean;
  registry_clean_level: string;
  random_time_range_days: number;
  auto_anti_analysis_check: boolean;
  anti_analysis_check_interval: number;
  alert_on_threat_detected: boolean;
  remind_disk_encryption: boolean;
}

// 默认设置
const defaultSettings: AppSettings = {
  language: 'auto',  // 默认自动检测系统语言
  theme: 'auto',     // 默认自动检测系统主题
  notifications: true,
  auto_cleanup: false,
  secure_delete: true,
  delete_method: 'dod',
  confirm_before_delete: true,
  auto_clean_logs: false,
  log_retention_days: 30,
  auto_clean_memory: false,
  memory_clean_interval: 30,
  default_memory_types: ['clipboard', 'working_set'],
  auto_clear_dns_cache: false,
  auto_clear_network_history: false,
  auto_clean_registry: false,
  registry_clean_level: 'high',
  random_time_range_days: 365,
  auto_anti_analysis_check: true,
  anti_analysis_check_interval: 60,
  alert_on_threat_detected: true,
  remind_disk_encryption: true,
};

// Context 类型
interface SettingsContextType {
  settings: AppSettings;
  loading: boolean;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
  saveSettings: () => Promise<void>;
  resetSettings: () => Promise<void>;
  reloadSettings: () => Promise<void>;
  markAsSaved: () => void;
  hasChanges: boolean;
}

const SettingsContext = createContext<SettingsContextType | null>(null);

export const useSettings = () => {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within SettingsProvider');
  }
  return context;
};

interface SettingsProviderProps {
  children: React.ReactNode;
}

export const SettingsProvider: React.FC<SettingsProviderProps> = ({ children }) => {
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [originalSettings, setOriginalSettings] = useState<AppSettings>(defaultSettings);
  const [loading, setLoading] = useState(true);
  const [hasChanges, setHasChanges] = useState(false);
  const initialized = useRef(false);

  // 应用主题
  const applyTheme = useCallback((theme: string) => {
    const root = document.documentElement;
    const body = document.body;

    if (theme === 'light') {
      root.classList.add('light-theme');
      root.classList.remove('dark-theme');
      body.classList.add('light-theme');
      body.classList.remove('dark-theme');
    } else if (theme === 'dark') {
      root.classList.add('dark-theme');
      root.classList.remove('light-theme');
      body.classList.add('dark-theme');
      body.classList.remove('light-theme');
    } else {
      // auto - 根据系统偏好
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (prefersDark) {
        root.classList.add('dark-theme');
        root.classList.remove('light-theme');
        body.classList.add('dark-theme');
        body.classList.remove('light-theme');
      } else {
        root.classList.add('light-theme');
        root.classList.remove('dark-theme');
        body.classList.add('light-theme');
        body.classList.remove('dark-theme');
      }
    }
  }, []);

  // 获取系统语言
  const getSystemLanguage = useCallback((): string => {
    const browserLang = navigator.language || (navigator as any).userLanguage || 'zh-CN';
    // 简化语言代码，支持 zh/en 等
    if (browserLang.startsWith('zh')) {
      return 'zh-CN';
    } else if (browserLang.startsWith('en')) {
      return 'en-US';
    }
    return 'zh-CN'; // 默认中文
  }, []);

  // 应用语言
  const applyLanguage = useCallback((language: string) => {
    let targetLang = language;

    // 如果是 auto 模式，检测系统语言
    if (language === 'auto') {
      targetLang = getSystemLanguage();
    }

    if (i18n.language !== targetLang) {
      i18n.changeLanguage(targetLang);
    }
  }, [getSystemLanguage]);

  // 从后端加载设置
  const loadSettings = useCallback(async () => {
    setLoading(true);
    try {
      const loadedSettings = await tauriCore.invoke<AppSettings>('load_settings');
      setSettings(loadedSettings);
      setOriginalSettings(loadedSettings);
      setHasChanges(false);

      // 应用主题和语言
      applyTheme(loadedSettings.theme);
      applyLanguage(loadedSettings.language);
    } catch (error) {
      // 使用默认设置
      setSettings(defaultSettings);
      setOriginalSettings(defaultSettings);
      applyTheme(defaultSettings.theme);
      applyLanguage(defaultSettings.language);
    } finally {
      setLoading(false);
    }
  }, [applyTheme, applyLanguage]);

  // 初始化加载设置
  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true;
      loadSettings();
    }
  }, [loadSettings]);

  // 监听系统主题变化（仅在 auto 模式下）
  useEffect(() => {
    if (settings.theme === 'auto') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => applyTheme('auto');
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }
  }, [settings.theme, applyTheme]);

  // 检测设置变化
  useEffect(() => {
    const changed = JSON.stringify(settings) !== JSON.stringify(originalSettings);
    setHasChanges(changed);
  }, [settings, originalSettings]);

  // 更新设置（本地状态）
  const updateSettings = useCallback((newSettings: Partial<AppSettings>) => {
    setSettings(prev => {
      const updated = { ...prev, ...newSettings };

      // 立即应用主题变化
      if (newSettings.theme !== undefined && newSettings.theme !== prev.theme) {
        applyTheme(newSettings.theme);
      }

      // 立即应用语言变化
      if (newSettings.language !== undefined && newSettings.language !== prev.language) {
        applyLanguage(newSettings.language);
      }

      return updated;
    });
  }, [applyTheme, applyLanguage]);

  // 保存设置到后端
  const saveSettings = useCallback(async () => {
    try {
      await tauriCore.invoke('save_settings', { settings });
      setOriginalSettings({ ...settings });
      setHasChanges(false);
    } catch (error) {
      throw error;
    }
  }, [settings]);

  // 重置设置
  const resetSettings = useCallback(async () => {
    try {
      const resetResult = await tauriCore.invoke<AppSettings>('reset_settings');
      setSettings(resetResult);
      setOriginalSettings(resetResult);
      setHasChanges(false);

      // 应用主题和语言
      applyTheme(resetResult.theme);
      applyLanguage(resetResult.language);
    } catch (error) {
      throw error;
    }
  }, [applyTheme, applyLanguage]);

  // 重新加载设置
  const reloadSettings = useCallback(async () => {
    await loadSettings();
  }, [loadSettings]);

  // 标记设置已保存（不调用 invoke，仅更新状态）
  const markAsSaved = useCallback(() => {
    setOriginalSettings({ ...settings });
    setHasChanges(false);
  }, [settings]);

  const value: SettingsContextType = {
    settings,
    loading,
    updateSettings,
    saveSettings,
    resetSettings,
    reloadSettings,
    markAsSaved,
    hasChanges,
  };

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
};

export default SettingsContext;
