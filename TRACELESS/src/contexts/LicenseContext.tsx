import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import * as tauriCore from '@tauri-apps/api/core';
import {
  type LicenseStatus,
  LicenseTier,
  type FeatureAccess,
  type FeatureKey,
  DEFAULT_FREE_ACCESS,
  getLicenseErrorMessage,
} from '../types/license';
import { supabase, isSupabaseConfigured } from '../config/supabase';
import {
  generateSecureParams,
  integrityCheck,
  detectDevTools,
  disableConsoleInProduction,
  generateClientFingerprint,
  obfuscateString,
  deobfuscateString
} from '../utils/security';

// Context 类型
interface LicenseContextType {
  /** Current license status */
  status: LicenseStatus | null;
  /** Feature access based on current license */
  features: FeatureAccess;
  /** Loading state */
  loading: boolean;
  /** Whether user has Pro license */
  isPro: boolean;
  /** Machine ID for license binding */
  machineId: string;
  /** Whether online verification is available */
  isOnline: boolean;
  /** Activate a license key */
  activateLicense: (key: string) => Promise<{ success: boolean; message: string }>;
  /** Deactivate current license */
  deactivateLicense: () => Promise<void>;
  /** Check if a specific feature is accessible */
  canAccessFeature: (feature: FeatureKey) => boolean;
  /** Refresh license status */
  refreshStatus: () => Promise<void>;
}

const LicenseContext = createContext<LicenseContextType | null>(null);

/**
 * Hook to access license context
 */
export const useLicense = () => {
  const context = useContext(LicenseContext);
  if (!context) {
    throw new Error('useLicense must be used within LicenseProvider');
  }
  return context;
};

interface LicenseProviderProps {
  children: React.ReactNode;
}

// 本地缓存的许可证密钥存储key
const LICENSE_CACHE_KEY = 'traceless_license_cache';

interface LicenseCache {
  license_key: string;
  license_info: LicenseStatus;
  cached_at: number;
  fingerprint?: string; // 客户端指纹
}

/**
 * License state provider component - 使用 Supabase 在线验证
 */
