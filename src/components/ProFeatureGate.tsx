import React from 'react';
import { useTranslation } from 'react-i18next';
import { useLicense } from '../contexts/LicenseContext';
import { type FeatureKey, isProFeature } from '../types/license';

interface ProFeatureGateProps {
  /** The feature key to check */
  feature: FeatureKey;
  /** Content to show when feature is accessible */
  children: React.ReactNode;
  /** Optional custom fallback content */
  fallback?: React.ReactNode;
  /** Show blur overlay style instead of replacing content */
  blurMode?: boolean;
  /** Callback when upgrade button is clicked */
  onUpgradeClick?: () => void;
}

/**
 * Component that gates content behind Pro license
 * Shows upgrade prompt for Pro features when user has Free license
 */
export const ProFeatureGate: React.FC<ProFeatureGateProps> = ({
  feature,
  children,
  fallback,
  blurMode = true,
  onUpgradeClick,
}) => {
  const { t } = useTranslation();
  const { canAccessFeature } = useLicense();

  // Free features are always accessible
  if (!isProFeature(feature)) {
    return <>{children}</>;
  }

  // Pro users can access all features
  if (canAccessFeature(feature)) {
    return <>{children}</>;
  }

  // Use custom fallback if provided
  if (fallback) {
    return <>{fallback}</>;
  }

  // Default upgrade prompt
  if (blurMode) {
    return (
      <div className="pro-feature-gate">
        <div className="pro-feature-gate__content">
          {children}
        </div>
        <div className="pro-feature-gate__overlay">
          <div className="pro-feature-gate__prompt">
            <div className="pro-feature-gate__icon">
              <svg
                viewBox="0 0 24 24"
                width="48"
                height="48"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                <path d="M9 12l2 2 4-4" />
              </svg>
            </div>
            <h3 className="pro-feature-gate__title">{t('license.proFeature')}</h3>
            <p className="pro-feature-gate__message">{t('license.upgradeMessage')}</p>
            <button
              className="pro-feature-gate__button"
              onClick={onUpgradeClick}
            >
              {t('license.activatePro')}
            </button>
          </div>
        </div>
        <style>{`
          .pro-feature-gate {
            position: relative;
            width: 100%;
            height: 100%;
            min-height: 200px;
            overflow: hidden;
          }

          .pro-feature-gate__content {
            filter: blur(4px);
            pointer-events: none;
            user-select: none;
            opacity: 0.6;
            height: 100%;
          }

          .pro-feature-gate__overlay {
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            background: var(--shadow-color, rgba(0, 0, 0, 0.5));
            backdrop-filter: blur(4px);
            z-index: 100;
            padding: 20px;
          }

          .pro-feature-gate__prompt {
            background: var(--bg-card, #2B2C30);
            border: 1px solid var(--accent-color, #D9943F);
            border-radius: 20px;
            padding: 32px 40px;
            text-align: center;
            max-width: 380px;
            width: 100%;
            box-shadow: 0 20px 60px var(--shadow-color, rgba(0, 0, 0, 0.5));
          }

          .pro-feature-gate__icon {
            color: var(--accent-color, #D9943F);
            margin-bottom: 16px;
          }

          .pro-feature-gate__icon svg {
            filter: drop-shadow(0 0 10px rgba(217, 148, 63, 0.4));
          }

          .pro-feature-gate__title {
            font-size: 1.5rem;
            font-weight: 700;
            margin: 0 0 12px 0;
            color: var(--text-primary, #E0E0E0);
          }

          .pro-feature-gate__message {
            font-size: 0.9rem;
            color: var(--text-secondary, #9CA3AF);
            margin: 0 0 24px 0;
            line-height: 1.6;
          }

          .pro-feature-gate__button {
            background: linear-gradient(135deg, var(--accent-color, #D9943F) 0%, var(--accent-hover, #c28538) 100%);
            color: var(--bg-primary, #1F2024);
            border: none;
            border-radius: 10px;
            padding: 14px 32px;
            font-size: 0.95rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            box-shadow: 0 4px 15px rgba(217, 148, 63, 0.3);
          }

          .pro-feature-gate__button:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(217, 148, 63, 0.5);
            filter: brightness(1.1);
          }

          .pro-feature-gate__button:active {
            transform: translateY(0);
          }
        `}</style>
      </div>
    );
  }

  // Non-blur mode - just show upgrade message
  return (
    <div className="pro-feature-locked">
      <div className="pro-feature-locked__content">
        <div className="pro-feature-locked__icon">
          <svg
            viewBox="0 0 24 24"
            width="32"
            height="32"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
        </div>
        <span className="pro-feature-locked__text">{t('license.proFeature')}</span>
        <button
          className="pro-feature-locked__button"
          onClick={onUpgradeClick}
        >
          {t('license.activatePro')}
        </button>
      </div>
      <style>{`
        .pro-feature-locked {
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 24px;
          background: var(--card-bg, #1a1a2e);
          border: 1px dashed var(--border-color, rgba(255, 255, 255, 0.2));
          border-radius: 12px;
        }

        .pro-feature-locked__content {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 12px;
        }

        .pro-feature-locked__icon {
          color: var(--text-secondary, rgba(255, 255, 255, 0.5));
        }

        .pro-feature-locked__text {
          font-size: 0.875rem;
          color: var(--text-secondary, rgba(255, 255, 255, 0.7));
        }

        .pro-feature-locked__button {
          background: var(--accent-color, #6366f1);
          color: white;
          border: none;
          border-radius: 6px;
          padding: 8px 16px;
          font-size: 0.75rem;
          cursor: pointer;
          transition: background 0.2s ease;
        }

        .pro-feature-locked__button:hover {
          background: var(--accent-hover, #5558e3);
        }
      `}</style>
    </div>
  );
};

/**
 * Badge component showing Pro status
 */
export const ProBadge: React.FC<{ className?: string }> = ({ className }) => {
  return (
    <span className={`pro-badge ${className || ''}`}>
      Pro
      <style>{`
        .pro-badge {
          display: inline-flex;
          align-items: center;
          padding: 2px 8px;
          font-size: 0.625rem;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
          color: white;
          border-radius: 4px;
          margin-left: 8px;
        }
      `}</style>
    </span>
  );
};

/**
 * Hook to check feature access
 */
export const useFeatureAccess = (feature: FeatureKey): boolean => {
  const { canAccessFeature } = useLicense();
  return canAccessFeature(feature);
};

export default ProFeatureGate;
