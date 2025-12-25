import { lazy, Suspense } from 'react';
import { HashRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, theme, App as AntdApp, Spin } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import enUS from 'antd/locale/en_US';
import { useTranslation } from 'react-i18next';
import { SettingsProvider, useSettings } from './contexts/SettingsContext';
import { LicenseProvider } from './contexts/LicenseContext';
import Layout from './components/layout/Layout';
import PermissionDialog from './components/PermissionDialog';
import './App.css';

// Lazy load page components for better bundle splitting
const Dashboard = lazy(() => import('./pages/Dashboard'));
const ScanPage = lazy(() => import('./pages/ScanPage'));
const FileCleanup = lazy(() => import('./pages/FileCleanup'));
const SystemLogs = lazy(() => import('./pages/SystemLogs'));
const MemoryCleanup = lazy(() => import('./pages/MemoryCleanup'));
const NetworkCleanup = lazy(() => import('./pages/NetworkCleanup'));
const RegistryCleanup = lazy(() => import('./pages/RegistryCleanup'));
const TimestampModifier = lazy(() => import('./pages/TimestampModifier'));
const AntiAnalysis = lazy(() => import('./pages/AntiAnalysis'));
const DiskEncryption = lazy(() => import('./pages/DiskEncryption'));
const Settings = lazy(() => import('./pages/Settings'));

// Page loading fallback component
function PageLoading() {
  const { t } = useTranslation();
  return (
    <div className="h-full w-full flex items-center justify-center min-h-[400px]">
      <div className="text-center">
        <Spin size="large" />
        <p className="text-[var(--text-secondary)] mt-4">{t('common.loading')}</p>
      </div>
    </div>
  );
}

// Internal app component that can use SettingsContext
function AppContent() {
  const { t } = useTranslation();
  const { settings, loading } = useSettings();

  // Get system language
  const getSystemLanguage = (): string => {
    const browserLang = navigator.language || (navigator as any).userLanguage || 'zh-CN';
    if (browserLang.startsWith('zh')) {
      return 'zh-CN';
    } else if (browserLang.startsWith('en')) {
      return 'en-US';
    }
    return 'zh-CN';
  };

  // Select Ant Design locale based on settings
  const getAntdLocale = () => {
    let lang = settings.language;
    if (lang === 'auto') {
      lang = getSystemLanguage();
    }
    return lang === 'en-US' ? enUS : zhCN;
  };

  // Select theme algorithm based on settings
  const getThemeAlgorithm = () => {
    if (settings.theme === 'light') {
      return theme.defaultAlgorithm;
    } else if (settings.theme === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      return prefersDark ? theme.darkAlgorithm : theme.defaultAlgorithm;
    }
    return theme.darkAlgorithm;
  };

  // Get theme colors based on theme setting
  const getThemeColors = () => {
    const isLight = settings.theme === 'light' ||
      (settings.theme === 'auto' && !window.matchMedia('(prefers-color-scheme: dark)').matches);

    return {
      colorPrimary: isLight ? '#C67B2E' : '#D9943F',
      colorBgContainer: isLight ? '#FFFFFF' : '#2B2C30',
      colorBgElevated: isLight ? '#FFFFFF' : '#2B2C30',
      colorText: isLight ? '#1F2024' : '#E0E0E0',
      colorTextSecondary: isLight ? '#4B5563' : '#9CA3AF',
      colorBorder: isLight ? 'rgba(0, 0, 0, 0.1)' : 'rgba(255, 255, 255, 0.1)',
    };
  };

  if (loading) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-[var(--bg-primary)]">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-[var(--text-secondary)]">{t('common.loading')}</p>
        </div>
      </div>
    );
  }

  const themeColors = getThemeColors();

  return (
    <ConfigProvider
      locale={getAntdLocale()}
      theme={{
        algorithm: getThemeAlgorithm(),
        token: {
          colorPrimary: themeColors.colorPrimary,
          colorBgContainer: themeColors.colorBgContainer,
          colorBgElevated: themeColors.colorBgElevated,
          colorText: themeColors.colorText,
          colorTextSecondary: themeColors.colorTextSecondary,
          colorBorder: themeColors.colorBorder,
        },
      }}
    >
      <AntdApp>
        {/* Permission initialization dialog - shown on first use */}
        <PermissionDialog onComplete={() => {}} />

        <Router>
          <Layout>
            <Suspense fallback={<PageLoading />}>
              <Routes>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/scan" element={<ScanPage />} />
                <Route path="/file-cleanup" element={<FileCleanup />} />
                <Route path="/system-logs" element={<SystemLogs />} />
                <Route path="/memory-cleanup" element={<MemoryCleanup />} />
                <Route path="/network-cleanup" element={<NetworkCleanup />} />
                <Route path="/registry-cleanup" element={<RegistryCleanup />} />
                <Route path="/timestamp-modifier" element={<TimestampModifier />} />
                <Route path="/anti-analysis" element={<AntiAnalysis />} />
                <Route path="/disk-encryption" element={<DiskEncryption />} />
                <Route path="/settings" element={<Settings />} />
              </Routes>
            </Suspense>
          </Layout>
        </Router>
      </AntdApp>
    </ConfigProvider>
  );
}

function App() {
  return (
    <SettingsProvider>
      <LicenseProvider>
        <AppContent />
      </LicenseProvider>
    </SettingsProvider>
  );
}

export default App;