export const LicenseProvider: React.FC<LicenseProviderProps> = ({ children }) => {
  const [status, setStatus] = useState<LicenseStatus | null>(null);
  const [features, setFeatures] = useState<FeatureAccess>(DEFAULT_FREE_ACCESS);
  const [loading, setLoading] = useState(true);
  const [machineId, setMachineId] = useState('');
  const [isOnline, setIsOnline] = useState(true);
  const initialized = useRef(false);
  const devToolsCheckInterval = useRef<number | null>(null);

  // 在生产环境禁用控制台
  useEffect(() => {
    disableConsoleInProduction();
  }, []);

  // 定期检测开发者工具
  useEffect(() => {
    if (import.meta.env.PROD) {
      devToolsCheckInterval.current = window.setInterval(() => {
        if (detectDevTools()) {
          // 开发者工具打开时，清除敏感数据
          setStatus(null);
          setFeatures(DEFAULT_FREE_ACCESS);
        }
      }, 5000);

      return () => {
        if (devToolsCheckInterval.current) {
          clearInterval(devToolsCheckInterval.current);
        }
      };
    }
  }, []);

  // Computed isPro value
  const isPro = status?.is_pro && status?.activated ? true : false;

  // 从 Supabase 响应转换为 LicenseStatus
  const convertToLicenseStatus = useCallback((
    info: {
      tier: number;
      tier_name: string;
      is_pro: boolean;
      activated: boolean;
      activated_at: string | null;
      expires_at: string | null;
      days_remaining: number | null;
      features: FeatureAccess;
    },
    currentMachineId: string
  ): LicenseStatus => {
    return {
      tier: info.tier as LicenseTier,
      is_pro: info.is_pro,
      activated: info.activated,
      expires_at: info.expires_at ? new Date(info.expires_at).getTime() / 1000 : null,
      days_remaining: info.days_remaining,
      machine_id: currentMachineId,
      activation_date: info.activated_at ? new Date(info.activated_at).getTime() / 1000 : null,
    };
  }, []);

  // Load machine ID
  const loadMachineId = useCallback(async () => {
    try {
      const id = await tauriCore.invoke<string>('get_machine_id');
      setMachineId(id);
      return id;
    } catch (error) {
      console.error('Failed to get machine ID:', error);
      return '';
    }
  }, []);

  // 从本地缓存加载许可证 (带混淆和指纹验证)
  const loadFromCache = useCallback((): LicenseCache | null => {
    try {
      const cached = localStorage.getItem(LICENSE_CACHE_KEY);
      if (cached) {
        // 反混淆缓存数据
        const deobfuscated = deobfuscateString(cached);
        const data = JSON.parse(deobfuscated) as LicenseCache;

        // 验证客户端指纹是否匹配
        const currentFingerprint = generateClientFingerprint();
        if (data.fingerprint && data.fingerprint !== currentFingerprint) {
          // 指纹不匹配，可能是缓存被复制到其他设备
          console.warn('Client fingerprint mismatch, clearing cache');
          localStorage.removeItem(LICENSE_CACHE_KEY);
          return null;
        }

        // 缓存有效期 24 小时
        if (Date.now() - data.cached_at < 24 * 60 * 60 * 1000) {
          return data;
        }
      }
    } catch (error) {
      console.error('Failed to load license from cache:', error);
      // 如果解析失败，可能是旧格式或被篡改
      localStorage.removeItem(LICENSE_CACHE_KEY);
    }
    return null;
  }, []);

  // 保存到本地缓存 (带混淆)
  const saveToCache = useCallback((license_key: string, license_info: LicenseStatus) => {
    try {
      const cache: LicenseCache = {
        license_key,
        license_info,
        cached_at: Date.now(),
        fingerprint: generateClientFingerprint(),
      };
      // 混淆缓存数据
      const obfuscated = obfuscateString(JSON.stringify(cache));
      localStorage.setItem(LICENSE_CACHE_KEY, obfuscated);
    } catch (error) {
      console.error('Failed to save license to cache:', error);
    }
  }, []);

  // 清除本地缓存
  const clearCache = useCallback(() => {
    try {
      localStorage.removeItem(LICENSE_CACHE_KEY);
    } catch (error) {
      console.error('Failed to clear license cache:', error);
    }
  }, []);

  // 在线验证许可证 - 使用 Supabase 官方客户端
  const verifyLicenseOnline = useCallback(async (
    licenseKey: string,
    currentMachineId: string
  ): Promise<{ valid: boolean; license_info?: LicenseStatus; features?: FeatureAccess; error_code?: string; error_message?: string }> => {
    if (!isSupabaseConfigured()) {
      console.warn('Supabase not configured, using offline mode');
      return { valid: false, error_code: 'SUPABASE_NOT_CONFIGURED', error_message: '在线验证未配置' };
    }

    try {
      const { data, error } = await supabase.rpc('verify_license', {
        p_license_key: licenseKey,
        p_machine_id: currentMachineId,
      });

      if (error) {
        console.error('Supabase RPC error:', error);
        setIsOnline(false);
        return { valid: false, error_code: 'RPC_ERROR', error_message: error.message };
      }

      setIsOnline(true);

      const result = data as {
        valid: boolean;
        error_code?: string;
        error_message?: string;
        license_info?: {
          tier: number;
          tier_name: string;
          is_pro: boolean;
          activated: boolean;
          activated_at: string | null;
          expires_at: string | null;
          days_remaining: number | null;
          features: FeatureAccess;
        };
      };

      if (result.valid && result.license_info) {
        const licenseStatus = convertToLicenseStatus(result.license_info, currentMachineId);
        return {
          valid: true,
          license_info: licenseStatus,
          features: result.license_info.features,
        };
      } else {
        return {
          valid: false,
          error_code: result.error_code,
          error_message: result.error_message,
        };
      }
    } catch (error) {
      console.error('Online verification failed:', error);
      setIsOnline(false);
      return { valid: false, error_code: 'NETWORK_ERROR', error_message: '网络连接失败' };
    }
  }, [convertToLicenseStatus]);

  // Load license status - 优先在线验证，离线时使用缓存
  const loadLicenseStatus = useCallback(async () => {
    const currentMachineId = machineId || await loadMachineId();
    const cache = loadFromCache();

    if (cache) {
      // 尝试在线验证缓存的许可证
      const onlineResult = await verifyLicenseOnline(cache.license_key, currentMachineId);

      if (onlineResult.valid && onlineResult.license_info) {
        setStatus(onlineResult.license_info);
        setFeatures(onlineResult.features || DEFAULT_FREE_ACCESS);
        saveToCache(cache.license_key, onlineResult.license_info);
        return;
      } else if (isOnline) {
        // 在线但验证失败（过期/撤销等），清除缓存
        clearCache();
      } else {
        // 离线模式，使用缓存
        setStatus(cache.license_info);
        // 从缓存的许可证状态推断功能访问权限
        if (cache.license_info.is_pro && cache.license_info.activated) {
          setFeatures({
            scan: true,
            file_shredder: true,
            system_logs: true,
            memory_cleanup: true,
            network_cleanup: true,
            registry_cleanup: true,
            timestamp_modifier: true,
            anti_analysis: true,
            disk_encryption: true,
            scheduled_tasks: true,
          });
        } else {
          setFeatures(DEFAULT_FREE_ACCESS);
        }
        return;
      }
    }

    // 没有缓存，使用免费版本
    setStatus({
      tier: LicenseTier.Free,
      is_pro: false,
      activated: false,
      expires_at: null,
      days_remaining: null,
      machine_id: currentMachineId,
      activation_date: null,
    });
    setFeatures(DEFAULT_FREE_ACCESS);
  }, [machineId, loadMachineId, loadFromCache, verifyLicenseOnline, isOnline, saveToCache, clearCache]);

  // Initialize
  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true;
      const init = async () => {
        setLoading(true);
        await loadMachineId();
        await loadLicenseStatus();
        setLoading(false);
      };
      init();
    }
  }, [loadMachineId, loadLicenseStatus]);

  // Activate license - 使用安全 API (带签名验证)
  const activateLicense = useCallback(async (key: string): Promise<{ success: boolean; message: string }> => {
    const currentMachineId = machineId || await loadMachineId();

    if (!isSupabaseConfigured()) {
      return {
        success: false,
        message: '在线验证服务未配置，请联系管理员',
      };
    }

    // 客户端完整性检查
    if (!integrityCheck()) {
      return {
        success: false,
        message: '客户端完整性校验失败',
      };
    }

    try {
      // 获取系统信息
      let osInfo = 'Unknown';
      let osVersion = 'Unknown';
      let hostname = 'Unknown';

      try {
        const sysInfo = await tauriCore.invoke<{ os: string; version: string }>('get_system_info_api');
        osInfo = sysInfo.os;
        osVersion = sysInfo.version;
      } catch (e) {
        console.warn('Failed to get system info:', e);
      }

      try {
        hostname = await tauriCore.invoke<string>('get_hostname_api');
      } catch (e) {
        console.warn('Failed to get hostname:', e);
      }

      // 生成安全请求参数 (时间戳、nonce、签名)
      const secureParams = await generateSecureParams(key, currentMachineId);

      // 使用安全激活 API
      const { data, error } = await supabase.rpc('secure_activate_license', {
        p_license_key: key,
        p_machine_id: currentMachineId,
        p_os_info: osInfo,
        p_os_version: osVersion,
        p_hostname: hostname,
        p_timestamp: secureParams.timestamp,
        p_nonce: secureParams.nonce,
        p_signature: secureParams.signature,
      });

      if (error) {
        console.error('Supabase RPC error:', error);
        setIsOnline(false);
        return {
          success: false,
          message: error.message || '网络连接失败',
        };
      }

      setIsOnline(true);

      const result = data as {
        success: boolean;
        error_code?: string;
        error_message?: string;
        license_info?: {
          tier: number;
          tier_name: string;
          is_pro: boolean;
          activated: boolean;
          activated_at: string | null;
          expires_at: string | null;
          days_remaining: number | null;
          features: FeatureAccess;
        };
      };

      if (result.success && result.license_info) {
        const licenseStatus = convertToLicenseStatus(result.license_info, currentMachineId);
        setStatus(licenseStatus);
        setFeatures(result.license_info.features);
        saveToCache(key, licenseStatus);

        return {
          success: true,
          message: 'license.activationSuccess',
        };
      } else {
        return {
          success: false,
          message: getLicenseErrorMessage(result.error_code ?? null) || result.error_message || '激活失败',
        };
      }
    } catch (error) {
      console.error('License activation failed:', error);
      setIsOnline(false);
      return {
        success: false,
        message: '网络连接失败，请检查网络后重试',
      };
    }
  }, [machineId, loadMachineId, convertToLicenseStatus, saveToCache]);

  // Deactivate license - 使用 Supabase 官方客户端
  const deactivateLicense = useCallback(async () => {
    const cache = loadFromCache();
    if (!cache) {
      clearCache();
      await loadLicenseStatus();
      return;
    }

    const currentMachineId = machineId || await loadMachineId();

    if (isSupabaseConfigured()) {
      try {
        await supabase.rpc('deactivate_license', {
          p_license_key: cache.license_key,
          p_machine_id: currentMachineId,
        });
      } catch (error) {
        console.error('Failed to deactivate license online:', error);
      }
    }

    // 清除本地缓存
    clearCache();

    // 重新加载状态
    await loadLicenseStatus();
  }, [machineId, loadMachineId, loadFromCache, clearCache, loadLicenseStatus]);

  // Check if feature is accessible
  const canAccessFeature = useCallback((feature: FeatureKey): boolean => {
    return features[feature] ?? false;
  }, [features]);

  // Refresh status
  const refreshStatus = useCallback(async () => {
    await loadLicenseStatus();
  }, [loadLicenseStatus]);

  const value: LicenseContextType = {
    status,
    features,
    loading,
    isPro,
    machineId,
    isOnline,
    activateLicense,
    deactivateLicense,
    canAccessFeature,
    refreshStatus,
  };

  return (
    <LicenseContext.Provider value={value}>
      {children}
    </LicenseContext.Provider>
  );
};

export default LicenseContext;
