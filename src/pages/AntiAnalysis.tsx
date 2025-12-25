import React, { useState, useEffect } from 'react';
import { Button, Progress, Badge, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Eye,
  Bug,
  Box,
  Server,
  CheckCircle2,
  XCircle,
  Loader2,
  AlertTriangle,
  Shield,
  RefreshCw,
  ArrowLeft,
  Monitor,
  Search,
  Target,
  AlertCircle,
  HardDrive,
  Activity,
  Clock,
  BarChart3,
} from 'lucide-react';

interface DetectionResult {
  name: string;
  detected: boolean;
  details?: string;
  category: string;
  confidence: 'high' | 'medium' | 'low';
}

interface CategoryStats {
  category: string;
  total: number;
  detected: number;
}

interface EnvironmentCheck {
  vm_detected: boolean;
  debugger_detected: boolean;
  sandbox_detected: boolean;
  forensic_tools_detected: boolean;
  details: DetectionResult[];
  category_stats: CategoryStats[];
  platform: string;
  scan_time: string;
}

interface ThreatCategory {
  key: keyof EnvironmentCheck;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  gradient: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
}

const AntiAnalysis: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [checkResult, setCheckResult] = useState<EnvironmentCheck | null>(null);
  const [scanProgress, setScanProgress] = useState(0);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  const threatCategories: ThreatCategory[] = [
    {
      key: 'vm_detected',
      label: t('antiAnalysis.categories.virtualMachine.label'),
      description: t('antiAnalysis.categories.virtualMachine.description'),
      icon: <Server size={24} />,
      color: 'text-red-400',
      gradient: 'from-red-500 to-rose-500',
      severity: 'critical',
    },
    {
      key: 'debugger_detected',
      label: t('antiAnalysis.categories.debugger.label'),
      description: t('antiAnalysis.categories.debugger.description'),
      icon: <Bug size={24} />,
      color: 'text-orange-400',
      gradient: 'from-orange-500 to-amber-500',
      severity: 'high',
    },
    {
      key: 'sandbox_detected',
      label: t('antiAnalysis.categories.sandbox.label'),
      description: t('antiAnalysis.categories.sandbox.description'),
      icon: <Box size={24} />,
      color: 'text-blue-400',
      gradient: 'from-blue-500 to-cyan-500',
      severity: 'medium',
    },
    {
      key: 'forensic_tools_detected',
      label: t('antiAnalysis.categories.forensicTools.label'),
      description: t('antiAnalysis.categories.forensicTools.description'),
      icon: <Eye size={24} />,
      color: 'text-purple-400',
      gradient: 'from-purple-500 to-pink-500',
      severity: 'high',
    },
  ];

  const handleCheck = async () => {
    setLoading(true);
    setScanProgress(0);

    // 模拟扫描进度
    const interval = setInterval(() => {
      setScanProgress(prev => {
        if (prev >= 90) {
          clearInterval(interval);
          return 90;
        }
        return prev + 15;
      });
    }, 200);

    try {
      const result = await invoke<EnvironmentCheck>('check_environment');
      clearInterval(interval);
      setScanProgress(100);
      setCheckResult(result);

      setTimeout(() => {
        setLoading(false);
      }, 500);
    } catch {
      clearInterval(interval);
      setLoading(false);
    }
  };

  useEffect(() => {
    handleCheck();
  }, []);

  const getOverallStatus = () => {
    if (!checkResult) return 'unknown';
    const detectedCount = [
      checkResult.vm_detected,
      checkResult.debugger_detected,
      checkResult.sandbox_detected,
      checkResult.forensic_tools_detected,
    ].filter(Boolean).length;

    if (detectedCount === 0) return 'safe';
    if (detectedCount <= 2) return 'warning';
    return 'danger';
  };

  const getThreatCount = () => {
    if (!checkResult) return 0;
    return [
      checkResult.vm_detected,
      checkResult.debugger_detected,
      checkResult.sandbox_detected,
      checkResult.forensic_tools_detected,
    ].filter(Boolean).length;
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'text-red-400 bg-red-500/10 border-red-500/30';
      case 'high': return 'text-orange-400 bg-orange-500/10 border-orange-500/30';
      case 'medium': return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
      case 'low': return 'text-green-400 bg-green-500/10 border-green-500/30';
      default: return 'text-gray-400 bg-gray-500/10 border-gray-500/30';
    }
  };

  const getCategoryIcon = (category: string) => {
    // 先尝试翻译类别名称
    const translatedCategory = t(`antiAnalysis.categoryNames.${category}`, { defaultValue: category });

    const categoryMap: Record<string, React.ReactNode> = {
      [t('antiAnalysis.categoryNames.vm')]: <Server size={16} />,
      [t('antiAnalysis.categoryNames.debugger')]: <Bug size={16} />,
      [t('antiAnalysis.categoryNames.sandbox')]: <Box size={16} />,
      [t('antiAnalysis.categoryNames.forensic')]: <Eye size={16} />,
    };
    return categoryMap[translatedCategory] || <Activity size={16} />;
  };

  // 翻译检测项名称
  const translateDetectionName = (name: string): string => {
    return t(`antiAnalysis.detectionNames.${name}`, { defaultValue: name });
  };

  // 翻译类别名称
  const translateCategoryName = (category: string): string => {
    return t(`antiAnalysis.categoryNames.${category}`, { defaultValue: category });
  };

  // 翻译检测详情
  const translateDetails = (details: string | undefined): string => {
    if (!details) return '';

    // 处理各种详情格式的翻译
    const patterns: Array<{ regex: RegExp; replacement: string | ((match: RegExpMatchArray) => string) }> = [
      // CPU和内存信息
      { regex: /CPU:\s*(\d+)\s*核,\s*内存:\s*([\d.]+)\s*GB/, replacement: (m) => `CPU: ${m[1]} ${t('antiAnalysis.detailPatterns.cores')}, ${t('antiAnalysis.detailPatterns.memory')}: ${m[2]} GB` },
      { regex: /CPU:\s*(\d+)\s*核/, replacement: (m) => `CPU: ${m[1]} ${t('antiAnalysis.detailPatterns.cores')}` },
      { regex: /内存:\s*([\d.]+)\s*GB/, replacement: (m) => `${t('antiAnalysis.detailPatterns.memory')}: ${m[1]} GB` },
      // 用户信息
      { regex: /当前用户:\s*(.+)/, replacement: (m) => `${t('antiAnalysis.detailPatterns.currentUser')}: ${m[1]}` },
      // 检测结果
      { regex: /^未检测到可疑文件$/, replacement: () => t('antiAnalysis.detailPatterns.noSuspiciousFiles') },
      { regex: /^未检测到调试器$/, replacement: () => t('antiAnalysis.detailPatterns.noDebuggerDetected') },
      { regex: /^未检测到沙箱环境$/, replacement: () => t('antiAnalysis.detailPatterns.noSandboxDetected') },
      { regex: /^未检测到虚拟机$/, replacement: () => t('antiAnalysis.detailPatterns.noVmDetected') },
      { regex: /^未检测到取证工具$/, replacement: () => t('antiAnalysis.detailPatterns.noForensicTools') },
      { regex: /^未在沙箱中运行$/, replacement: () => t('antiAnalysis.detailPatterns.notInSandbox') },
      { regex: /^系统完整性保护正常$/, replacement: () => t('antiAnalysis.detailPatterns.sipNormal') },
      { regex: /^未检测到容器环境$/, replacement: () => t('antiAnalysis.detailPatterns.noContainerDetected') },
      { regex: /^物理硬件$/, replacement: () => t('antiAnalysis.detailPatterns.physicalHardware') },
      // 硬件模型（后端返回"硬件模型"而非"硬件型号"）
      { regex: /硬件模型:\s*(.+)/, replacement: (m) => `${t('antiAnalysis.detailPatterns.hardwareModel')}: ${m[1]}` },
      // MAC地址
      { regex: /MAC地址正常/, replacement: () => t('antiAnalysis.detailPatterns.macAddressNormal') },
      // 更多检测结果
      { regex: /^未检测到虚拟化设备$/, replacement: () => t('antiAnalysis.detailPatterns.noVirtualizationDevice') },
      { regex: /^SMC 控制器正常$/, replacement: () => t('antiAnalysis.detailPatterns.smcNormal') },
      { regex: /^未检测到虚拟机进程$/, replacement: () => t('antiAnalysis.detailPatterns.noVmProcess') },
      { regex: /^未运行在容器中$/, replacement: () => t('antiAnalysis.detailPatterns.notInContainer') },
      { regex: /^未检测到调试端口$/, replacement: () => t('antiAnalysis.detailPatterns.noDebugPort') },
      { regex: /^未被追踪$/, replacement: () => t('antiAnalysis.detailPatterns.notTraced') },
      { regex: /^未检测到调试器进程$/, replacement: () => t('antiAnalysis.detailPatterns.noDebuggerProcess') },
      { regex: /^未检测到断点$/, replacement: () => t('antiAnalysis.detailPatterns.noBreakpoints') },
      { regex: /^未检测到监控工具$/, replacement: () => t('antiAnalysis.detailPatterns.noMonitoringTools') },
      { regex: /^未检测到抓包工具$/, replacement: () => t('antiAnalysis.detailPatterns.noPacketCapture') },
      // App Sandbox
      { regex: /^未在 App Sandbox 中运行$/, replacement: () => t('antiAnalysis.detailPatterns.notInAppSandbox') },
      // 检测到的工具
      { regex: /^检测到:\s*(.+)$/, replacement: (m) => `${t('antiAnalysis.detailPatterns.detected')}: ${m[1]}` },
      // 调试器相关
      { regex: /^未检测到调试器附加$/, replacement: () => t('antiAnalysis.detailPatterns.noDebuggerAttached') },
    ];

    for (const pattern of patterns) {
      const match = details.match(pattern.regex);
      if (match) {
        if (typeof pattern.replacement === 'function') {
          return pattern.replacement(match);
        }
        return pattern.replacement;
      }
    }

    return details;
  };

  const getConfidenceStyle = (confidence: 'high' | 'medium' | 'low') => {
    switch (confidence) {
      case 'high': return { bg: 'bg-red-500/20', text: 'text-red-400', label: t('antiAnalysis.confidence.high') };
      case 'medium': return { bg: 'bg-amber-500/20', text: 'text-amber-400', label: t('antiAnalysis.confidence.medium') };
      case 'low': return { bg: 'bg-blue-500/20', text: 'text-blue-400', label: t('antiAnalysis.confidence.low') };
    }
  };

  // 根据分类过滤详细检测项
  const getFilteredDetails = () => {
    if (!checkResult) return [];
    if (!selectedCategory) return checkResult.details;
    return checkResult.details.filter(d => d.category === selectedCategory);
  };

  return (
    <ProFeatureGate
      feature="anti_analysis"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/2 left-1/2 w-96 h-96 bg-cyan-500/5 rounded-full blur-3xl animate-pulse" />
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
                <Shield className="w-7 h-7 text-cyan-400" />
                {t('antiAnalysis.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('antiAnalysis.subtitle')}
              </p>
            </div>
          </div>

          {checkResult && !loading && (
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 px-3 py-1.5 bg-slate-800/50 border border-white/10 rounded-lg">
                <HardDrive className="w-4 h-4 text-accent" />
                <span className="text-sm text-slate-300">{checkResult.platform}</span>
              </div>
              <div className="flex items-center gap-2 px-3 py-1.5 bg-slate-800/50 border border-white/10 rounded-lg">
                <Clock className="w-4 h-4 text-blue-400" />
                <span className="text-sm text-slate-300">{checkResult.scan_time}</span>
              </div>
            </div>
          )}
        </div>

        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-0 overflow-y-auto custom-scrollbar">
          {/* 左侧：状态总览 */}
          <div className="flex flex-col space-y-4">
            {/* 整体状态 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="text-center">
                {loading ? (
                  <div>
                    <div className="relative inline-block mb-4">
                      <div className="absolute inset-0 bg-cyan-500/20 rounded-full blur-2xl animate-pulse" />
                      <div className="relative p-6 bg-gradient-to-br from-cyan-500/20 to-cyan-500/5 rounded-full border border-cyan-500/30">
                        <Loader2 className="w-12 h-12 text-cyan-400 animate-spin" />
                      </div>
                    </div>
                    <div className="text-white font-bold mb-2">{t('antiAnalysis.checking')}</div>
                    <div className="text-sm text-slate-400 mb-4">{t('antiAnalysis.detectingThreats')}</div>
                    <Progress
                      percent={scanProgress}
                      strokeColor={{
                        '0%': '#06b6d4',
                        '100%': '#0891b2',
                      }}
                      trailColor="rgba(255,255,255,0.05)"
                      showInfo={false}
                      strokeWidth={8}
                    />
                  </div>
                ) : (
                  <div>
                    <div className="relative inline-block mb-4">
                      <div className={`absolute inset-0 ${getOverallStatus() === 'safe' ? 'bg-green-500/20' : 'bg-red-500/20'} rounded-full blur-2xl`} />
                      <div className={`relative p-6 bg-gradient-to-br ${getOverallStatus() === 'safe' ? 'from-green-500/20 to-green-500/5 border-green-500/30' : 'from-red-500/20 to-red-500/5 border-red-500/30'} rounded-full border`}>
                        {getOverallStatus() === 'safe' ? (
                          <CheckCircle2 className="w-12 h-12 text-green-400" />
                        ) : (
                          <AlertCircle className="w-12 h-12 text-red-400" />
                        )}
                      </div>
                    </div>
                    <div className={`text-xl font-bold mb-2 ${getOverallStatus() === 'safe' ? 'text-green-400' : 'text-red-400'}`}>
                      {getOverallStatus() === 'safe' ? t('antiAnalysis.status.safe') : t('antiAnalysis.status.threatsDetected')}
                    </div>
                    <div className="text-sm text-slate-400">
                      {getThreatCount()} {t('antiAnalysis.threatsCount')}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* 快速操作 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.1s' }}>
              <h3 className="text-lg font-bold text-white mb-4">{t('common.operation')}</h3>
              <Button
                type="primary"
                size="large"
                block
                onClick={handleCheck}
                disabled={loading}
                className="h-12 bg-gradient-to-r from-cyan-600 to-cyan-500 border-none hover:from-cyan-500 hover:to-cyan-400 text-white font-bold"
                icon={loading ? <Loader2 className="animate-spin" size={20} /> : <RefreshCw size={20} />}
              >
                {loading ? t('antiAnalysis.scanning') : t('antiAnalysis.recheckEnvironment')}
              </Button>

              {!loading && checkResult && getOverallStatus() !== 'safe' && (
                <div className="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
                  <div className="flex items-start gap-2">
                    <AlertTriangle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
                    <div className="text-xs text-red-300">
                      <div className="font-bold mb-1">{t('antiAnalysis.warnings.threatDetected')}</div>
                      {t('antiAnalysis.warnings.environmentCompromised')}
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 分类统计 */}
            {!loading && checkResult && checkResult.category_stats && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.2s' }}>
                <div className="flex items-center gap-2 mb-4">
                  <BarChart3 className="w-5 h-5 text-blue-400" />
                  <h3 className="text-lg font-bold text-white">{t('antiAnalysis.detectionStats')}</h3>
                </div>
                <div className="space-y-2 max-h-[200px] overflow-y-auto custom-scrollbar">
                  {checkResult.category_stats.map((stat) => {
                    const translatedCategory = translateCategoryName(stat.category);
                    const categoryIcon = getCategoryIcon(stat.category);
                    const hasDetection = stat.detected > 0;
                    return (
                      <Tooltip key={stat.category} title={`${t('antiAnalysis.clickToView')} ${translatedCategory} ${t('antiAnalysis.details')}`}>
                        <button
                          onClick={() => setSelectedCategory(selectedCategory === stat.category ? null : stat.category)}
                          className={`w-full p-3 rounded-lg border transition-all text-left ${selectedCategory === stat.category
                            ? 'border-accent/50 bg-accent/10'
                            : hasDetection
                              ? 'border-red-500/30 bg-red-500/5 hover:bg-red-500/10'
                              : 'border-white/10 bg-slate-800/30 hover:bg-slate-800/50'
                            }`}
                        >
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <div className={`p-1.5 rounded-lg ${hasDetection ? 'bg-red-500/20 text-red-400' : 'bg-slate-700/50 text-slate-400'}`}>
                                {categoryIcon}
                              </div>
                              <span className="text-sm text-white font-medium">{translatedCategory}</span>
                            </div>
                            <div className="flex items-center gap-2">
                              <span className={`text-xs px-2 py-0.5 rounded ${hasDetection ? 'bg-red-500/20 text-red-400' : 'bg-green-500/20 text-green-400'}`}>
                                {stat.detected}/{stat.total}
                              </span>
                              {hasDetection ? (
                                <XCircle size={14} className="text-red-400" />
                              ) : (
                                <CheckCircle2 size={14} className="text-green-400" />
                              )}
                            </div>
                          </div>
                        </button>
                      </Tooltip>
                    );
                  })}
                </div>
              </div>
            )}
          </div>

          {/* 中间：威胁类别 */}
          <div className="lg:col-span-2 flex flex-col space-y-4">
            {/* 威胁检测结果 */}
            <div className="flex-1 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight min-h-0 flex flex-col">
              <div className="flex items-center gap-2 mb-4">
                <Target className="w-5 h-5 text-accent" />
                <h3 className="text-lg font-bold text-white">{t('antiAnalysis.threatCategories')}</h3>
              </div>

              {!loading && checkResult ? (
                <div className="flex-1 overflow-y-auto custom-scrollbar space-y-3">
                  {threatCategories.map((category) => {
                    const detected = checkResult[category.key];
                    return (
                      <div
                        key={category.key}
                        className={`
                          relative p-4 rounded-xl border-2 transition-all duration-300
                          ${detected
                            ? 'border-red-400/30 bg-red-500/10 shadow-lg shadow-red-500/20'
                            : 'border-green-400/30 bg-green-500/5'
                          }
                        `}
                      >
                        <div className={`absolute inset-0 bg-gradient-to-br ${category.gradient} opacity-0 ${detected ? 'opacity-5' : ''} transition-opacity`} />

                        <div className="relative z-10">
                          <div className="flex items-center justify-between mb-3">
                            <div className="flex items-center gap-3">
                              <div className={`p-2 rounded-lg bg-slate-700/50 ${category.color}`}>
                                {category.icon}
                              </div>
                              <div>
                                <div className="text-white font-bold">{category.label}</div>
                                <div className="text-xs text-slate-400">{category.description}</div>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <div className={`px-2 py-1 rounded-lg text-xs font-medium border ${getSeverityColor(category.severity)}`}>
                                {category.severity === 'critical' && t('antiAnalysis.severity.critical')}
                                {category.severity === 'high' && t('antiAnalysis.severity.high')}
                                {category.severity === 'medium' && t('antiAnalysis.severity.medium')}
                                {category.severity === 'low' && t('antiAnalysis.severity.low')}
                              </div>
                              {detected ? (
                                <XCircle className="w-5 h-5 text-red-400" />
                              ) : (
                                <CheckCircle2 className="w-5 h-5 text-green-400" />
                              )}
                            </div>
                          </div>

                          <div className={`px-3 py-2 rounded-lg text-sm font-medium ${detected ? 'bg-red-500/10 text-red-300' : 'bg-green-500/10 text-green-300'}`}>
                            {detected ? `✗ ${t('antiAnalysis.detected')}` : `✓ ${t('antiAnalysis.notDetected')}`}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div className="flex-1 flex items-center justify-center">
                  <div className="text-center text-slate-500">
                    <Monitor className="w-16 h-16 mx-auto mb-4 opacity-20" />
                    <div>{t('antiAnalysis.detectingEnvironment')}</div>
                  </div>
                </div>
              )}
            </div>

            {/* 详细检测项 */}
            {!loading && checkResult && checkResult.details && checkResult.details.length > 0 && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.1s' }}>
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <Search className="w-5 h-5 text-blue-400" />
                    <h3 className="text-lg font-bold text-white">{t('antiAnalysis.detailResults')}</h3>
                    <Badge count={getFilteredDetails().length} overflowCount={99} color="#3b82f6" />
                  </div>
                  {selectedCategory && (
                    <button
                      onClick={() => setSelectedCategory(null)}
                      className="text-xs text-slate-400 hover:text-white flex items-center gap-1"
                    >
                      <XCircle size={14} />
                      {t('antiAnalysis.clearFilter')}
                    </button>
                  )}
                </div>

                {selectedCategory && (
                  <div className="mb-3 flex items-center gap-2">
                    <span className="text-xs text-slate-400">{t('antiAnalysis.currentFilter')}:</span>
                    <span className="text-xs px-2 py-1 bg-accent/20 text-accent rounded-lg flex items-center gap-1">
                      {getCategoryIcon(selectedCategory)}
                      {translateCategoryName(selectedCategory)}
                    </span>
                  </div>
                )}

                <div className="max-h-64 overflow-y-auto custom-scrollbar space-y-2">
                  {getFilteredDetails().map((item, index) => {
                    const confidenceStyle = getConfidenceStyle(item.confidence);
                    const translatedName = translateDetectionName(item.name);
                    const translatedCategory = translateCategoryName(item.category);
                    return (
                    <div
                      key={index}
                      className={`p-3 rounded-lg border transition-all ${item.detected
                        ? 'border-red-500/30 bg-red-500/5 hover:bg-red-500/10'
                        : 'border-slate-700 bg-slate-800/30 hover:bg-slate-800/50'
                        }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2 flex-1 min-w-0">
                          {item.detected ? (
                            <XCircle size={16} className="text-red-400 flex-shrink-0" />
                          ) : (
                            <CheckCircle2 size={16} className="text-green-400 flex-shrink-0" />
                          )}
                          <span className="text-sm text-white font-medium truncate">{translatedName}</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <Tooltip title={`${t('antiAnalysis.confidenceLabel')}: ${confidenceStyle.label}`}>
                            <span className={`text-xs px-1.5 py-0.5 rounded ${confidenceStyle.bg} ${confidenceStyle.text}`}>
                              {confidenceStyle.label}
                            </span>
                          </Tooltip>
                          <span className="text-xs px-2 py-0.5 rounded bg-slate-700/50 text-slate-400">
                            {translatedCategory}
                          </span>
                          <Badge status={item.detected ? 'error' : 'success'} />
                        </div>
                      </div>
                      {item.details && (
                        <p className="text-xs text-slate-400 mt-1 ml-6">{translateDetails(item.details)}</p>
                      )}
                    </div>
                  )})}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
      </div>
      <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
    </ProFeatureGate>
  );
};

export default AntiAnalysis;
