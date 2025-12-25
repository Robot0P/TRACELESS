import React, { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  Shield,
  Search,
  CheckCircle2,
  AlertTriangle,
  Trash2,
  FileText,
  Cpu,
  Database,
  Clock,
  Eye,
  Play,
  Loader2,
  AlertCircle,
  ArrowLeft,
  Globe,
  Terminal,
  HardDrive,
  Download,
  FolderOpen,
  Wifi,
  Monitor,
} from 'lucide-react';
import { Progress, Checkbox, Button, Modal } from 'antd';

interface ScanResult {
  id: string;
  category: string;
  type: string;
  description: string;
  severity: 'high' | 'medium' | 'low';
  path?: string;
  size?: string;
  icon: React.ReactNode;
  color: string;
}

const ScanPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const { t } = useTranslation();

  // 从路由获取扫描模式
  const scanMode = (location.state as { mode?: 'smart' | 'full' })?.mode || 'smart';

  const [scanning, setScanning] = useState(true);
  const [scanProgress, setScanProgress] = useState(0);
  const [currentScanItem, setCurrentScanItem] = useState('');
  const [scanResults, setScanResults] = useState<ScanResult[]>([]);
  const [selectedItems, setSelectedItems] = useState<string[]>([]);
  const [processing, setProcessing] = useState(false);
  const [processProgress, setProcessProgress] = useState(0);
  const [showSuccessModal, setShowSuccessModal] = useState(false);

  // 扫描时间跟踪
  const [, setScanStartTime] = useState<number>(0);
  const [elapsedTime, setElapsedTime] = useState<number>(0);
  const [checkedItems, setCheckedItems] = useState<number>(0);

  // 扫描状态展示文字
  const scanItems = [
    t('scanPage.scanItems.systemLogs'),
    t('scanPage.scanItems.tempFiles'),
    t('scanPage.scanItems.browserHistory'),
    t('scanPage.scanItems.recentDocs'),
    t('scanPage.scanItems.networkCache'),
    t('scanPage.scanItems.shellHistory'),
    t('scanPage.scanItems.appCache'),
    t('scanPage.scanItems.crashLogs'),
    t('scanPage.scanItems.downloads'),
    t('scanPage.scanItems.trash'),
    t('scanPage.scanItems.sshRecords'),
    t('scanPage.scanItems.vmEnvironment'),
  ];

  // 翻译后端返回的类别名称
  const translateCategory = (category: string): string => {
    const key = `scanPage.categories.${category}`;
    const translated = t(key);
    // 如果翻译键不存在，返回原文
    return translated === key ? category : translated;
  };

  // 翻译后端返回的类型名称
  const translateType = (type: string): string => {
    const key = `scanPage.types.${type}`;
    const translated = t(key);
    // 如果翻译键不存在，返回原文
    return translated === key ? type : translated;
  };

  // 翻译后端返回的描述
  // 后端返回的描述始终是中文，这里根据描述模式匹配并翻译
  const translateDescription = (description: string): string => {
    // 首先检查是否有直接的翻译键（用于浏览器特定描述）
    const directKey = `scanPage.descriptions.${description}`;
    const directTranslation = t(directKey);
    if (directTranslation !== directKey) {
      return directTranslation;
    }

    // 匹配 "发现 X 个/条Y" 模式
    const countMatch = description.match(/发现\s*(\d+)\s*(?:个|条)?\s*(.+)/);
    if (countMatch) {
      const count = parseInt(countMatch[1]);
      let itemType = countMatch[2].trim();

      // 移除可能的量词前缀
      itemType = itemType.replace(/^条/, '');

      // 根据项目类型返回翻译
      const typeMap: Record<string, string> = {
        '日志文件': 'foundLogFiles',
        '临时文件': 'foundTempFiles',
        '浏览记录/缓存/Cookie': 'foundBrowsingData',
        '缓存文件': 'foundCacheFiles',
        '已删除文件': 'foundDeletedFiles',
        '崩溃报告': 'foundCrashReports',
        '下载文件': 'foundDownloadFiles',
        'SSH 连接记录': 'foundSshRecords',
        '命令记录': 'foundCommands',
        'DNS 缓存记录': 'foundDnsRecords',
        '统一日志文件': 'foundUnifiedLogs',
      };

      const key = typeMap[itemType];
      if (key) {
        return t(`scanPage.descriptions.${key}`, { count });
      }

      // 尝试翻译项目类型
      const itemTranslations: Record<string, string> = {
        '日志文件': 'log files',
        '临时文件': 'temp files',
        '缓存文件': 'cache files',
        '已删除文件': 'deleted files',
        '崩溃报告': 'crash reports',
        '下载文件': 'download files',
        '命令记录': 'command records',
        '统一日志文件': 'unified log files',
      };
      const translatedItem = itemTranslations[itemType] || itemType;

      // 通用模式：发现 X 个 Y
      return t('scanPage.descriptions.foundItems', { count, item: translatedItem });
    }

    // 检测特定描述的翻译
    const descMap: Record<string, string> = {
      '检测到虚拟机环境': 'vmDetected',
      '未检测到虚拟机环境': 'vmNotDetected',
      'DNS 缓存可能存在记录': 'dnsCacheMayExist',
      '发现最近使用记录': 'foundRecentRecords',
      '发现 Safari 浏览记录/缓存/Cookie': 'foundSafariBrowsingData',
      '发现 Chrome 浏览记录/缓存/Cookie': 'foundChromeBrowsingData',
      '发现 Firefox 浏览记录/缓存/Cookie': 'foundFirefoxBrowsingData',
      '发现 Edge 浏览记录/缓存/Cookie': 'foundEdgeBrowsingData',
      '发现 Arc 浏览记录/缓存/Cookie': 'foundArcBrowsingData',
      '发现 Brave 浏览记录/缓存/Cookie': 'foundBraveBrowsingData',
      '发现 Vivaldi 浏览记录/缓存/Cookie': 'foundVivaldiBrowsingData',
      '发现 Opera 浏览记录/缓存/Cookie': 'foundOperaBrowsingData',
      '发现 Chromium 浏览记录/缓存/Cookie': 'foundChromiumBrowsingData',
    };

    const mappedKey = descMap[description];
    if (mappedKey) {
      return t(`scanPage.descriptions.${mappedKey}`);
    }

    // 返回原文
    return description;
  };

  // 根据类别获取图标和颜色（基于后端返回的中文类别名）
  const getCategoryStyle = (category: string): { icon: React.ReactNode; color: string } => {
    const styles: Record<string, { icon: React.ReactNode; color: string }> = {
      '系统日志': { icon: <FileText size={20} />, color: 'text-purple-400' },
      '临时文件': { icon: <Trash2 size={20} />, color: 'text-blue-400' },
      '浏览器数据': { icon: <Globe size={20} />, color: 'text-green-400' },
      '最近文档': { icon: <FolderOpen size={20} />, color: 'text-orange-400' },
      '网络缓存': { icon: <Wifi size={20} />, color: 'text-cyan-400' },
      '命令历史': { icon: <Terminal size={20} />, color: 'text-yellow-400' },
      '应用缓存': { icon: <HardDrive size={20} />, color: 'text-indigo-400' },
      '崩溃日志': { icon: <AlertTriangle size={20} />, color: 'text-red-400' },
      '下载记录': { icon: <Download size={20} />, color: 'text-pink-400' },
      '回收站': { icon: <Trash2 size={20} />, color: 'text-amber-400' },
      'SSH 记录': { icon: <Terminal size={20} />, color: 'text-emerald-400' },
      '环境检测': { icon: <Monitor size={20} />, color: 'text-slate-400' },
      '内存痕迹': { icon: <Cpu size={20} />, color: 'text-orange-400' },
      '注册表': { icon: <Database size={20} />, color: 'text-pink-400' },
      '文件时间戳': { icon: <Clock size={20} />, color: 'text-indigo-400' },
      '反分析': { icon: <Eye size={20} />, color: 'text-cyan-400' },
    };
    return styles[category] || { icon: <FileText size={20} />, color: 'text-slate-400' };
  };

  // 执行扫描
  useEffect(() => {
    if (!scanning) return;

    // 记录扫描开始时间
    const startTime = Date.now();
    setScanStartTime(startTime);

    // 时间更新定时器
    const timeInterval = setInterval(() => {
      setElapsedTime(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);

    const performScan = async () => {
      const totalSteps = scanMode === 'smart' ? 20 : 50;
      const stepDuration = scanMode === 'smart' ? 100 : 200;
      let itemCount = 0;

      // 模拟进度更新
      const progressInterval = setInterval(() => {
        setScanProgress((prev) => {
          if (prev >= 90) {
            return prev; // 在90%停止，等待真实扫描完成
          }

          // 更新当前扫描项
          const itemIndex = Math.floor((prev / 100) * scanItems.length);
          if (itemIndex < scanItems.length) {
            setCurrentScanItem(scanItems[itemIndex]);
          }

          // 更新已检查项目数
          itemCount++;
          setCheckedItems(itemCount);

          return prev + (90 / totalSteps);
        });
      }, stepDuration);

      try {
        // 调用真实的后端扫描 API
        const results = await invoke<any[]>('perform_system_scan', { mode: scanMode });

        // 清除进度更新
        clearInterval(progressInterval);
        clearInterval(timeInterval);
        setScanProgress(100);
        setScanning(false);
        // 记录最终的已检查项目数
        setCheckedItems(results.length > 0 ? results.length * 10 : itemCount);

        // 转换后端结果为前端格式
        const formattedResults = results.map((result) => {
          // 使用统一的样式获取函数
          const style = getCategoryStyle(result.category);

          return {
            id: result.id,
            category: result.category,
            type: result.item_type,
            description: result.description,
            severity: result.severity,
            path: result.path,
            size: result.size,
            icon: style.icon,
            color: style.color,
          } as ScanResult;
        });

        setScanResults(formattedResults);

        // 默认全选高危项
        const highSeverityIds = formattedResults
          .filter(r => r.severity === 'high')
          .map(r => r.id);
        setSelectedItems(highSeverityIds);
      } catch (error) {
        clearInterval(progressInterval);
        clearInterval(timeInterval);
        setScanProgress(100);
        setScanning(false);
        // 显示错误提示
        Modal.error({
          title: t('scanPage.errors.scanFailed'),
          content: String(error),
          centered: true,
        });
        setScanResults([]);
        setSelectedItems([]);
      }
    };

    performScan();

    return () => {
      // 清理定时器
      clearInterval(timeInterval);
    };
  }, [scanning, scanMode]);

  // 处理全选/取消全选
  const handleSelectAll = () => {
    if (selectedItems.length === scanResults.length) {
      setSelectedItems([]);
    } else {
      setSelectedItems(scanResults.map(r => r.id));
    }
  };

  // 处理单个项目选择
  const handleSelectItem = (id: string) => {
    setSelectedItems(prev =>
      prev.includes(id)
        ? prev.filter(i => i !== id)
        : [...prev, id]
    );
  };

  // 处理清理
  const handleCleanup = async () => {
    if (selectedItems.length === 0) return;

    setProcessing(true);
    setProcessProgress(0);

    // 模拟进度更新
    const progressInterval = setInterval(() => {
      setProcessProgress(prev => {
        if (prev >= 90) {
          return prev; // 在90%停止，等待真实清理完成
        }
        return prev + 10;
      });
    }, 200);

    try {
      // 构建清理项目列表（包含路径信息）
      const cleanupItems = selectedItems.map(id => {
        const result = scanResults.find(r => r.id === id);
        return {
          id,
          path: result?.path || null,
        };
      });

      // 调用真实的后端清理 API
      await invoke('cleanup_scan_items', { items: cleanupItems });

      // 清除进度更新
      clearInterval(progressInterval);
      setProcessProgress(100);
      setProcessing(false);
      setShowSuccessModal(true);
    } catch {
      clearInterval(progressInterval);
      setProcessing(false);
    }
  };

  // 格式化耗时
  const formatElapsedTime = (seconds: number) => {
    if (seconds < 60) {
      return t('scanPage.time.seconds', { count: seconds });
    }
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return t('scanPage.time.minutesSeconds', { minutes, seconds: secs });
  };

  // 获取严重程度颜色
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'high': return 'text-red-400 bg-red-500/10 border-red-500/30';
      case 'medium': return 'text-amber-400 bg-amber-500/10 border-amber-500/30';
      case 'low': return 'text-green-400 bg-green-500/10 border-green-500/30';
      default: return 'text-gray-400 bg-gray-500/10 border-gray-500/30';
    }
  };

  // 获取严重程度图标
  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'high': return <AlertTriangle size={16} className="text-red-400" />;
      case 'medium': return <AlertCircle size={16} className="text-amber-400" />;
      case 'low': return <CheckCircle2 size={16} className="text-green-400" />;
      default: return null;
    }
  };

  // 分组结果
  const groupedResults = scanResults.reduce((acc, result) => {
    if (!acc[result.category]) {
      acc[result.category] = [];
    }
    acc[result.category].push(result);
    return acc;
  }, {} as Record<string, ScanResult[]>);

  return (
    <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-accent/5 rounded-full blur-3xl animate-pulse" />
      </div>

      <div className="flex-1 flex flex-col p-6 relative z-10 max-w-7xl mx-auto w-full min-h-0">
        {/* 顶部导航栏 */}
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
                <Search className="w-7 h-7 text-accent" />
                {scanMode === 'smart' ? t('scanPage.smartScan') : t('scanPage.fullScan')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {scanMode === 'smart'
                  ? t('scanPage.smartScanDesc')
                  : t('scanPage.fullScanDesc')}
              </p>
            </div>
          </div>

          {!scanning && scanResults.length > 0 && (
            <div className="flex items-center gap-3">
              <div className="text-right">
                <div className="text-sm text-slate-400">{t('scanPage.detected')}</div>
                <div className="text-2xl font-bold text-accent">{scanResults.length}</div>
              </div>
            </div>
          )}
        </div>

        {/* 扫描中 */}
        {scanning && (
          <div className="flex-1 flex items-center justify-center">
            <div className="w-full max-w-2xl space-y-8 animate-slideInUp">
              <div className="text-center">
                <div className="relative inline-block mb-6">
                  <div className="absolute inset-0 bg-accent/20 rounded-full blur-2xl animate-pulse" />
                  <div className="relative p-8 bg-gradient-to-br from-accent/20 to-accent/5 rounded-full border border-accent/30">
                    <Search className="w-16 h-16 text-accent animate-scanPulse" />
                  </div>
                </div>
                <h2 className="text-2xl font-bold text-white mb-2">{t('scanPage.scanning')}</h2>
                <p className="text-slate-400">{currentScanItem}</p>
              </div>

              <div>
                <div className="flex items-center justify-between mb-3">
                  <span className="text-sm text-slate-300">{t('scanPage.scanProgress')}</span>
                  <span className="text-sm font-mono text-accent">{Math.floor(scanProgress)}%</span>
                </div>
                <Progress
                  percent={scanProgress}
                  strokeColor={{
                    '0%': '#d9943f',
                    '100%': '#f59e0b',
                  }}
                  trailColor="rgba(255,255,255,0.05)"
                  showInfo={false}
                  strokeWidth={12}
                  className="scan-progress"
                />
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div className="text-center p-4 bg-slate-800/30 rounded-lg border border-white/5">
                  <p className="text-xl font-bold text-white">{checkedItems}</p>
                  <p className="text-xs text-slate-400 mt-1">{t('scanPage.checkedItems')}</p>
                </div>
                <div className="text-center p-4 bg-slate-800/30 rounded-lg border border-white/5">
                  <p className="text-xl font-bold text-accent">{Math.floor(scanProgress / 10)}</p>
                  <p className="text-xs text-slate-400 mt-1">{t('scanPage.tracesFound')}</p>
                </div>
                <div className="text-center p-4 bg-slate-800/30 rounded-lg border border-white/5">
                  <p className="text-xl font-bold text-blue-400">{formatElapsedTime(elapsedTime)}</p>
                  <p className="text-xs text-slate-400 mt-1">{t('scanPage.elapsedTime')}</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* 扫描结果 */}
        {!scanning && !processing && (
          <div className="flex-1 flex flex-col animate-slideInUp min-h-0 overflow-hidden">
            {/* 操作栏 */}
            <div className="flex items-center justify-between mb-4 p-4 bg-slate-800/30 rounded-lg border border-white/5 flex-shrink-0">
              <div className="flex items-center gap-4">
                <Checkbox
                  checked={selectedItems.length === scanResults.length}
                  indeterminate={selectedItems.length > 0 && selectedItems.length < scanResults.length}
                  onChange={handleSelectAll}
                  className="custom-checkbox"
                >
                  <span className="text-white font-medium">
                    {t('scanPage.selectAll')} ({selectedItems.length}/{scanResults.length})
                  </span>
                </Checkbox>
              </div>

              <Button
                type="primary"
                size="large"
                disabled={selectedItems.length === 0}
                onClick={handleCleanup}
                className="bg-accent hover:bg-accent/80 border-none"
                icon={<Play size={16} />}
              >
                {t('scanPage.cleanSelected')} ({selectedItems.length})
              </Button>
            </div>

            {/* 结果列表 */}
            <div className="flex-1 overflow-y-auto overflow-x-hidden custom-scrollbar space-y-6">
              {Object.entries(groupedResults).map(([category, items]) => (
                <div key={category} className="space-y-3">
                  <div className="flex items-center gap-2 px-2">
                    <Shield className="w-5 h-5 text-accent" />
                    <h3 className="text-lg font-bold text-white">{translateCategory(category)}</h3>
                    <span className="text-sm text-slate-400">({items.length})</span>
                  </div>

                  {items.map((result) => (
                    <div
                      key={result.id}
                      className={`
                        p-4 bg-slate-800/30 rounded-lg border transition-all duration-300 cursor-pointer group
                        hover:scale-[1.01] hover:-translate-y-0.5 hover:shadow-lg hover:bg-slate-700/30
                        ${selectedItems.includes(result.id)
                          ? 'border-accent/30 bg-accent/5'
                          : 'border-white/5 hover:border-white/20'
                        }
                      `}
                      onClick={() => handleSelectItem(result.id)}
                    >
                      <div className="flex items-start gap-4">
                        <Checkbox
                          checked={selectedItems.includes(result.id)}
                          onClick={(e) => e.stopPropagation()}
                          onChange={() => handleSelectItem(result.id)}
                          className="mt-1"
                        />

                        <div className={`p-2 rounded-lg bg-slate-700/50 ${result.color}`}>
                          {result.icon}
                        </div>

                        <div className="flex-1 min-w-0">
                          <div className="flex items-start justify-between gap-4 mb-2">
                            <div>
                              <h4 className="text-white font-medium">{translateType(result.type)}</h4>
                              <p className="text-sm text-slate-400 mt-1">{translateDescription(result.description)}</p>
                            </div>
                            <div className={`flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-medium border ${getSeverityColor(result.severity)}`}>
                              {getSeverityIcon(result.severity)}
                              {result.severity === 'high' && t('scanPage.severity.high')}
                              {result.severity === 'medium' && t('scanPage.severity.medium')}
                              {result.severity === 'low' && t('scanPage.severity.low')}
                            </div>
                          </div>

                          {result.path && (
                            <div className="flex items-center gap-2 text-xs text-slate-500 font-mono">
                              <span>📁</span>
                              <span className="truncate">{result.path}</span>
                            </div>
                          )}

                          {result.size && (
                            <div className="text-xs text-slate-500 mt-1">
                              {t('scanPage.size')}: {result.size}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 处理中 */}
        {processing && (
          <div className="flex-1 flex items-center justify-center">
            <div className="w-full max-w-2xl space-y-8 animate-slideInUp">
              <div className="text-center">
                <div className="relative inline-block mb-6">
                  <div className="absolute inset-0 bg-accent/20 rounded-full blur-2xl animate-pulse" />
                  <div className="relative p-8 bg-gradient-to-br from-accent/20 to-accent/5 rounded-full border border-accent/30">
                    <Loader2 className="w-16 h-16 text-accent animate-spin" />
                  </div>
                </div>
                <h2 className="text-2xl font-bold text-white mb-2">{t('scanPage.cleaning')}</h2>
                <p className="text-slate-400">{t('scanPage.cleaningDesc', { count: selectedItems.length })}</p>
              </div>

              <div>
                <div className="flex items-center justify-between mb-3">
                  <span className="text-sm text-slate-300">{t('scanPage.cleanProgress')}</span>
                  <span className="text-sm font-mono text-accent">{Math.floor(processProgress)}%</span>
                </div>
                <Progress
                  percent={processProgress}
                  strokeColor={{
                    '0%': '#10b981',
                    '100%': '#059669',
                  }}
                  trailColor="rgba(255,255,255,0.05)"
                  showInfo={false}
                  strokeWidth={12}
                />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* 成功Modal */}
      <Modal
        open={showSuccessModal}
        footer={null}
        closable={false}
        centered
        width={500}
        className="success-modal"
      >
        <div className="text-center py-8">
          <div className="relative inline-block mb-6">
            <div className="absolute inset-0 bg-green-500/20 rounded-full blur-2xl" />
            <div className="relative p-6 bg-gradient-to-br from-green-500/20 to-green-500/5 rounded-full border border-green-500/30">
              <CheckCircle2 className="w-16 h-16 text-green-400" />
            </div>
          </div>

          <h2 className="text-2xl font-bold text-white mb-2">{t('scanPage.cleanComplete')}</h2>
          <p className="text-slate-400 mb-6">
            {t('scanPage.cleanedCount', { count: selectedItems.length })}
          </p>

          <div className="flex gap-3 justify-center">
            <Button
              size="large"
              onClick={() => navigate('/dashboard')}
              className="bg-slate-700 hover:bg-slate-600 text-white border-none"
            >
              {t('scanPage.backToHome')}
            </Button>
            <Button
              type="primary"
              size="large"
              onClick={() => {
                setShowSuccessModal(false);
                setScanning(true);
                setScanProgress(0);
                setScanResults([]);
                setSelectedItems([]);
              }}
              className="bg-accent hover:bg-accent/80 border-none"
            >
              {t('scanPage.rescan')}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default ScanPage;
