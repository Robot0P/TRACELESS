import React, { useState, useEffect } from 'react';
import { Button, Progress, Modal, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Database,
  History,
  Folder,
  Usb,
  CheckCircle2,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  FolderTree,
  Key,
  FileSearch,
  Trash2,
  Shield,
  RefreshCw,
  Terminal,
  Eye,
  Download,
  Globe,
  Search,
  Wifi,
  HardDrive,
} from 'lucide-react';

interface RegistryItemInfo {
  key: string;
  name: string;
  description: string;
  path: string;
  entry_count: number;
  size_estimate: string;
  risk_level: string;
  category: string;
  platform: string;
  requires_admin: boolean;
  last_modified: string | null;
}

interface RegistryStatus {
  platform: string;
  supported: boolean;
  items: RegistryItemInfo[];
  total_entries: number;
  categories: string[];
}

// 根据 key 获取图标
const getItemIcon = (key: string) => {
  const iconMap: Record<string, React.ReactNode> = {
    // Windows
    mru: <History size={24} />,
    userassist: <FileSearch size={24} />,
    shellbags: <Folder size={24} />,
    recentdocs: <FolderTree size={24} />,
    usbhistory: <Usb size={24} />,
    network: <Wifi size={24} />,
    search: <Search size={24} />,
    // macOS
    recent_items: <History size={24} />,
    finder_recents: <Folder size={24} />,
    app_usage: <Shield size={24} />,
    quicklook: <Eye size={24} />,
    spotlight: <Search size={24} />,
    shell_history: <Terminal size={24} />,
    quarantine: <Download size={24} />,
    safari: <Globe size={24} />,
    // Linux
    recently_used: <History size={24} />,
    bash_history: <Terminal size={24} />,
    zsh_history: <Terminal size={24} />,
    thumbnails: <Eye size={24} />,
    trash: <Trash2 size={24} />,
    viminfo: <FileSearch size={24} />,
  };
  return iconMap[key] || <Database size={24} />;
};

// 根据 key 获取翻译的项目名称
const translateItemName = (t: any, key: string, fallback: string) => {
  const translationKey = `registryCleanup.itemNames.${key}`;
  const translated = t(translationKey);
  // 如果翻译键不存在，i18next 会返回键本身，此时使用 fallback
  return translated === translationKey ? fallback : translated;
};

// 根据 key 获取翻译的项目描述
const translateItemDescription = (t: any, key: string, fallback: string) => {
  const translationKey = `registryCleanup.itemDescriptions.${key}`;
  const translated = t(translationKey);
  return translated === translationKey ? fallback : translated;
};

// 根据中文分类名称获取翻译
const translateCategory = (t: any, category: string) => {
  const categoryMap: Record<string, string> = {
    '使用记录': 'registryCleanup.categoryNames.usage',
    '访问记录': 'registryCleanup.categoryNames.access',
    '权限记录': 'registryCleanup.categoryNames.permission',
    '浏览记录': 'registryCleanup.categoryNames.browsing',
    '浏览器记录': 'registryCleanup.categoryNames.browsing',
    '搜索记录': 'registryCleanup.categoryNames.search',
    '命令记录': 'registryCleanup.categoryNames.command',
    '命令历史': 'registryCleanup.categoryNames.command',
    '下载记录': 'registryCleanup.categoryNames.download',
    '缓存记录': 'registryCleanup.categoryNames.cache',
  };

  const translationKey = categoryMap[category];
  if (translationKey) {
    return t(translationKey);
  }
  return category;
};

// 根据 key 获取颜色
const getItemColor = (key: string) => {
  const colorMap: Record<string, string> = {
    mru: 'text-blue-400',
    userassist: 'text-purple-400',
    shellbags: 'text-orange-400',
    recentdocs: 'text-green-400',
    usbhistory: 'text-pink-400',
    network: 'text-cyan-400',
    search: 'text-yellow-400',
    recent_items: 'text-blue-400',
    finder_recents: 'text-green-400',
    app_usage: 'text-purple-400',
    quicklook: 'text-orange-400',
    spotlight: 'text-yellow-400',
    shell_history: 'text-red-400',
    quarantine: 'text-pink-400',
    safari: 'text-cyan-400',
    recently_used: 'text-blue-400',
    bash_history: 'text-red-400',
    zsh_history: 'text-green-400',
    thumbnails: 'text-orange-400',
    trash: 'text-pink-400',
    viminfo: 'text-purple-400',
  };
  return colorMap[key] || 'text-slate-400';
};

