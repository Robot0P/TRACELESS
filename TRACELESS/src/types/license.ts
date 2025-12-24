/**
 * License tier types
 */
export const LicenseTier = {
  Free: 0,
  Monthly: 1,
  Quarterly: 2,
  Yearly: 3,
} as const;

export type LicenseTier = (typeof LicenseTier)[keyof typeof LicenseTier];

/**
 * Get display name for license tier
 */
export function getLicenseTierName(tier: LicenseTier): string {
  switch (tier) {
    case LicenseTier.Free:
      return 'Free';
    case LicenseTier.Monthly:
      return 'Monthly';
    case LicenseTier.Quarterly:
      return 'Quarterly';
    case LicenseTier.Yearly:
      return 'Yearly';
    default:
      return 'Unknown';
  }
}

/**
 * Get duration in days for license tier
 */
export function getLicenseTierDays(tier: LicenseTier): number {
  switch (tier) {
    case LicenseTier.Free:
      return 0;
    case LicenseTier.Monthly:
      return 30;
    case LicenseTier.Quarterly:
      return 90;
    case LicenseTier.Yearly:
      return 365;
    default:
      return 0;
  }
}

/**
 * License status information
 */
export interface LicenseStatus {
  /** Current license tier */
  tier: LicenseTier;
  /** Whether this is a Pro license */
  is_pro: boolean;
  /** Whether a license is currently activated */
  activated: boolean;
  /** Expiration timestamp (Unix timestamp in seconds) */
  expires_at: number | null;
  /** Days remaining until expiration */
  days_remaining: number | null;
  /** Machine ID this license is bound to */
  machine_id: string;
  /** Activation timestamp (Unix timestamp in seconds) */
  activation_date: number | null;
}

/**
 * License validation result
 */
export interface LicenseValidationResult {
  /** Whether the license is valid */
  valid: boolean;
  /** Error code if validation failed */
  error_code: string | null;
  /** Human-readable error message */
  error_message: string | null;
  /** License info if validation succeeded */
  license_info: LicenseStatus | null;
}

/**
 * Feature access rights based on license tier
 */
export interface FeatureAccess {
  /** Scan functionality - Free */
  scan: boolean;
  /** File shredder - Free */
  file_shredder: boolean;
  /** System logs cleanup - Pro */
  system_logs: boolean;
  /** Memory cleanup - Pro */
  memory_cleanup: boolean;
  /** Network cleanup - Pro */
  network_cleanup: boolean;
  /** Registry/privacy cleanup - Pro */
  registry_cleanup: boolean;
  /** Timestamp modifier - Pro */
  timestamp_modifier: boolean;
  /** Anti-analysis detection - Pro */
  anti_analysis: boolean;
  /** Disk encryption management - Pro */
  disk_encryption: boolean;
  /** Scheduled tasks - Pro */
  scheduled_tasks: boolean;
}

/**
 * Feature key type
 */
export type FeatureKey = keyof FeatureAccess;

/**
 * List of all features
 */
export const ALL_FEATURES: FeatureKey[] = [
  'scan',
  'file_shredder',
  'system_logs',
  'memory_cleanup',
  'network_cleanup',
  'registry_cleanup',
  'timestamp_modifier',
  'anti_analysis',
  'disk_encryption',
  'scheduled_tasks',
];

/**
 * List of free features
 */
export const FREE_FEATURES: FeatureKey[] = ['scan', 'file_shredder', 'timestamp_modifier', 'anti_analysis'];

/**
 * List of Pro features
 */
export const PRO_FEATURES: FeatureKey[] = [
  'system_logs',
  'memory_cleanup',
  'network_cleanup',
  'registry_cleanup',
  'disk_encryption',
  'scheduled_tasks',
];

/**
 * Check if a feature is a Pro feature
 */
export function isProFeature(feature: FeatureKey): boolean {
  return PRO_FEATURES.includes(feature);
}

/**
 * License error codes
 */
export const LicenseErrorCode = {
  InvalidFormat: 'INVALID_FORMAT',
  InvalidChecksum: 'INVALID_CHECKSUM',
  InvalidSignature: 'INVALID_SIGNATURE',
  MachineMismatch: 'MACHINE_MISMATCH',
  Expired: 'EXPIRED',
  Unknown: 'UNKNOWN',
} as const;

export type LicenseErrorCode = (typeof LicenseErrorCode)[keyof typeof LicenseErrorCode];

/**
 * Get user-friendly error message for license error code
 */
export function getLicenseErrorMessage(code: string | null): string {
  switch (code) {
    case LicenseErrorCode.InvalidFormat:
      return 'license.errors.INVALID_FORMAT';
    case LicenseErrorCode.InvalidChecksum:
      return 'license.errors.INVALID_CHECKSUM';
    case LicenseErrorCode.InvalidSignature:
      return 'license.errors.INVALID_SIGNATURE';
    case LicenseErrorCode.MachineMismatch:
      return 'license.errors.MACHINE_MISMATCH';
    case LicenseErrorCode.Expired:
      return 'license.errors.EXPIRED';
    default:
      return 'license.errors.UNKNOWN';
  }
}

/**
 * Format expiration date
 */
export function formatExpirationDate(timestamp: number | null): string {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString();
}

/**
 * Check if license is expiring soon (within 7 days)
 */
export function isExpiringSoon(daysRemaining: number | null): boolean {
  if (daysRemaining === null) return false;
  return daysRemaining <= 7 && daysRemaining > 0;
}

/**
 * Default feature access for free tier
 */
export const DEFAULT_FREE_ACCESS: FeatureAccess = {
  scan: true,
  file_shredder: true,
  system_logs: false,
  memory_cleanup: false,
  network_cleanup: false,
  registry_cleanup: false,
  timestamp_modifier: true,
  anti_analysis: true,
  disk_encryption: false,
  scheduled_tasks: false,
};

/**
 * Full feature access for Pro tier
 */
export const FULL_PRO_ACCESS: FeatureAccess = {
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
};
