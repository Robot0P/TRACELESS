import React, { useState, useEffect } from 'react';
import { Button, Progress, Modal, Switch, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Cpu,
  HardDrive,
  Zap,
  CheckCircle2,
  AlertCircle,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  Activity,
  Database,
  Copy,
  Power,
  RefreshCw,
  Clock,
  User,
  MemoryStick,
  Gauge,
} from 'lucide-react';

interface MemoryType {
  value: string;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  gradient: string;
  requiresAdmin: boolean;
  platform: 'all' | 'macos' | 'windows' | 'linux';
  getSize?: (info: MemoryInfo | undefined) => string;
}

// 扫描项接口
interface MemoryCleanItem {
  item_type: string;
  label: string;
  description: string;
  size: number;
  size_display: string;
  accessible: boolean;
  category: string;
}

// 扫描结果接口
interface MemoryScanResult {
  items: MemoryCleanItem[];
  total_size: number;
  total_items: number;
  memory_info: MemoryInfo;
}

interface MemoryInfo {
  total_memory: number;
  used_memory: number;
  free_memory: number;
  memory_usage: number;
  wired_memory: number;
  active_memory: number;
  inactive_memory: number;
  compressed_memory: number;
  cached_memory: number;
  swap_used: number;
  swap_total: number;
  app_memory: number;
  pagefile_size?: string;
  hibernation_size?: string;
  swap_size?: string;
}

interface ProcessInfo {
  pid: number;
  name: string;
  memory: number;
  memory_display: string;
  cpu_usage: number;
  user: string;
}

interface DetailedMemoryInfo {
  memory_info: MemoryInfo;
  top_processes: ProcessInfo[];
  memory_pressure: string;
  uptime: string;
}