// 根据 key 获取渐变色
const getItemGradient = (key: string) => {
  const gradientMap: Record<string, string> = {
    mru: 'from-blue-500 to-cyan-500',
    userassist: 'from-purple-500 to-pink-500',
    shellbags: 'from-orange-500 to-red-500',
    recentdocs: 'from-green-500 to-emerald-500',
    usbhistory: 'from-pink-500 to-rose-500',
    network: 'from-cyan-500 to-teal-500',
    search: 'from-yellow-500 to-orange-500',
    recent_items: 'from-blue-500 to-indigo-500',
    finder_recents: 'from-green-500 to-teal-500',
    app_usage: 'from-purple-500 to-violet-500',
    quicklook: 'from-orange-500 to-amber-500',
    spotlight: 'from-yellow-500 to-lime-500',
    shell_history: 'from-red-500 to-pink-500',
    quarantine: 'from-pink-500 to-fuchsia-500',
    safari: 'from-cyan-500 to-blue-500',
    recently_used: 'from-blue-500 to-purple-500',
    bash_history: 'from-red-500 to-orange-500',
    zsh_history: 'from-green-500 to-cyan-500',
    thumbnails: 'from-orange-500 to-yellow-500',
    trash: 'from-pink-500 to-red-500',
    viminfo: 'from-purple-500 to-indigo-500',
  };
  return gradientMap[key] || 'from-slate-500 to-slate-600';
};

