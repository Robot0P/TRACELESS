import React, { createContext, useContext, useState, useEffect, useCallback, useRef } from 'react';
import * as tauriCore from '@tauri-apps/api/core';
import {
  type LicenseStatus,
  LicenseTier,
  type FeatureAccess,
  type FeatureKey,
  type LicenseValidationResult,
  DEFAULT_FREE_ACCESS,
  getLicenseErrorMessage,
} from '../types/license';

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

/**
 * License state provider component
 */
export const LicenseProvider: React.FC<LicenseProviderProps> = ({ children }) => {
  const [status, setStatus] = useState<LicenseStatus | null>(null);
  const [features, setFeatures] = useState<FeatureAccess>(DEFAULT_FREE_ACCESS);
  const [loading, setLoading] = useState(true);
  const [machineId, setMachineId] = useState('');
  const initialized = useRef(false);

  // Computed isPro value
  const isPro = status?.is_pro && status?.activated ? true : false;

  // Load machine ID
  const loadMachineId = useCallback(async () => {
    try {
      const id = await tauriCore.invoke<string>('get_machine_id');
      setMachineId(id);
    } catch (error) {
      console.error('Failed to get machine ID:', error);
    }
  }, []);

  // Load license status
  const loadLicenseStatus = useCallback(async () => {
    try {
      const licenseStatus = await tauriCore.invoke<LicenseStatus>('get_license_status');
      setStatus(licenseStatus);

      // Load feature access
      const featureAccess = await tauriCore.invoke<FeatureAccess>('get_feature_access');
      setFeatures(featureAccess);
    } catch (error) {
      console.error('Failed to load license status:', error);
      // Use default free access on error
      setStatus({
        tier: LicenseTier.Free,
        is_pro: false,
        activated: false,
        expires_at: null,
        days_remaining: null,
        machine_id: machineId,
        activation_date: null,
      });
      setFeatures(DEFAULT_FREE_ACCESS);
    }
  }, [machineId]);

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

  // Activate license
  const activateLicense = useCallback(async (key: string): Promise<{ success: boolean; message: string }> => {
    try {
      // First validate the key
      const validation = await tauriCore.invoke<LicenseValidationResult>('validate_license', { licenseKey: key });

      if (!validation.valid) {
        return {
          success: false,
          message: getLicenseErrorMessage(validation.error_code),
        };
      }

      // Activate the license
      const newStatus = await tauriCore.invoke<LicenseStatus>('activate_license', { licenseKey: key });
      setStatus(newStatus);

      // Reload feature access
      const featureAccess = await tauriCore.invoke<FeatureAccess>('get_feature_access');
      setFeatures(featureAccess);

      return {
        success: true,
        message: 'license.activationSuccess',
      };
    } catch (error) {
      const errorMessage = typeof error === 'string' ? error : String(error);
      return {
        success: false,
        message: getLicenseErrorMessage(errorMessage),
      };
    }
  }, []);

  // Deactivate license
  const deactivateLicense = useCallback(async () => {
    try {
      await tauriCore.invoke('deactivate_license');

      // Reload status
      await loadLicenseStatus();
    } catch (error) {
      console.error('Failed to deactivate license:', error);
      throw error;
    }
  }, [loadLicenseStatus]);

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