// 格式化大小的辅助函数
const formatSize = (mb: number) => {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(2)} GB`;
  }
  return `${mb.toFixed(0)} MB`;
};

const MemoryCleanup: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [showSuccess, setShowSuccess] = useState(false);
  const [currentItem, setCurrentItem] = useState('');
  const [refreshing, setRefreshing] = useState(false);
  const [scanning, setScanning] = useState(true);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  // 详细内存信息
  const [detailedInfo, setDetailedInfo] = useState<DetailedMemoryInfo | null>(null);

  // 扫描结果 (保留用于未来扩展显示扫描项详情)
  const [_scanResult, setScanResult] = useState<MemoryScanResult | null>(null);

  // 平台检测
  const [platform, setPlatform] = useState<string>('macos');

  // 根据平台过滤的内存类型
  const allMemoryTypes: MemoryType[] = [
    {
      value: 'clipboard',
      label: t('memoryCleanup.types.clipboard.label'),
      description: t('memoryCleanup.types.clipboard.desc'),
      icon: <Copy size={24} />,
      color: 'text-cyan-400',
      gradient: 'from-cyan-500 to-blue-500',
      requiresAdmin: false,
      platform: 'all',
      getSize: () => '~1 KB',
    },
    {
      value: 'working_set',
      label: t('memoryCleanup.types.working_set.label'),
      description: t('memoryCleanup.types.working_set.desc'),
      icon: <MemoryStick size={24} />,
      color: 'text-blue-400',
      gradient: 'from-blue-500 to-indigo-500',
      requiresAdmin: false,
      platform: 'macos',
      getSize: (info) => info?.inactive_memory ? formatSize(info.inactive_memory) : '--',
    },
    {
      value: 'standby',
      label: t('memoryCleanup.types.standby.label'),
      description: t('memoryCleanup.types.standby.desc'),
      icon: <Zap size={24} />,
      color: 'text-yellow-400',
      gradient: 'from-yellow-500 to-orange-500',
      requiresAdmin: false,
      platform: 'macos',
      getSize: (info) => info?.cached_memory ? formatSize(info.cached_memory) : '--',
    },
    {
      value: 'swap',
      label: t('memoryCleanup.types.swap.label'),
      description: t('memoryCleanup.types.swap.desc'),
      icon: <Database size={24} />,
      color: 'text-green-400',
      gradient: 'from-green-500 to-emerald-500',
      requiresAdmin: true,
      platform: 'all',
      getSize: (info) => {
        if (info?.swap_size) return info.swap_size;
        if (info?.swap_used) return formatSize(info.swap_used);
        return '--';
      },
    },
    {
      value: 'dns_cache',
      label: t('memoryCleanup.types.dns_cache.label'),
      description: t('memoryCleanup.types.dns_cache.desc'),
      icon: <Activity size={24} />,
      color: 'text-purple-400',
      gradient: 'from-purple-500 to-pink-500',
      requiresAdmin: false,
      platform: 'macos',
      getSize: () => '~512 KB',
    },
    {
      value: 'pagefile',
      label: t('memoryCleanup.types.pagefile.label'),
      description: t('memoryCleanup.types.pagefile.desc'),
      icon: <HardDrive size={24} />,
      color: 'text-orange-400',
      gradient: 'from-orange-500 to-red-500',
      requiresAdmin: true,
      platform: 'windows',
      getSize: (info) => info?.pagefile_size || '--',
    },
    {
      value: 'hibernation',
      label: t('memoryCleanup.types.hibernation.label'),
      description: t('memoryCleanup.types.hibernation.desc'),
      icon: <Power size={24} />,
      color: 'text-red-400',
      gradient: 'from-red-500 to-pink-500',
      requiresAdmin: true,
      platform: 'windows',
      getSize: (info) => info?.hibernation_size || '--',
    },
  ];

  // 根据平台过滤内存类型
  const memoryTypes = allMemoryTypes.filter(
    (type) => type.platform === 'all' || type.platform === platform
  );

  useEffect(() => {
    // 获取平台信息
    invoke<string>('get_platform').then((p) => {
      setPlatform(p.toLowerCase());
    }).catch(() => {
      // 默认 macOS
      setPlatform('macos');
    });

    // 执行扫描
    performScan();
  }, []);

  // 执行内存扫描
  const performScan = async () => {
    setScanning(true);
    const startTime = Date.now();
    const minDisplayTime = 800; // 最小显示时间 800ms，确保用户能看到扫描动画

    try {
      // 调用扫描命令
      const result = await invoke<MemoryScanResult>('scan_memory_items');
      setScanResult(result);

      // 同时获取详细内存信息
      setDetailedInfo({
        memory_info: result.memory_info,
        top_processes: [],
        memory_pressure: result.memory_info.memory_usage > 90 ? 'critical' : result.memory_info.memory_usage > 75 ? 'warning' : 'normal',
        uptime: t('memoryCleanup.unknown'),
      });

      // 默认选中所有可访问的项目
      const accessibleItems = result.items
        .filter(item => item.accessible && item.size > 0)
        .map(item => item.item_type);
      setSelectedTypes(accessibleItems.length > 0 ? accessibleItems : ['clipboard']);

      // 获取详细信息（包含进程列表）
      fetchDetailedMemoryInfo();
    } catch {
      // 如果扫描失败，回退到基本模式
      setSelectedTypes(['clipboard']);
      fetchDetailedMemoryInfo();
    } finally {
      // 确保扫描动画至少显示 minDisplayTime
      const elapsed = Date.now() - startTime;
      if (elapsed < minDisplayTime) {
        await new Promise(resolve => setTimeout(resolve, minDisplayTime - elapsed));
      }
      setScanning(false);
    }
  };

  // 定时刷新详细信息
  useEffect(() => {
    if (!scanning) {
      const interval = setInterval(fetchDetailedMemoryInfo, 3000);
      return () => clearInterval(interval);
    }
  }, [scanning]);

  const fetchDetailedMemoryInfo = async () => {
    try {
      const info = await invoke<DetailedMemoryInfo>('get_detailed_memory_info');
      setDetailedInfo(info);
    } catch {
      // 尝试获取基本内存信息
      try {
        const basicInfo = await invoke<MemoryInfo>('get_memory_info');
        setDetailedInfo({
          memory_info: basicInfo,
          top_processes: [],
          memory_pressure: basicInfo.memory_usage > 90 ? 'critical' : basicInfo.memory_usage > 75 ? 'warning' : 'normal',
          uptime: t('memoryCleanup.unknown'),
        });
      } catch {
        // Silently fail
      }
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await performScan();
    setTimeout(() => setRefreshing(false), 500);
  };

  const handleToggleType = (value: string) => {
    setSelectedTypes((prev) =>
      prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value]
    );
  };

  const handleSelectAll = () => {
    if (selectedTypes.length === memoryTypes.length) {
      setSelectedTypes([]);
    } else {
      setSelectedTypes(memoryTypes.map((t) => t.value));
    }
  };

  const handleClean = async () => {
    if (selectedTypes.length === 0) {
      Modal.warning({
        title: t('common.warning'),
        content: t('memoryCleanup.errors.selectTypes'),
        centered: true,
      });
      return;
    }

    setLoading(true);
    setProgress(0);

    try {
      for (let i = 0; i < selectedTypes.length; i++) {
        const typeValue = selectedTypes[i];
        const typeData = memoryTypes.find((t) => t.value === typeValue);

        setCurrentItem(typeData?.label || typeValue);

        // 模拟进度
        for (let j = 0; j <= 10; j++) {
          await new Promise((resolve) => setTimeout(resolve, 100));
          const itemProgress = ((i + j / 10) / selectedTypes.length) * 100;
          setProgress(Math.min(itemProgress, 100));
        }

        // 调用实际的清理命令
        await invoke('clean_memory', {
          types: [typeValue],
        });
      }

      // 重新获取内存信息
      await fetchDetailedMemoryInfo();

      setLoading(false);
      setShowSuccess(true);
    } catch (error) {
      setLoading(false);
      Modal.error({
        title: t('memoryCleanup.errors.cleanFailed'),
        content: String(error),
        centered: true,
      });
    }
  };

  const getPressureColor = (pressure: string) => {
    switch (pressure) {
      case 'critical':
        return 'text-red-400';
      case 'warning':
        return 'text-yellow-400';
      default:
        return 'text-green-400';
    }
  };

  const getPressureText = (pressure: string) => {
    switch (pressure) {
      case 'critical':
        return t('memoryCleanup.pressure.critical');
      case 'warning':
        return t('memoryCleanup.pressure.warning');
      default:
        return t('memoryCleanup.pressure.normal');
    }
  };

  const memoryInfo = detailedInfo?.memory_info;

  return (
    <ProFeatureGate
      feature="memory_cleanup"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 right-1/4 w-96 h-96 bg-orange-500/5 rounded-full blur-3xl animate-pulse" />
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
                <Cpu className="w-7 h-7 text-orange-400" />
                {t('memoryCleanup.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('memoryCleanup.subtitle')}
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
            <div className={`flex items-center gap-2 px-4 py-2 rounded-lg border ${
              detailedInfo?.memory_pressure === 'critical'
                ? 'bg-red-500/10 border-red-500/20'
                : detailedInfo?.memory_pressure === 'warning'
                ? 'bg-yellow-500/10 border-yellow-500/20'
                : 'bg-green-500/10 border-green-500/20'
            }`}>
              <Gauge className={`w-4 h-4 ${getPressureColor(detailedInfo?.memory_pressure || 'normal')}`} />
              <span className={`text-sm ${getPressureColor(detailedInfo?.memory_pressure || 'normal')}`}>
                {t('memoryCleanup.memoryPressure')}: {getPressureText(detailedInfo?.memory_pressure || 'normal')}
              </span>
            </div>
          </div>
        </div>

        {scanning ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Loader2 className="w-12 h-12 text-orange-400 animate-spin mx-auto mb-4" />
              <p className="text-slate-400">{t('memoryCleanup.scanning')}</p>
            </div>
          </div>
        ) : (
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden min-h-0">
          {/* 左侧：内存可视化 + 类型选择 */}
          <div className="lg:col-span-2 flex flex-col space-y-6 min-h-0 overflow-y-auto custom-scrollbar">
            {/* 内存可视化 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Cpu className="w-5 h-5 text-orange-400" />
                  <h3 className="text-lg font-bold text-white">{t('memoryCleanup.memoryStatus')}</h3>
                </div>
                {detailedInfo?.uptime && (
                  <div className="flex items-center gap-2 text-xs text-slate-400">
                    <Clock size={14} />
                    <span>{t('memoryCleanup.uptime')}: {detailedInfo.uptime}</span>
                  </div>
                )}
              </div>

              <div className="grid grid-cols-2 gap-6">
                {/* 内存使用环形图 */}
                <div className="flex items-center justify-center">
                  <div className="relative">
                    <svg width="180" height="180" viewBox="0 0 180 180">
                      <circle
                        cx="90"
                        cy="90"
                        r="70"
                        fill="none"
                        stroke="rgba(255,255,255,0.05)"
                        strokeWidth="16"
                      />
                      <circle
                        cx="90"
                        cy="90"
                        r="70"
                        fill="none"
                        stroke="url(#memoryGradient)"
                        strokeWidth="16"
                        strokeDasharray={`${((memoryInfo?.memory_usage || 0) / 100) * 439.82} 439.82`}
                        strokeLinecap="round"
                        transform="rotate(-90 90 90)"
                        className="transition-all duration-1000"
                      />
                      <defs>
                        <linearGradient id="memoryGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                          <stop offset="0%" stopColor="#f97316" />
                          <stop offset="100%" stopColor="#ef4444" />
                        </linearGradient>
                      </defs>
                    </svg>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                      <div className="text-3xl font-bold text-white">
                        {(memoryInfo?.memory_usage || 0).toFixed(1)}%
                      </div>
                      <div className="text-xs text-slate-400">{t('memoryCleanup.stats.used')}</div>
                    </div>
                  </div>
                </div>

                {/* 内存详细统计 */}
                <div className="grid grid-cols-2 gap-2">
                  <div className="p-2 bg-blue-500/10 rounded-lg border border-blue-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.totalMemory')}</div>
                    <div className="text-sm font-bold text-blue-400">
                      {formatSize(memoryInfo?.total_memory || 0)}
                    </div>
                  </div>
                  <div className="p-2 bg-orange-500/10 rounded-lg border border-orange-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.usedMemory')}</div>
                    <div className="text-sm font-bold text-orange-400">
                      {formatSize(memoryInfo?.used_memory || 0)}
                    </div>
                  </div>
                  <div className="p-2 bg-green-500/10 rounded-lg border border-green-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.freeMemory')}</div>
                    <div className="text-sm font-bold text-green-400">
                      {formatSize(memoryInfo?.free_memory || 0)}
                    </div>
                  </div>
                  <div className="p-2 bg-purple-500/10 rounded-lg border border-purple-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.appMemory')}</div>
                    <div className="text-sm font-bold text-purple-400">
                      {formatSize(memoryInfo?.app_memory || 0)}
                    </div>
                  </div>
                  {platform === 'macos' && (
                    <>
                      <div className="p-2 bg-cyan-500/10 rounded-lg border border-cyan-500/20">
                        <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.wiredMemory')}</div>
                        <div className="text-sm font-bold text-cyan-400">
                          {formatSize(memoryInfo?.wired_memory || 0)}
                        </div>
                      </div>
                      <div className="p-2 bg-pink-500/10 rounded-lg border border-pink-500/20">
                        <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.compressedMemory')}</div>
                        <div className="text-sm font-bold text-pink-400">
                          {formatSize(memoryInfo?.compressed_memory || 0)}
                        </div>
                      </div>
                    </>
                  )}
                  <div className="p-2 bg-yellow-500/10 rounded-lg border border-yellow-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.cachedMemory')}</div>
                    <div className="text-sm font-bold text-yellow-400">
                      {formatSize(memoryInfo?.cached_memory || 0)}
                    </div>
                  </div>
                  <div className="p-2 bg-red-500/10 rounded-lg border border-red-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.stats.swapSpace')}</div>
                    <div className="text-sm font-bold text-red-400">
                      {memoryInfo?.swap_size || '0 MB'}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* 进程列表 */}
            {detailedInfo?.top_processes && detailedInfo.top_processes.length > 0 && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.05s' }}>
                <div className="flex items-center gap-2 mb-4">
                  <Activity className="w-5 h-5 text-accent" />
                  <h3 className="text-lg font-bold text-white">{t('memoryCleanup.topProcesses')}</h3>
                  <span className="text-xs text-slate-400">{t('memoryCleanup.topProcessesDesc')}</span>
                </div>

                <div className="space-y-2 max-h-48 overflow-y-auto custom-scrollbar">
                  {detailedInfo.top_processes.slice(0, 10).map((process, index) => (
                    <div
                      key={`${process.pid}-${index}`}
                      className="flex items-center justify-between p-2 bg-slate-800/30 rounded-lg hover:bg-slate-800/50 transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-xs text-slate-500 w-5">{index + 1}</span>
                        <div>
                          <div className="text-sm text-white font-medium truncate max-w-[180px]">
                            {process.name}
                          </div>
                          <div className="flex items-center gap-2 text-xs text-slate-400">
                            <span>PID: {process.pid}</span>
                            {process.user && (
                              <>
                                <span>•</span>
                                <User size={10} />
                                <span>{process.user}</span>
                              </>
                            )}
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-sm font-bold text-orange-400">
                          {process.memory_display}
                        </div>
                        <div className="text-xs text-slate-400">
                          CPU: {process.cpu_usage.toFixed(1)}%
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 内存类型选择 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.1s' }}>
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Database className="w-5 h-5 text-accent" />
                  <h3 className="text-lg font-bold text-white">{t('memoryCleanup.cleanupItems')}</h3>
                  <span className="text-sm text-slate-400">
                    ({selectedTypes.length}/{memoryTypes.length})
                  </span>
                </div>
                <Button
                  onClick={handleSelectAll}
                  size="small"
                  className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
                >
                  {selectedTypes.length === memoryTypes.length ? t('common.clearAll') : t('common.selectAll')}
                </Button>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                {memoryTypes.map((type) => {
                  const isSelected = selectedTypes.includes(type.value);
                  const sizeText = type.getSize ? type.getSize(memoryInfo) : '';
                  return (
                    <Tooltip key={type.value} title={type.description}>
                      <button
                        onClick={() => handleToggleType(type.value)}
                        className={`
                          relative p-4 rounded-xl border-2 transition-all duration-300 text-left overflow-hidden
                          ${
                            isSelected
                              ? 'border-orange-400/30 bg-orange-500/10 shadow-lg shadow-orange-500/20'
                              : 'border-white/10 hover:border-white/20 bg-slate-800/30'
                          }
                        `}
                      >
                        <div
                          className={`absolute inset-0 bg-gradient-to-br ${type.gradient} opacity-0 ${
                            isSelected ? 'opacity-10' : ''
                          } transition-opacity`}
                        />

                        <div className="relative z-10">
                          <div className="flex items-center justify-between mb-2">
                            <div className={type.color}>{type.icon}</div>
                            <div onClick={(e) => e.stopPropagation()}>
                              <Switch
                                checked={isSelected}
                                size="small"
                                onChange={() => handleToggleType(type.value)}
                              />
                            </div>
                          </div>
                          <div className="flex items-center justify-between mb-1">
                            <div className="text-white font-bold flex items-center gap-2">
                              {type.label}
                              {type.requiresAdmin && (
                                <AlertCircle className="w-3 h-3 text-amber-400" />
                              )}
                            </div>
                            {sizeText && (
                              <span className={`text-xs px-2 py-0.5 rounded ${isSelected ? 'bg-orange-500/20 text-orange-400' : 'bg-slate-700/50 text-slate-400'}`}>
                                {sizeText}
                              </span>
                            )}
                          </div>
                          <div className="text-xs text-slate-400 line-clamp-1">
                            {type.description}
                          </div>
                        </div>
                      </button>
                    </Tooltip>
                  );
                })}
              </div>
            </div>
          </div>

          {/* 右侧：控制面板 */}
          <div className="space-y-6">
            {/* 清理统计 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight">
              <h3 className="text-lg font-bold text-white mb-4">{t('memoryCleanup.cleanupInfo')}</h3>

              <div className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('memoryCleanup.selectedItems')}</span>
                    <span className="text-white font-medium">{selectedTypes.length} {t('memoryCleanup.items')}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('memoryCleanup.requiresAdmin')}</span>
                    <span className="text-amber-400 font-medium">
                      {memoryTypes.filter((t) => selectedTypes.includes(t.value) && t.requiresAdmin).length} {t('memoryCleanup.items')}
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('memoryCleanup.currentPlatform')}</span>
                    <span className="text-accent font-medium capitalize">{platform}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('memoryCleanup.cleanupMethod')}</span>
                    <span className="text-green-400 font-medium">{t('memoryCleanup.safeCleanup')}</span>
                  </div>
                </div>

                {/* 休眠文件信息 */}
                {memoryInfo?.hibernation_size && (
                  <div className="p-3 bg-slate-800/50 rounded-lg border border-white/5">
                    <div className="text-xs text-slate-400 mb-1">{t('memoryCleanup.types.hibernation.label')}</div>
                    <div className="text-sm font-bold text-white">{memoryInfo.hibernation_size}</div>
                  </div>
                )}
              </div>
            </div>

            {/* 操作按钮 */}
            <div
              className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight"
              style={{ animationDelay: '0.1s' }}
            >
              <Button
                type="primary"
                size="large"
                block
                onClick={handleClean}
                disabled={selectedTypes.length === 0 || loading}
                className="h-12 bg-gradient-to-r from-orange-600 to-orange-500 border-none hover:from-orange-500 hover:to-orange-400 text-white font-bold"
                icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Zap size={20} />}
              >
                {loading ? t('memoryCleanup.cleaning') : t('memoryCleanup.startClean')}
              </Button>

              {loading && (
                <div className="mt-4 space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-slate-300">{t('memoryCleanup.cleanupProgress')}</span>
                    <span className="text-orange-400 font-mono">{Math.floor(progress)}%</span>
                  </div>
                  <Progress
                    percent={progress}
                    strokeColor={{
                      '0%': '#f97316',
                      '100%': '#ef4444',
                    }}
                    trailColor="rgba(255,255,255,0.05)"
                    showInfo={false}
                    strokeWidth={10}
                  />
                  <div className="text-xs text-slate-400 truncate">{currentItem}</div>
                </div>
              )}

              <div className="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
                  <div className="text-xs text-amber-300">
                    {t('memoryCleanup.warnings.adminRequired')}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        )}
      </div>

      {/* 成功弹窗 */}
      <Modal open={showSuccess} onCancel={() => setShowSuccess(false)} footer={null} centered width={500}>
        <div className="text-center py-8">
          <div className="relative inline-block mb-6">
            <div className="absolute inset-0 bg-green-500/20 rounded-full blur-2xl" />
            <div className="relative p-6 bg-gradient-to-br from-green-500/20 to-green-500/5 rounded-full border border-green-500/30">
              <CheckCircle2 className="w-16 h-16 text-green-400" />
            </div>
          </div>

          <h2 className="text-2xl font-bold text-white mb-2">{t('memoryCleanup.cleanComplete')}</h2>
          <p className="text-slate-400 mb-2">
            {t('memoryCleanup.cleanCompleteDesc1')} <span className="text-accent font-bold">{selectedTypes.length}</span> {t('memoryCleanup.cleanCompleteDesc2')}
          </p>
          <p className="text-green-400 text-sm mb-6">{t('memoryCleanup.successMessage')}</p>

          <Button
            type="primary"
            size="large"
            onClick={() => {
              setShowSuccess(false);
              setSelectedTypes(['clipboard']);
            }}
            className="bg-accent hover:bg-accent/80 border-none"
          >
            {t('common.close')}
          </Button>
        </div>
      </Modal>
        <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </ProFeatureGate>
  );
};

export default MemoryCleanup;