const RegistryCleanup: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [initialLoading, setInitialLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [showSuccess, setShowSuccess] = useState(false);
  const [currentKey, setCurrentKey] = useState('');
  const [registryStatus, setRegistryStatus] = useState<RegistryStatus | null>(null);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  // 加载注册表信息
  const loadRegistryInfo = async () => {
    setInitialLoading(true);
    const startTime = Date.now();
    const minDisplayTime = 800; // 最小显示时间 800ms，确保用户能看到扫描动画

    try {
      const result = await invoke<RegistryStatus>('get_registry_info');
      setRegistryStatus(result);
      // 默认选中高风险项
      const highRiskKeys = result.items
        .filter(item => item.risk_level === 'high')
        .map(item => item.key);
      setSelectedKeys(highRiskKeys);
    } catch {
      // Silently fail
    } finally {
      // 确保扫描动画至少显示 minDisplayTime
      const elapsed = Date.now() - startTime;
      if (elapsed < minDisplayTime) {
        await new Promise(resolve => setTimeout(resolve, minDisplayTime - elapsed));
      }
      setInitialLoading(false);
    }
  };

  useEffect(() => {
    loadRegistryInfo();
  }, []);

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadRegistryInfo();
    setTimeout(() => setRefreshing(false), 500);
  };

  const handleToggleKey = (value: string) => {
    setSelectedKeys(prev =>
      prev.includes(value)
        ? prev.filter(v => v !== value)
        : [...prev, value]
    );
  };

  const handleSelectAll = () => {
    if (!registryStatus) return;
    if (selectedKeys.length === registryStatus.items.length) {
      setSelectedKeys([]);
    } else {
      setSelectedKeys(registryStatus.items.map(item => item.key));
    }
  };

  const handleClean = async () => {
    if (selectedKeys.length === 0 || !registryStatus) {
      Modal.warning({
        title: t('common.warning'),
        content: t('registryCleanup.errors.selectTypes'),
        centered: true,
      });
      return;
    }

    const selectedItems = registryStatus.items.filter(item => selectedKeys.includes(item.key));
    const totalEntries = selectedItems.reduce((sum, item) => sum + item.entry_count, 0);

    Modal.confirm({
      title: t('registryCleanup.warnings.confirmClean'),
      icon: <AlertTriangle className="text-red-500" size={24} />,
      content: (
        <div>
          <p className="text-slate-400 mb-2">
            {t('registryCleanup.warnings.cannotUndo')}
          </p>
          <p className="text-red-400 text-sm">
            {t('registryCleanup.selectedItems')}: {selectedKeys.length} {t('registryCleanup.items')} · {t('registryCleanup.estimatedCleanup')} {totalEntries} {t('registryCleanup.records')}
          </p>
        </div>
      ),
      okText: t('registryCleanup.startClean'),
      okType: 'danger',
      cancelText: t('common.cancel'),
      centered: true,
      onOk: async () => {
        setLoading(true);
        setProgress(0);

        try {
          for (let i = 0; i < selectedKeys.length; i++) {
            const keyValue = selectedKeys[i];
            const keyData = registryStatus.items.find(item => item.key === keyValue);

            setCurrentKey(keyData ? translateItemName(t, keyData.key, keyData.name) : keyValue);

            // 模拟清理过程
            for (let j = 0; j <= 10; j++) {
              await new Promise(resolve => setTimeout(resolve, 100));
              const itemProgress = ((i + j / 10) / selectedKeys.length) * 100;
              setProgress(Math.min(itemProgress, 100));
            }

            // 调用实际的清理命令
            await invoke('clean_registry', {
              types: [keyValue],
            });
          }

          // 重新加载数据
          await loadRegistryInfo();

          setLoading(false);
          setShowSuccess(true);
        } catch (error) {
          setLoading(false);
          Modal.error({
            title: t('registryCleanup.errors.cleanFailed'),
            content: String(error),
            centered: true,
          });
        }
      },
    });
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'high': return 'text-red-400 bg-red-500/10 border-red-500/30';
      case 'medium': return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
      case 'low': return 'text-green-400 bg-green-500/10 border-green-500/30';
      default: return 'text-gray-400 bg-gray-500/10 border-gray-500/30';
    }
  };

  const getRiskLabel = (risk: string) => {
    switch (risk) {
      case 'high': return t('registryCleanup.riskLevels.high');
      case 'medium': return t('registryCleanup.riskLevels.medium');
      case 'low': return t('registryCleanup.riskLevels.low');
      default: return '';
    }
  };

  const getPlatformIcon = () => {
    if (!registryStatus) return <Database className="w-7 h-7 text-pink-400" />;
    switch (registryStatus.platform) {
      case 'Windows': return <Database className="w-7 h-7 text-blue-400" />;
      case 'macOS': return <HardDrive className="w-7 h-7 text-slate-400" />;
      case 'Linux': return <Terminal className="w-7 h-7 text-orange-400" />;
      default: return <Database className="w-7 h-7 text-pink-400" />;
    }
  };

  // 计算选中项的总条目数
  const selectedItems = registryStatus?.items.filter(item => selectedKeys.includes(item.key)) || [];
  const totalEntries = selectedItems.reduce((sum, item) => sum + item.entry_count, 0);

  if (initialLoading) {
    return (
      <div className="h-full flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="text-center">
          <Loader2 className="w-12 h-12 text-pink-400 animate-spin mx-auto mb-4" />
          <p className="text-slate-400">{t('registryCleanup.scanning')}</p>
        </div>
      </div>
    );
  }

  if (!registryStatus) {
    return (
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="flex-1 flex items-center justify-center p-6">
          <div className="text-center animate-slideInUp">
            <div className="relative inline-block mb-6">
              <div className="absolute inset-0 bg-red-500/20 rounded-full blur-2xl" />
              <div className="relative p-8 bg-gradient-to-br from-red-500/20 to-red-500/5 rounded-full border border-red-500/30">
                <AlertTriangle className="w-16 h-16 text-red-400" />
              </div>
            </div>
            <h2 className="text-2xl font-bold text-white mb-2">{t('registryCleanup.loadingFailed')}</h2>
            <p className="text-slate-400">
              {t('registryCleanup.loadingFailedDesc')}
            </p>
            <Button
              size="large"
              onClick={() => navigate('/dashboard')}
              className="mt-6 bg-slate-700 hover:bg-slate-600 text-white border-none"
            >
              {t('registryCleanup.backHome')}
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <ProFeatureGate
      feature="registry_cleanup"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 right-1/3 w-96 h-96 bg-pink-500/5 rounded-full blur-3xl animate-pulse" />
      </div>

      <div className="flex-1 flex flex-col p-6 relative z-10 min-h-0 overflow-hidden">
        {/* 顶部导航 */}
        <div className="flex items-center justify-between mb-6 animate-slideInDown flex-shrink-0">
          <div className="flex items-center gap-4">
            <button
              onClick={() => navigate('/dashboard')}
              className="p-2 rounded-lg bg-slate-800/50 border border-white/5 hover:border-accent/30 text-slate-400 hover:text-white transition-all"
            >
              <ArrowLeft size={20} />
            </button>
            <div>
              <h1 className="text-2xl font-bold text-white flex items-center gap-3">
                {getPlatformIcon()}
                {t('registryCleanup.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('registryCleanup.subtitle')}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={handleRefresh}
              className={`p-2 rounded-lg bg-slate-800/50 border border-white/5 hover:border-accent/30 text-slate-400 hover:text-white transition-all ${refreshing ? 'animate-spin' : ''}`}
            >
              <RefreshCw size={18} />
            </button>
            <div className="flex items-center gap-2 px-4 py-2 bg-pink-500/10 border border-pink-500/20 rounded-lg">
              <Shield className="w-4 h-4 text-pink-400" />
              <span className="text-sm text-pink-300">{registryStatus.platform}</span>
            </div>
          </div>
        </div>

        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden min-h-0">
          {/* 左侧：项目列表 */}
          <div className="lg:col-span-2 flex-1 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft min-h-0 flex flex-col">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <FolderTree className="w-5 h-5 text-pink-400" />
                <h3 className="text-lg font-bold text-white">{t('registryCleanup.privacyTraces')}</h3>
                <span className="text-sm text-slate-400">({selectedKeys.length}/{registryStatus.items.length})</span>
              </div>
              <Button
                onClick={handleSelectAll}
                size="small"
                className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
              >
                {selectedKeys.length === registryStatus.items.length ? t('registryCleanup.deselectAll') : t('common.selectAll')}
              </Button>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar space-y-3">
              {registryStatus.items.map((item) => {
                const isSelected = selectedKeys.includes(item.key);
                return (
                  <Tooltip key={item.key} title={item.path}>
                    <button
                      onClick={() => handleToggleKey(item.key)}
                      className={`
                        w-full relative p-4 rounded-xl border-2 transition-all duration-300 text-left overflow-hidden
                        ${isSelected
                          ? 'border-pink-400/30 bg-pink-500/10 shadow-lg shadow-pink-500/20'
                          : 'border-white/10 hover:border-white/20 bg-slate-800/30'
                        }
                      `}
                    >
                      <div className={`absolute inset-0 bg-gradient-to-br ${getItemGradient(item.key)} opacity-0 ${isSelected ? 'opacity-10' : ''} transition-opacity`} />

                      <div className="relative z-10">
                        <div className="flex items-start justify-between mb-3">
                          <div className="flex items-center gap-3">
                            <div className={`p-2 rounded-lg bg-slate-700/50 ${getItemColor(item.key)}`}>
                              {getItemIcon(item.key)}
                            </div>
                            <div>
                              <div className="text-white font-bold mb-1">{translateItemName(t, item.key, item.name)}</div>
                              <div className="text-xs text-slate-400">{translateItemDescription(t, item.key, item.description)}</div>
                            </div>
                          </div>
                          <div className={`flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-medium border ${getRiskColor(item.risk_level)}`}>
                            {getRiskLabel(item.risk_level)}
                          </div>
                        </div>

                        <div className="bg-black/20 rounded-lg p-2 border border-white/5">
                          <div className="flex items-center gap-2 text-xs font-mono text-slate-500">
                            <Key className="w-3 h-3" />
                            <span className="truncate">{item.path}</span>
                          </div>
                        </div>

                        <div className="mt-2 flex items-center justify-between text-xs">
                          <div className="flex items-center gap-3">
                            <span className="text-slate-400">{item.entry_count} {t('registryCleanup.records')}</span>
                            <span className={`px-2 py-0.5 rounded ${isSelected ? 'bg-pink-500/20 text-pink-400' : 'bg-slate-700/50 text-slate-400'}`}>
                              {item.size_estimate}
                            </span>
                            <span className="text-slate-500">{translateCategory(t, item.category)}</span>
                          </div>
                          <div className={`w-4 h-4 rounded-full border-2 transition-all ${isSelected ? 'border-pink-400 bg-pink-400' : 'border-slate-600'} flex items-center justify-center`}>
                            {isSelected && <CheckCircle2 className="w-2.5 h-2.5 text-slate-900" />}
                          </div>
                        </div>
                      </div>
                    </button>
                  </Tooltip>
                );
              })}
            </div>
          </div>

          {/* 右侧：控制面板 */}
          <div className="space-y-6">
            {/* 统计信息 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight">
              <h3 className="text-lg font-bold text-white mb-4">{t('registryCleanup.cleanupStats')}</h3>

              <div className="space-y-4">
                <div className="p-4 bg-pink-500/10 rounded-lg border border-pink-500/20">
                  <div className="text-sm text-slate-400 mb-1">{t('registryCleanup.estimatedCleanup')}</div>
                  <div className="text-3xl font-bold text-pink-400">{totalEntries}</div>
                  <div className="text-xs text-pink-300 mt-1">{t('registryCleanup.privacyTracesCount')}</div>
                </div>

                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('registryCleanup.selectedItems')}</span>
                    <span className="text-white font-medium">{selectedKeys.length} {t('registryCleanup.items')}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('registryCleanup.highRiskItems')}</span>
                    <span className="text-red-400 font-medium">
                      {selectedItems.filter(item => item.risk_level === 'high').length} {t('registryCleanup.items')}
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('registryCleanup.systemTotalRecords')}</span>
                    <span className="text-accent font-medium">{registryStatus.total_entries} {t('registryCleanup.records')}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('registryCleanup.categoriesCount')}</span>
                    <span className="text-white font-medium">{registryStatus.categories.length} {t('registryCleanup.categories')}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* 操作按钮 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.1s' }}>
              <Button
                type="primary"
                size="large"
                block
                onClick={handleClean}
                disabled={selectedKeys.length === 0 || loading}
                className="h-12 bg-gradient-to-r from-pink-600 to-pink-500 border-none hover:from-pink-500 hover:to-pink-400 text-white font-bold"
                icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Trash2 size={20} />}
              >
                {loading ? t('registryCleanup.cleaning') : t('registryCleanup.startClean')}
              </Button>

              {loading && (
                <div className="mt-4 space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-slate-300">{t('registryCleanup.cleanupProgress')}</span>
                    <span className="text-pink-400 font-mono">{Math.floor(progress)}%</span>
                  </div>
                  <Progress
                    percent={progress}
                    strokeColor={{
                      '0%': '#ec4899',
                      '100%': '#be185d',
                    }}
                    trailColor="rgba(255,255,255,0.05)"
                    showInfo={false}
                    strokeWidth={10}
                  />
                  <div className="text-xs text-slate-400 truncate">
                    {currentKey}
                  </div>
                </div>
              )}

              <div className="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
                  <div className="text-xs text-red-300">
                    <div className="font-bold mb-1">{t('common.warning')}</div>
                    {t('registryCleanup.warnings.deleteWarning')}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 成功弹窗 */}
      <Modal
        open={showSuccess}
        onCancel={() => setShowSuccess(false)}
        footer={null}
        centered
        width={500}
      >
        <div className="text-center py-8">
          <div className="relative inline-block mb-6">
            <div className="absolute inset-0 bg-green-500/20 rounded-full blur-2xl" />
            <div className="relative p-6 bg-gradient-to-br from-green-500/20 to-green-500/5 rounded-full border border-green-500/30">
              <CheckCircle2 className="w-16 h-16 text-green-400" />
            </div>
          </div>

          <h2 className="text-2xl font-bold text-white mb-2">{t('registryCleanup.cleanComplete')}</h2>
          <p className="text-slate-400 mb-2">
            {t('registryCleanup.cleanCompleteDesc1')} <span className="text-accent font-bold">{selectedKeys.length}</span> {t('registryCleanup.cleanCompleteDesc2')}
          </p>
          <p className="text-green-400 text-sm mb-6">
            {t('registryCleanup.deletedRecords')} {totalEntries} {t('registryCleanup.privacyTracesCount')}
          </p>

          <Button
            type="primary"
            size="large"
            onClick={() => {
              setShowSuccess(false);
              // 重新选中高风险项
              if (registryStatus) {
                const highRiskKeys = registryStatus.items
                  .filter(item => item.risk_level === 'high')
                  .map(item => item.key);
                setSelectedKeys(highRiskKeys);
              }
            }}
            className="bg-accent hover:bg-accent/80 border-none"
          >
            {t('fileCleanup.status.done')}
          </Button>
        </div>
      </Modal>
      <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </ProFeatureGate>
  );
};

export default RegistryCleanup;
