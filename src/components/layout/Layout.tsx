import React, { useState, useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  Home, Trash2, FileText, Cpu, Settings, Activity,
  Database, Clock, Eye, ChevronRight, ChevronLeft,
  HardDrive, Shield
} from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import AboutDialog from '../AboutDialog';
import LicenseActivationDialog from '../LicenseActivation';
import { useLicense } from '../../contexts/LicenseContext';
import { PRO_FEATURES, type FeatureKey } from '../../types/license';

interface LayoutProps {
  children: React.ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const navigate = useNavigate();
  const location = useLocation();
  const { t, i18n } = useTranslation();
  const { isPro, status } = useLicense();
  const [hoveredItem, setHoveredItem] = useState<string | null>(null);
  const [showAboutDialog, setShowAboutDialog] = useState(false);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);
  const [isCollapsed, setIsCollapsed] = useState(() => {
    const saved = localStorage.getItem('sidebar-collapsed');
    return saved ? JSON.parse(saved) : false;
  });

  useEffect(() => {
    localStorage.setItem('sidebar-collapsed', JSON.stringify(isCollapsed));
  }, [isCollapsed]);

  useEffect(() => {
    const unlistenAbout = listen('menu-about', () => {
      setShowAboutDialog(true);
    });
    return () => {
      unlistenAbout.then(fn => fn());
    };
  }, []);

  // Map paths to feature keys for Pro badge display
  const pathToFeature: Record<string, FeatureKey | null> = {
    '/dashboard': null,
    '/file-cleanup': 'file_shredder',
    '/system-logs': 'system_logs',
    '/memory-cleanup': 'memory_cleanup',
    '/network-cleanup': 'network_cleanup',
    '/disk-encryption': 'disk_encryption',
    '/registry-cleanup': 'registry_cleanup',
    '/timestamp-modifier': 'timestamp_modifier',
    '/anti-analysis': 'anti_analysis',
    '/settings': null,
  };

  const isProFeaturePath = (path: string): boolean => {
    const feature = pathToFeature[path];
    return feature !== null && PRO_FEATURES.includes(feature);
  };

  const menuItems = [
    { path: '/dashboard', icon: <Home size={20} />, label: t('nav.dashboard') },
    { path: '/file-cleanup', icon: <Trash2 size={20} />, label: t('nav.fileCleanup') },
    { path: '/system-logs', icon: <FileText size={20} />, label: t('nav.systemLogs') },
    { path: '/memory-cleanup', icon: <Cpu size={20} />, label: t('nav.memoryCleanup') },
    { path: '/network-cleanup', icon: <Activity size={20} />, label: t('nav.networkCleanup') },
    { path: '/disk-encryption', icon: <HardDrive size={20} />, label: t('nav.diskEncryption') },
    { path: '/registry-cleanup', icon: <Database size={20} />, label: t('nav.registryCleanup') },
    { path: '/timestamp-modifier', icon: <Clock size={20} />, label: t('nav.timestampModifier') },
    { path: '/anti-analysis', icon: <Eye size={20} />, label: t('nav.antiAnalysis') },
  ];

  const handleNavigation = (path: string) => {
    navigate(path);
  };

  // --- SidebarItem Component ---
  const SidebarItem = ({
    item,
    isActive,
    onClick,
    showProBadge = false
  }: {
    item: { path: string, icon: React.ReactNode, label: string },
    isActive: boolean,
    onClick: () => void,
    showProBadge?: boolean
  }) => (
    <div
      onClick={onClick}
      onMouseEnter={() => setHoveredItem(item.path)}
      onMouseLeave={() => setHoveredItem(null)}
      className={`
        relative flex items-center h-12 mx-3 rounded-xl cursor-pointer
        transition-all duration-200 group
        ${isCollapsed ? 'justify-center px-0' : 'px-3'}
        ${isActive
          ? 'bg-slate-800/80'
          : 'hover:bg-slate-800/40'
        }
      `}
      title={isCollapsed ? item.label : undefined}
    >
      {/* Icon */}
      <div className={`
        w-6 h-6 flex items-center justify-center flex-shrink-0
        transition-colors duration-200
        ${isActive ? 'text-accent' : 'text-slate-400 group-hover:text-slate-300'}
      `}>
        {item.icon}
      </div>

      {/* Label */}
      <span className={`
        text-sm font-medium whitespace-nowrap overflow-hidden
        transition-all duration-300
        ${isActive ? 'text-white' : 'text-slate-400 group-hover:text-slate-300'}
        ${isCollapsed ? 'w-0 opacity-0 ml-0' : 'flex-1 opacity-100 ml-3'}
      `}>
        {item.label}
      </span>

      {/* Pro Badge - shown for non-Pro users */}
      {showProBadge && !isPro && !isCollapsed && (
        <span className="px-1.5 py-0.5 text-[9px] font-semibold bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded flex-shrink-0 mr-1">
          Pro
        </span>
      )}

      {/* Active Chevron Arrow */}
      {isActive && !isCollapsed && (
        <ChevronRight size={16} className="text-slate-500 flex-shrink-0" />
      )}

      {/* Collapsed Tooltip */}
      {isCollapsed && hoveredItem === item.path && (
        <div className="absolute left-full top-1/2 -translate-y-1/2 ml-3 px-3 py-1.5 bg-slate-800 border border-white/10 rounded-lg text-xs text-white whitespace-nowrap z-[100] shadow-xl flex items-center gap-2">
          {item.label}
          {showProBadge && !isPro && (
            <span className="px-1 py-0.5 text-[8px] font-semibold bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded">
              Pro
            </span>
          )}
        </div>
      )}
    </div>
  );

  return (
    <div className="flex h-screen w-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950 text-text-main overflow-hidden font-sans relative">
      {/* Draggable Titlebar */}
      <div data-tauri-drag-region className="absolute top-0 left-0 right-0 h-8 z-50" />

      {/* Subtle Grid Background */}
      <div className="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:40px_40px] pointer-events-none opacity-20" />

      {/* Sidebar */}
      <div className={`
        ${isCollapsed ? 'w-[72px]' : 'w-[280px]'}
        bg-slate-950/80 backdrop-blur-xl flex flex-col border-r border-white/5 z-40 
        transition-all duration-300 ease-out relative
      `}>

        {/* Collapse Button */}
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="absolute top-[72px] -right-2.5 w-5 h-5 rounded-full flex items-center justify-center bg-slate-800 border border-slate-600 text-accent hover:bg-slate-700 hover:border-accent/50 transition-all duration-200 z-50"
        >
          {isCollapsed ? <ChevronRight size={12} /> : <ChevronLeft size={12} />}
        </button>

        {/* Navigation Container - Logo and Menu in same container */}
        <div className="flex-1 overflow-y-auto overflow-x-hidden pt-[40px] pb-2 custom-scrollbar flex flex-col">
          {/* Logo Section */}
          <div className={`mb-5 ${isCollapsed ? 'flex justify-center' : 'flex items-center pl-1 pr-2'}`}>
            {/* Logo Icon */}
            <div className="relative flex-shrink-0 group/logo">
              {/* Logo container - Clean, no border/background */}
              <div className={`relative flex items-center justify-center cursor-pointer transition-all duration-300 group-hover/logo:scale-110 ${isCollapsed ? 'w-[56px] h-[51px]' : 'w-[74px] h-[67px]'}`}>
                <img
                  src="/logo-shield.png"
                  alt="Logo"
                  className="w-full h-full object-contain drop-shadow-lg"
                />
              </div>
            </div>

            {/* Logo Text - Only show when expanded */}
            {!isCollapsed && (
              <div className="ml-3 overflow-hidden">
                <div className="flex items-center gap-2 flex-wrap">
                  <h1 className={`${i18n.language === 'en-US' ? 'text-[11px] font-bold tracking-normal' : 'text-xl font-bold tracking-wide'} text-white whitespace-nowrap`}>
                    {i18n.language === 'en-US' ? 'TRACELESS' : '无痕'}
                  </h1>
                  <span className="px-1.5 py-0.5 text-[10px] font-medium text-accent bg-accent/10 border border-accent/20 rounded flex-shrink-0">
                    v1.0.0
                  </span>
                  {/* License Status Badge - Right of version */}
                  {isPro ? (
                    <span className="px-1.5 py-0.5 text-[9px] font-semibold bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded flex-shrink-0 flex items-center gap-1">
                      <Shield size={9} />
                      {t('license.pro')}
                      {status?.days_remaining !== null && status?.days_remaining !== undefined && (
                        <span className="opacity-80">· {t('license.daysRemainingShort', { days: status.days_remaining })}</span>
                      )}
                    </span>
                  ) : (
                    <span className="px-1.5 py-0.5 text-[9px] font-medium text-slate-400 bg-slate-800/50 border border-slate-700/50 rounded flex-shrink-0">
                      {t('license.free')}
                    </span>
                  )}
                </div>
                {i18n.language === 'en-US' && (
                  <p className="text-[10px] text-slate-400 tracking-wide mt-0.5">PROTECTION</p>
                )}
              </div>
            )}
          </div>

          {/* Menu Items */}
          <div className="flex flex-col gap-0.5">
            {menuItems.map(item => (
              <SidebarItem
                key={item.path}
                item={item}
                isActive={location.pathname === item.path}
                onClick={() => handleNavigation(item.path)}
                showProBadge={isProFeaturePath(item.path)}
              />
            ))}
          </div>
        </div>

        {/* Footer - License & Settings */}
        <div className="flex-shrink-0 py-4 border-t border-white/5">
          {/* License Status/Activate Button */}
          <div
            onClick={() => setShowLicenseDialog(true)}
            onMouseEnter={() => setHoveredItem('license')}
            onMouseLeave={() => setHoveredItem(null)}
            className={`
              relative flex items-center h-12 mx-3 rounded-xl cursor-pointer
              transition-all duration-200 group
              ${isCollapsed ? 'justify-center px-0' : 'px-3'}
              ${isPro
                ? 'bg-gradient-to-r from-indigo-500/10 to-purple-500/10 border border-indigo-500/20'
                : 'hover:bg-slate-800/40'
              }
            `}
            title={isCollapsed ? (isPro ? t('license.pro') : t('license.activatePro')) : undefined}
          >
            <div className={`
              w-6 h-6 flex items-center justify-center flex-shrink-0
              transition-colors duration-200
              ${isPro ? 'text-indigo-400' : 'text-slate-400 group-hover:text-indigo-400'}
            `}>
              <Shield size={20} />
            </div>
            <span className={`
              text-sm font-medium whitespace-nowrap overflow-hidden
              transition-all duration-300
              ${isPro ? 'text-indigo-300' : 'text-slate-400 group-hover:text-slate-300'}
              ${isCollapsed ? 'w-0 opacity-0 ml-0' : 'flex-1 opacity-100 ml-3'}
            `}>
              {isPro ? (
                <>
                  {t('license.pro')}
                  {status?.days_remaining !== null && status?.days_remaining !== undefined && (
                    <span className="ml-1 text-xs text-slate-500">
                      {t('license.daysRemainingShort', { days: status.days_remaining })}
                    </span>
                  )}
                </>
              ) : t('license.activatePro')}
            </span>
            {isCollapsed && hoveredItem === 'license' && (
              <div className="absolute left-full top-1/2 -translate-y-1/2 ml-3 px-3 py-1.5 bg-slate-800 border border-white/10 rounded-lg text-xs text-white whitespace-nowrap z-[100] shadow-xl">
                {isPro ? `${t('license.pro')} ${t('license.daysRemainingShort', { days: status?.days_remaining || 0 })}` : t('license.activatePro')}
              </div>
            )}
          </div>

          <SidebarItem
            item={{ path: '/settings', icon: <Settings size={20} />, label: t('nav.settings') }}
            isActive={location.pathname === '/settings'}
            onClick={() => handleNavigation('/settings')}
          />
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col relative z-10 h-full overflow-hidden bg-transparent">
        <main className="flex-1 overflow-y-auto overflow-x-hidden p-6 relative custom-scrollbar">
          {children}
        </main>
        <AboutDialog open={showAboutDialog} onClose={() => setShowAboutDialog(false)} />
        <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </div>
  );
};

export default Layout;
