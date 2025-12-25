import React, { useState, useCallback, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { useLicense } from '../contexts/LicenseContext';
import {
  getLicenseTierName,
  formatExpirationDate,
  isExpiringSoon,
} from '../types/license';

interface LicenseActivationDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
}

/**
 * License activation/management dialog
 * 许可证激活对话框 - 不显示机器ID，激活时自动上传系统信息
 */
export const LicenseActivationDialog: React.FC<LicenseActivationDialogProps> = ({
  isOpen,
  onClose,
}) => {
  const { t } = useTranslation();
  const {
    status,
    isPro,
    isOnline,
    activateLicense,
    deactivateLicense,
  } = useLicense();

  const [licenseKey, setLicenseKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLicenseKey('');
      setError(null);
      setSuccess(null);
    }
  }, [isOpen]);

  // Format license key as user types
  const handleKeyChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    let value = e.target.value.toUpperCase();
    // Remove all non-alphanumeric characters except dashes
    value = value.replace(/[^A-Z0-9-]/g, '');
    // Remove existing dashes for formatting
    const raw = value.replace(/-/g, '');
    // Add dashes every 5 characters
    const parts = raw.match(/.{1,5}/g) || [];
    const formatted = parts.join('-');
    // Limit to 29 characters (25 chars + 4 dashes)
    setLicenseKey(formatted.slice(0, 29));
    setError(null);
  }, []);

  // Handle paste event
  const handlePaste = useCallback((e: React.ClipboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    const pastedText = e.clipboardData.getData('text');
    let value = pastedText.toUpperCase();
    // Remove all non-alphanumeric characters except dashes
    value = value.replace(/[^A-Z0-9-]/g, '');
    // Remove existing dashes for formatting
    const raw = value.replace(/-/g, '');
    // Add dashes every 5 characters
    const parts = raw.match(/.{1,5}/g) || [];
    const formatted = parts.join('-');
    // Limit to 29 characters (25 chars + 4 dashes)
    setLicenseKey(formatted.slice(0, 29));
    setError(null);
  }, []);

  // Activate license
  const handleActivate = useCallback(async () => {
    if (!licenseKey || licenseKey.length < 29) {
      setError(t('license.errors.INVALID_FORMAT'));
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await activateLicense(licenseKey);
      if (result.success) {
        setSuccess(t(result.message));
        setLicenseKey('');
      } else {
        setError(t(result.message));
      }
    } catch (err) {
      setError(t('license.errors.UNKNOWN'));
    } finally {
      setLoading(false);
    }
  }, [licenseKey, activateLicense, t]);

  // Deactivate license
  const handleDeactivate = useCallback(async () => {
    setLoading(true);
    try {
      await deactivateLicense();
      setSuccess(t('license.deactivateSuccess'));
    } catch (err) {
      setError(t('license.errors.UNKNOWN'));
    } finally {
      setLoading(false);
    }
  }, [deactivateLicense, t]);

  if (!isOpen) return null;

  const tierName = status ? getLicenseTierName(status.tier) : 'Free';
  const tierKey = tierName.toLowerCase();
  const expirationDate = status?.expires_at ? formatExpirationDate(status.expires_at) : null;
  const daysRemaining = status?.days_remaining;
  const expiringSoon = isExpiringSoon(daysRemaining ?? null);

  // Get translated tier name with fallback
  const translatedTierName = t(`license.${tierKey}`, { defaultValue: tierName });

  // Use createPortal to render dialog at document.body level
  return createPortal(
    <div className="license-dialog-overlay" onClick={onClose}>
      <div className="license-dialog" onClick={(e) => e.stopPropagation()}>
        <button className="license-dialog__close" onClick={onClose}>
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>

        <h2 className="license-dialog__title">{t('license.activation')}</h2>

        {/* Connection Status */}
        <div className="license-dialog__connection-status">
          <span className={`connection-dot ${isOnline ? 'online' : 'offline'}`}></span>
          <span>{isOnline ? t('license.serverConnected', { defaultValue: '许可证服务器连接成功' }) : t('license.serverDisconnected', { defaultValue: '许可证服务器连接失败' })}</span>
        </div>

        {/* Current License Status */}
        {status?.activated && (
          <div className="license-dialog__section">
            <label className="license-dialog__label">{t('license.currentStatus')}</label>
            <div className={`license-dialog__status ${expiringSoon ? 'expiring' : ''}`}>
              <div className="license-dialog__status-tier">
                <span className={`tier-badge tier-badge--${tierKey}`}>
                  {translatedTierName}
                </span>
                {isPro && <span className="pro-indicator">Pro</span>}
              </div>
              {expirationDate && (
                <div className="license-dialog__status-expiry">
                  <span>{t('license.expiresOn')}: {expirationDate}</span>
                  {daysRemaining !== null && (
                    <span className={expiringSoon ? 'warning' : ''}>
                      ({t('license.daysRemaining', { days: daysRemaining })})
                    </span>
                  )}
                </div>
              )}
            </div>
            <button
              className="license-dialog__deactivate-btn"
              onClick={handleDeactivate}
              disabled={loading}
            >
              {t('license.deactivate')}
            </button>
          </div>
        )}

        {/* License Key Input */}
        <div className="license-dialog__section">
          <label className="license-dialog__label">{t('license.enterKey')}</label>
          <div className="license-dialog__input-wrapper">
            <input
              type="text"
              className="license-dialog__input"
              value={licenseKey}
              onChange={handleKeyChange}
              onPaste={handlePaste}
              placeholder="XXXXX-XXXXX-XXXXX-XXXXX-XXXXX"
              disabled={loading}
              autoComplete="off"
              autoCorrect="off"
              autoCapitalize="characters"
              spellCheck={false}
            />
            <button
              type="button"
              className="license-dialog__paste-btn"
              onClick={async () => {
                try {
                  const text = await navigator.clipboard.readText();
                  let value = text.toUpperCase();
                  value = value.replace(/[^A-Z0-9-]/g, '');
                  const raw = value.replace(/-/g, '');
                  const parts = raw.match(/.{1,5}/g) || [];
                  const formatted = parts.join('-');
                  setLicenseKey(formatted.slice(0, 29));
                  setError(null);
                } catch (err) {
                  console.error('Failed to paste:', err);
                }
              }}
              disabled={loading}
              title={t('common.paste')}
            >
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2" />
                <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
              </svg>
            </button>
          </div>
          <p className="license-dialog__hint">{t('license.activationHint', { defaultValue: '输入许可证密钥后将自动绑定到此设备' })}</p>
        </div>

        {/* Error/Success Messages */}
        {error && (
          <div className="license-dialog__message license-dialog__message--error">
            {error}
          </div>
        )}
        {success && (
          <div className="license-dialog__message license-dialog__message--success">
            {success}
          </div>
        )}

        {/* Activate Button */}
        <button
          className="license-dialog__activate-btn"
          onClick={handleActivate}
          disabled={loading || licenseKey.length < 29 || !isOnline}
        >
          {loading ? t('common.loading') : t('license.activate')}
        </button>

        <style>{`
          .license-dialog-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: var(--shadow-color, rgba(0, 0, 0, 0.7));
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 9999;
            backdrop-filter: blur(8px);
            padding: 20px;
          }

          .license-dialog {
            background: var(--bg-card, #2B2C30);
            border: 1px solid var(--accent-color, #D9943F);
            border-radius: 20px;
            padding: 32px;
            width: 100%;
            max-width: 480px;
            max-height: calc(100vh - 40px);
            overflow-y: auto;
            position: relative;
            box-shadow: 0 20px 60px var(--shadow-color, rgba(0, 0, 0, 0.5));
          }

          .license-dialog__close {
            position: absolute;
            top: 16px;
            right: 16px;
            background: var(--bg-hover, rgba(255, 255, 255, 0.1));
            border: none;
            color: var(--text-secondary, #9CA3AF);
            cursor: pointer;
            padding: 8px;
            border-radius: 8px;
            transition: all 0.2s ease;
          }

          .license-dialog__close:hover {
            background: var(--bg-hover, rgba(255, 255, 255, 0.15));
            color: var(--text-primary, #E0E0E0);
          }

          .license-dialog__title {
            font-size: 1.5rem;
            font-weight: 700;
            margin: 0 0 16px 0;
            color: var(--text-primary, #E0E0E0);
          }

          .license-dialog__connection-status {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 24px;
            font-size: 0.875rem;
            color: var(--text-secondary, #9CA3AF);
          }

          .connection-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
          }

          .connection-dot.online {
            background: #22c55e;
            box-shadow: 0 0 8px rgba(34, 197, 94, 0.5);
          }

          .connection-dot.offline {
            background: #ef4444;
            box-shadow: 0 0 8px rgba(239, 68, 68, 0.5);
          }

          .license-dialog__section {
            margin-bottom: 24px;
          }

          .license-dialog__label {
            display: block;
            font-size: 0.875rem;
            font-weight: 500;
            margin-bottom: 8px;
            color: var(--text-secondary, #9CA3AF);
          }

          .license-dialog__hint {
            font-size: 0.75rem;
            color: var(--text-muted, #6B7280);
            margin: 8px 0 0 0;
          }

          .license-dialog__input-wrapper {
            display: flex;
            gap: 8px;
            align-items: center;
          }

          .license-dialog__input {
            flex: 1;
            background: var(--bg-secondary, rgba(255, 255, 255, 0.05));
            border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
            border-radius: 8px;
            padding: 12px 16px;
            font-family: 'SF Mono', Monaco, monospace;
            font-size: 1rem;
            color: var(--text-primary, #E0E0E0);
            text-align: center;
            letter-spacing: 2px;
            transition: border-color 0.2s ease;
            -webkit-user-select: text !important;
            user-select: text !important;
          }

          .license-dialog__paste-btn {
            background: var(--bg-secondary, rgba(255, 255, 255, 0.05));
            border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
            border-radius: 8px;
            padding: 12px;
            color: var(--text-secondary, #9CA3AF);
            cursor: pointer;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            justify-content: center;
          }

          .license-dialog__paste-btn:hover {
            background: var(--bg-hover, rgba(255, 255, 255, 0.1));
            color: var(--accent-color, #D9943F);
            border-color: var(--accent-color, #D9943F);
          }

          .license-dialog__paste-btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }

          .license-dialog__input:focus {
            outline: none;
            border-color: var(--accent-color, #D9943F);
          }

          .license-dialog__input::placeholder {
            color: var(--text-muted, #6B7280);
          }

          .license-dialog__status {
            background: var(--bg-secondary, rgba(255, 255, 255, 0.05));
            border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
            border-radius: 8px;
            padding: 16px;
          }

          .license-dialog__status.expiring {
            border-color: var(--warning-color, #FAAD14);
            background: rgba(250, 173, 20, 0.1);
          }

          .license-dialog__status-tier {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
          }

          .tier-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
          }

          .tier-badge--monthly { background: #3b82f6; color: white; }
          .tier-badge--quarterly { background: #8b5cf6; color: white; }
          .tier-badge--yearly { background: var(--warning-color, #FAAD14); color: white; }

          .pro-indicator {
            background: linear-gradient(135deg, var(--accent-color, #D9943F) 0%, var(--accent-hover, #c28538) 100%);
            color: var(--bg-primary, #1F2024);
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.625rem;
            font-weight: 600;
          }

          .license-dialog__status-expiry {
            font-size: 0.875rem;
            color: var(--text-secondary, #9CA3AF);
            display: flex;
            flex-direction: column;
            gap: 4px;
          }

          .license-dialog__status-expiry .warning {
            color: var(--warning-color, #FAAD14);
          }

          .license-dialog__deactivate-btn {
            margin-top: 12px;
            background: none;
            border: 1px solid rgba(239, 68, 68, 0.5);
            color: var(--danger-color, #F5222D);
            padding: 8px 16px;
            border-radius: 6px;
            font-size: 0.75rem;
            cursor: pointer;
            transition: all 0.2s ease;
          }

          .license-dialog__deactivate-btn:hover {
            background: rgba(239, 68, 68, 0.1);
            border-color: var(--danger-color, #F5222D);
          }

          .license-dialog__message {
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 16px;
            font-size: 0.875rem;
          }

          .license-dialog__message--error {
            background: rgba(239, 68, 68, 0.1);
            border: 1px solid rgba(239, 68, 68, 0.3);
            color: var(--danger-color, #F5222D);
          }

          .license-dialog__message--success {
            background: rgba(34, 197, 94, 0.1);
            border: 1px solid rgba(34, 197, 94, 0.3);
            color: var(--success-color, #52C41A);
          }

          .license-dialog__activate-btn {
            width: 100%;
            background: linear-gradient(135deg, var(--accent-color, #D9943F) 0%, var(--accent-hover, #c28538) 100%);
            color: var(--bg-primary, #1F2024);
            border: none;
            border-radius: 10px;
            padding: 14px 24px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            box-shadow: 0 4px 15px rgba(217, 148, 63, 0.3);
          }

          .license-dialog__activate-btn:hover:not(:disabled) {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(217, 148, 63, 0.5);
            filter: brightness(1.1);
          }

          .license-dialog__activate-btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }
        `}</style>
      </div>
    </div>,
    document.body
  );
};

export default LicenseActivationDialog;
