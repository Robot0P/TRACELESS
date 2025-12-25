import React, { useState, useEffect, useMemo } from 'react';
import { Button, Progress, Modal, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  FileText,
  Shield,
  Trash2,
  CheckCircle2,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  Server,
  Terminal,
  Lock,
  RefreshCw,
  Settings,
  HardDrive,
  FolderOpen,
  Globe,
  Code,
  Briefcase,
  MessageCircle,
  Music,
  ShieldAlert,
  Download,
  Wrench,
  ChevronDown,
  ChevronRight,
  Cloud,
  Bot,
  Database,
  Monitor,
  Palette,
  StickyNote,
  Wifi,
  Gamepad2,
  Video,
  Headphones,
  Mail,
  ListTodo,
  Image,
  Keyboard,
  BookOpen,
} from 'lucide-react';

interface LogInfo {
  log_type: string;
  label: string;
  description: string;
  size: number;
  size_display: string;
  file_count: number;
  accessible: boolean;
  category: string;
}

interface LogScanResult {
  logs: LogInfo[];
  total_size: number;
  total_files: number;
  needs_permission: boolean;
  permission_guide: string;
}

// 分类配置
const categoryConfig: Record<string, { icon: React.ReactNode; color: string; gradient: string }> = {
  '系统': { icon: <Server size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-cyan-500' },
  '浏览器': { icon: <Globe size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-emerald-500' },
  '开发工具': { icon: <Code size={18} />, color: 'text-purple-400', gradient: 'from-purple-500 to-pink-500' },
  '办公软件': { icon: <Briefcase size={18} />, color: 'text-orange-400', gradient: 'from-orange-500 to-amber-500' },
  '通讯软件': { icon: <MessageCircle size={18} />, color: 'text-cyan-400', gradient: 'from-cyan-500 to-teal-500' },
  '多媒体': { icon: <Music size={18} />, color: 'text-pink-400', gradient: 'from-pink-500 to-rose-500' },
  '安全工具': { icon: <ShieldAlert size={18} />, color: 'text-red-400', gradient: 'from-red-500 to-rose-500' },
  '终端': { icon: <Terminal size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-lime-500' },
  '运维工具': { icon: <Wrench size={18} />, color: 'text-indigo-400', gradient: 'from-indigo-500 to-violet-500' },
  '文件': { icon: <Download size={18} />, color: 'text-yellow-400', gradient: 'from-yellow-500 to-orange-500' },
  '云存储': { icon: <Cloud size={18} />, color: 'text-sky-400', gradient: 'from-sky-500 to-blue-500' },
  'AI工具': { icon: <Bot size={18} />, color: 'text-violet-400', gradient: 'from-violet-500 to-purple-500' },
  '数据库': { icon: <Database size={18} />, color: 'text-emerald-400', gradient: 'from-emerald-500 to-teal-500' },
  '虚拟化': { icon: <Monitor size={18} />, color: 'text-slate-400', gradient: 'from-slate-500 to-gray-500' },
  '设计工具': { icon: <Palette size={18} />, color: 'text-fuchsia-400', gradient: 'from-fuchsia-500 to-pink-500' },
  '笔记工具': { icon: <StickyNote size={18} />, color: 'text-amber-400', gradient: 'from-amber-500 to-yellow-500' },
  '下载工具': { icon: <Download size={18} />, color: 'text-lime-400', gradient: 'from-lime-500 to-green-500' },
  '系统工具': { icon: <Settings size={18} />, color: 'text-gray-400', gradient: 'from-gray-500 to-slate-500' },
  '网络工具': { icon: <Wifi size={18} />, color: 'text-teal-400', gradient: 'from-teal-500 to-cyan-500' },
  '游戏': { icon: <Gamepad2 size={18} />, color: 'text-red-400', gradient: 'from-red-500 to-orange-500' },
  '视频编辑': { icon: <Video size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-indigo-500' },
  '音频编辑': { icon: <Headphones size={18} />, color: 'text-purple-400', gradient: 'from-purple-500 to-violet-500' },
  '邮件': { icon: <Mail size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-cyan-500' },
  '效率工具': { icon: <ListTodo size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-teal-500' },
  '阅读器': { icon: <BookOpen size={18} />, color: 'text-amber-400', gradient: 'from-amber-500 to-orange-500' },
  '输入法': { icon: <Keyboard size={18} />, color: 'text-gray-400', gradient: 'from-gray-500 to-slate-500' },
  '图片': { icon: <Image size={18} />, color: 'text-pink-400', gradient: 'from-pink-500 to-rose-500' },

  // English keys support
  'System': { icon: <Server size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-cyan-500' },
  'Browser': { icon: <Globe size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-emerald-500' },
  'Dev Tools': { icon: <Code size={18} />, color: 'text-purple-400', gradient: 'from-purple-500 to-pink-500' },
  'Office': { icon: <Briefcase size={18} />, color: 'text-orange-400', gradient: 'from-orange-500 to-amber-500' },
  'Messaging': { icon: <MessageCircle size={18} />, color: 'text-cyan-400', gradient: 'from-cyan-500 to-teal-500' },
  'Multimedia': { icon: <Music size={18} />, color: 'text-pink-400', gradient: 'from-pink-500 to-rose-500' },
  'Security Tools': { icon: <ShieldAlert size={18} />, color: 'text-red-400', gradient: 'from-red-500 to-rose-500' },
  'Terminal': { icon: <Terminal size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-lime-500' },
  'DevOps Tools': { icon: <Wrench size={18} />, color: 'text-indigo-400', gradient: 'from-indigo-500 to-violet-500' },
  'Files': { icon: <Download size={18} />, color: 'text-yellow-400', gradient: 'from-yellow-500 to-orange-500' },
  'Cloud Storage': { icon: <Cloud size={18} />, color: 'text-sky-400', gradient: 'from-sky-500 to-blue-500' },
  'AI Tools': { icon: <Bot size={18} />, color: 'text-violet-400', gradient: 'from-violet-500 to-purple-500' },
  'Database': { icon: <Database size={18} />, color: 'text-emerald-400', gradient: 'from-emerald-500 to-teal-500' },
  'Virtualization': { icon: <Monitor size={18} />, color: 'text-slate-400', gradient: 'from-slate-500 to-gray-500' },
  'Design Tools': { icon: <Palette size={18} />, color: 'text-fuchsia-400', gradient: 'from-fuchsia-500 to-pink-500' },
  'Note Tools': { icon: <StickyNote size={18} />, color: 'text-amber-400', gradient: 'from-amber-500 to-yellow-500' },
  'Download Tools': { icon: <Download size={18} />, color: 'text-lime-400', gradient: 'from-lime-500 to-green-500' },
  'System Tools': { icon: <Settings size={18} />, color: 'text-gray-400', gradient: 'from-gray-500 to-slate-500' },
  'Network Tools': { icon: <Wifi size={18} />, color: 'text-teal-400', gradient: 'from-teal-500 to-cyan-500' },
  'Games': { icon: <Gamepad2 size={18} />, color: 'text-red-400', gradient: 'from-red-500 to-orange-500' },
  'Video Editing': { icon: <Video size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-indigo-500' },
  'Audio Editing': { icon: <Headphones size={18} />, color: 'text-purple-400', gradient: 'from-purple-500 to-violet-500' },
  'Email': { icon: <Mail size={18} />, color: 'text-blue-400', gradient: 'from-blue-500 to-cyan-500' },
  'Productivity': { icon: <ListTodo size={18} />, color: 'text-green-400', gradient: 'from-green-500 to-teal-500' },
  'Reader': { icon: <BookOpen size={18} />, color: 'text-amber-400', gradient: 'from-amber-500 to-orange-500' },
  'Input Method': { icon: <Keyboard size={18} />, color: 'text-gray-400', gradient: 'from-gray-500 to-slate-500' },
  'Images': { icon: <Image size={18} />, color: 'text-pink-400', gradient: 'from-pink-500 to-rose-500' },
  'Other': { icon: <FolderOpen size={18} />, color: 'text-slate-400', gradient: 'from-slate-500 to-slate-600' },
};

const SystemLogs: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedLogs, setSelectedLogs] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [scanning, setScanning] = useState(true);
  const [progress, setProgress] = useState(0);
  const [currentLog, setCurrentLog] = useState('');
  const [showSuccess, setShowSuccess] = useState(false);
  const [platform, setPlatform] = useState<string>('');
  const [scanResult, setScanResult] = useState<LogScanResult | null>(null);
  const [expandedCategories, setExpandedCategories] = useState<string[]>([]);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  // 翻译分类名称的函数
  const translateCategory = (category: string): string => {
    return t(`systemLogs.categories.${category}`, category);
  };

  // 翻译日志项标签
  const translateLogLabel = (logType: string, defaultLabel: string): string => {
    // 首先尝试通过 logType 获取翻译
    const translatedByType = t(`systemLogs.logItems.${logType}.label`, { defaultValue: '' });
    if (translatedByType && translatedByType !== `systemLogs.logItems.${logType}.label`) {
      return translatedByType;
    }

    // 如果没有找到，使用标签映射表
    const labelPatterns: Record<string, string> = {
      '提醒事项': t('systemLogs.labelPatterns.reminders'),
      '日历': t('systemLogs.labelPatterns.calendar'),
      '钥匙串': t('systemLogs.labelPatterns.keychain'),
      '系统邮件缓存、附件': t('systemLogs.labelPatterns.appleMail'),
    };

    return labelPatterns[defaultLabel] || defaultLabel;
  };

  // 翻译日志项描述
  const translateLogDescription = (logType: string, defaultDesc: string): string => {
    // 首先尝试通过 logType 获取翻译
    const translatedByType = t(`systemLogs.logItems.${logType}.description`, { defaultValue: '' });
    if (translatedByType && translatedByType !== `systemLogs.logItems.${logType}.description`) {
      return translatedByType;
    }

    // 如果没有找到，使用描述模式匹配
    const descriptionPatterns: Record<string, string> = {
      '系统和应用程序运行日志': t('systemLogs.descPatterns.systemAppLogs'),
      '所有应用程序的缓存文件': t('systemLogs.descPatterns.allAppCache'),
      '应用崩溃报告和诊断数据': t('systemLogs.descPatterns.crashDiagnostic'),
      '最近打开的文件和应用记录': t('systemLogs.descPatterns.recentFilesApps'),
      '软件安装和更新记录': t('systemLogs.descPatterns.installUpdate'),
      '浏览历史、缓存、Cookie': t('systemLogs.descPatterns.browserData'),
      '浏览历史、缓存、Cookie、密码': t('systemLogs.descPatterns.browserDataWithPassword'),
      '编辑器缓存、日志、工作区记录': t('systemLogs.descPatterns.editorCacheWorkspace'),
      '构建缓存、模拟器数据、日志': t('systemLogs.descPatterns.buildSimulator'),
      'IntelliJ/PyCharm/WebStorm 等 IDE 数据': t('systemLogs.descPatterns.jetbrainsIde'),
      'Git 配置和凭据': t('systemLogs.descPatterns.gitConfig'),
      'NPM 缓存和历史': t('systemLogs.descPatterns.npmCache'),
      'Python/pip/Jupyter 缓存和历史': t('systemLogs.descPatterns.pythonCache'),
      'Docker 配置、镜像缓存': t('systemLogs.descPatterns.dockerCache'),
      'Word/Excel/PowerPoint 缓存和最近文档': t('systemLogs.descPatterns.officeCache'),
      '聊天缓存、图片、视频': t('systemLogs.descPatterns.chatMediaCache'),
      '聊天缓存、媒体文件': t('systemLogs.descPatterns.chatMedia'),
      '音乐缓存、播放历史': t('systemLogs.descPatterns.musicCache'),
      '播放历史、配置': t('systemLogs.descPatterns.playbackConfig'),
      '项目文件、配置、历史': t('systemLogs.descPatterns.projectConfig'),
      '抓包文件、配置': t('systemLogs.descPatterns.packetCapture'),
      '数据库、日志、模块缓存': t('systemLogs.descPatterns.databaseCache'),
      'Bash/Zsh/Fish 命令历史': t('systemLogs.descPatterns.shellHistory'),
      '已知主机记录': t('systemLogs.descPatterns.knownHosts'),
      '终端配置、日志': t('systemLogs.descPatterns.terminalConfig'),
      '软件包缓存、安装日志': t('systemLogs.descPatterns.packageCache'),
      '集群配置、缓存': t('systemLogs.descPatterns.clusterConfig'),
      '配置、凭据、缓存': t('systemLogs.descPatterns.configCredCache'),
      '已删除的文件': t('systemLogs.descPatterns.deletedFiles'),
      '下载的文件': t('systemLogs.descPatterns.downloadedFiles'),
      'Android 开发工具、SDK、模拟器': t('systemLogs.descPatterns.androidSdk'),
      'Rust 工具链、包缓存': t('systemLogs.descPatterns.rustToolchain'),
      'Go 模块缓存、构建缓存': t('systemLogs.descPatterns.goCache'),
      'Ruby 包管理器缓存': t('systemLogs.descPatterns.rubyCache'),
      'Java 构建工具缓存': t('systemLogs.descPatterns.javaCache'),
      '编辑器缓存、会话数据': t('systemLogs.descPatterns.editorSession'),
      'AI 编辑器缓存、配置': t('systemLogs.descPatterns.aiEditorCache'),
      'JavaScript 包管理器缓存': t('systemLogs.descPatterns.jsPackageCache'),
      'iOS/macOS 依赖管理缓存': t('systemLogs.descPatterns.cocoapodsCache'),
      '聊天缓存、文件、日志': t('systemLogs.descPatterns.chatFileLogs'),
      '会议缓存、录制、日志': t('systemLogs.descPatterns.meetingCache'),
      '聊天缓存、会议数据': t('systemLogs.descPatterns.chatMeetingData'),
      '聊天缓存、通话记录': t('systemLogs.descPatterns.chatCallHistory'),
      '视频缓存、观看历史': t('systemLogs.descPatterns.videoCache'),
      '云同步缓存': t('systemLogs.descPatterns.cloudSyncCache'),
      '同步缓存、配置': t('systemLogs.descPatterns.syncConfig'),
      '同步缓存、日志': t('systemLogs.descPatterns.syncLogs'),
      '下载缓存、配置': t('systemLogs.descPatterns.downloadConfig'),
      '对话缓存、配置': t('systemLogs.descPatterns.conversationCache'),
      'AI 代码助手缓存': t('systemLogs.descPatterns.aiCodeAssistant'),
      '数据库连接、查询历史': t('systemLogs.descPatterns.dbQueryHistory'),
      'MySQL 客户端数据': t('systemLogs.descPatterns.mysqlClient'),
      'MongoDB 客户端数据': t('systemLogs.descPatterns.mongoClient'),
      'Redis 客户端数据': t('systemLogs.descPatterns.redisClient'),
      'HTTP 代理、抓包数据': t('systemLogs.descPatterns.httpProxy'),
      'API 测试、历史记录': t('systemLogs.descPatterns.apiTestHistory'),
      'API 测试、请求历史': t('systemLogs.descPatterns.apiRequestHistory'),
      '虚拟机配置、日志': t('systemLogs.descPatterns.vmConfigLogs'),
      '设计缓存、本地文件': t('systemLogs.descPatterns.designCacheLocal'),
      '设计缓存、插件': t('systemLogs.descPatterns.designCachePlugins'),
      'PS/AI/PR 缓存和配置': t('systemLogs.descPatterns.adobeCache'),
      '笔记缓存、离线数据': t('systemLogs.descPatterns.noteCacheOffline'),
      '笔记缓存、插件数据': t('systemLogs.descPatterns.noteCachePlugins'),
      '笔记数据、附件': t('systemLogs.descPatterns.noteAttachments'),
      '下载缓存、任务记录': t('systemLogs.descPatterns.downloadTaskCache'),
      '清理工具缓存、日志': t('systemLogs.descPatterns.cleanerCache'),
      '搜索缓存、工作流': t('systemLogs.descPatterns.searchWorkflow'),
      '搜索缓存、扩展数据': t('systemLogs.descPatterns.searchExtensions'),
      '密码管理器缓存': t('systemLogs.descPatterns.passwordManager'),
      '系统密码存储': t('systemLogs.descPatterns.systemKeychain'),
      'Surge/ClashX/V2Ray 配置': t('systemLogs.descPatterns.proxyConfig'),
      '基础设施即代码工具缓存': t('systemLogs.descPatterns.iacCache'),
      // 新增的描述模式
      '系统邮件缓存、附件': t('systemLogs.descPatterns.mailCacheAttachments'),
      'Apple 提醒事项缓存': t('systemLogs.descPatterns.appleReminders'),
      'Apple 日历缓存': t('systemLogs.descPatterns.appleCalendar'),
      '日历软件缓存': t('systemLogs.descPatterns.calendarCache'),
    };

    return descriptionPatterns[defaultDesc] || defaultDesc;
  };

  // 按分类分组日志
  const groupedLogs = useMemo(() => {
    if (!scanResult) return {};
    const groups: Record<string, LogInfo[]> = {};
    for (const log of scanResult.logs) {
      const category = log.category || '其他';
      if (!groups[category]) {
        groups[category] = [];
      }
      groups[category].push(log);
    }
    return groups;
  }, [scanResult]);

  // 分类排序
  const sortedCategories = useMemo(() => {
    const order = [
      '系统', '浏览器', '开发工具', '办公软件', '通讯软件', '多媒体',
      '游戏', '视频编辑', '音频编辑', '邮件', '效率工具',
      '安全工具', '终端', '运维工具', '云存储', 'AI工具', '数据库',
      '虚拟化', '设计工具', '笔记工具', '下载工具', '系统工具', '网络工具',
      '阅读器', '输入法', '图片', '文件'
    ];
    return Object.keys(groupedLogs).sort((a, b) => {
      const indexA = order.indexOf(a);
      const indexB = order.indexOf(b);
      if (indexA === -1 && indexB === -1) return a.localeCompare(b);
      if (indexA === -1) return 1;
      if (indexB === -1) return -1;
      return indexA - indexB;
    });
  }, [groupedLogs]);

  const scanLogs = async () => {
    setScanning(true);
    try {
      const os = await invoke<string>('get_platform');
      setPlatform(os);

      const result = await invoke<LogScanResult>('scan_system_logs');
      setScanResult(result);

      // 默认选中所有可访问的日志
      const accessibleLogs = result.logs
        .filter(l => l.accessible && l.file_count > 0)
        .map(l => l.log_type);
      setSelectedLogs(accessibleLogs);

      // 默认展开所有分类
      const categories = [...new Set(result.logs.map(l => l.category))];
      setExpandedCategories(categories);
    } catch {
      // Silently fail
    } finally {
      setScanning(false);
    }
  };

  useEffect(() => {
    scanLogs();
  }, []);

  const handleToggleLog = (value: string) => {
    setSelectedLogs(prev =>
      prev.includes(value)
        ? prev.filter(v => v !== value)
        : [...prev, value]
    );
  };

  const handleToggleCategory = (category: string) => {
    const categoryLogs = groupedLogs[category]?.filter(l => l.accessible).map(l => l.log_type) || [];
    const allSelected = categoryLogs.every(l => selectedLogs.includes(l));

    if (allSelected) {
      setSelectedLogs(prev => prev.filter(l => !categoryLogs.includes(l)));
    } else {
      setSelectedLogs(prev => [...new Set([...prev, ...categoryLogs])]);
    }
  };

  const handleSelectAll = () => {
    if (!scanResult) return;
    const accessibleLogs = scanResult.logs
      .filter(l => l.accessible)
      .map(l => l.log_type);

    if (selectedLogs.length === accessibleLogs.length) {
      setSelectedLogs([]);
    } else {
      setSelectedLogs(accessibleLogs);
    }
  };

  const toggleCategoryExpand = (category: string) => {
    setExpandedCategories(prev =>
      prev.includes(category)
        ? prev.filter(c => c !== category)
        : [...prev, category]
    );
  };

  const handleClearLogs = async () => {
    if (selectedLogs.length === 0) {
      Modal.warning({
        title: t('common.warning'),
        content: t('systemLogs.errors.selectLogs'),
        centered: true,
      });
      return;
    }

    setLoading(true);
    setProgress(0);

    try {
      for (let i = 0; i < selectedLogs.length; i++) {
        const logType = selectedLogs[i];
        const logData = scanResult?.logs.find(l => l.log_type === logType);

        setCurrentLog(logData?.label || logType);

        const itemProgress = ((i + 0.5) / selectedLogs.length) * 100;
        setProgress(Math.min(itemProgress, 100));

        await invoke('clear_system_logs', {
          logTypes: [logType],
        });

        setProgress(((i + 1) / selectedLogs.length) * 100);
      }

      setLoading(false);
      setShowSuccess(true);

      await scanLogs();
    } catch (error) {
      setLoading(false);
      Modal.error({
        title: t('systemLogs.errors.clearFailed'),
        content: String(error),
        centered: true,
      });
    }
  };

  const formatTotalSize = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024) {
      return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    } else if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    } else if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${bytes} B`;
  };

  const selectedSize = scanResult?.logs
    .filter(log => selectedLogs.includes(log.log_type))
    .reduce((sum, log) => sum + log.size, 0) || 0;

  const selectedFileCount = scanResult?.logs
    .filter(log => selectedLogs.includes(log.log_type))
    .reduce((sum, log) => sum + log.file_count, 0) || 0;

  // 计算分类统计
  const getCategoryStats = (category: string) => {
    const logs = groupedLogs[category] || [];
    const selectedInCategory = logs.filter(l => selectedLogs.includes(l.log_type));
    const accessibleInCategory = logs.filter(l => l.accessible);
    const size = selectedInCategory.reduce((sum, l) => sum + l.size, 0);
    return {
      selected: selectedInCategory.length,
      total: accessibleInCategory.length,
      size,
    };
  };

  return (
    <ProFeatureGate
      feature="system_logs"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
    <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-purple-500/5 rounded-full blur-3xl animate-pulse" />
      </div>

      <div className="flex-1 flex flex-col p-6 relative z-10 overflow-hidden min-h-0">
        {/* 顶部导航 */}
        <div className="flex items-center justify-between mb-4 animate-slideInDown flex-shrink-0">
          <div className="flex items-center gap-4">
            <button
              onClick={() => navigate('/dashboard')}
              className="p-2 rounded-lg bg-slate-800/50 border border-white/5 hover:border-accent/30 text-slate-400 hover:text-white transition-all"
            >
              <ArrowLeft size={20} />
            </button>
            <div>
              <h1 className="text-2xl font-bold text-white flex items-center gap-3">
                <FileText className="w-7 h-7 text-purple-400" />
                {t('systemLogs.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('systemLogs.subtitle')}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 px-4 py-2 bg-purple-500/10 border border-purple-500/20 rounded-lg">
              <Server className="w-4 h-4 text-purple-400" />
              <span className="text-sm text-purple-300">
                {platform === 'windows' ? 'Windows' : platform === 'macos' ? 'macOS' : 'Linux'}
              </span>
            </div>
          </div>
        </div>

        {scanning ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Loader2 className="w-12 h-12 text-purple-400 animate-spin mx-auto mb-4" />
              <p className="text-slate-400">{t('systemLogs.scanning')}</p>
            </div>
          </div>
        ) : (
          <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden min-h-0">
            {/* 左侧：日志类型选择 */}
            <div className="lg:col-span-2 flex flex-col space-y-4 min-h-0 overflow-hidden">
              {/* 日志列表头部 */}
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 p-4 shadow-2xl animate-slideInLeft flex-shrink-0">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Shield className="w-5 h-5 text-accent" />
                    <h3 className="text-lg font-bold text-white">{t('systemLogs.logTypes')}</h3>
                    <span className="text-sm text-slate-400">
                      ({selectedLogs.length}/{scanResult?.logs.filter(l => l.accessible).length || 0})
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      onClick={scanLogs}
                      size="small"
                      icon={<RefreshCw size={14} />}
                      className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
                    >
                      {t('systemLogs.refresh')}
                    </Button>
                    <Button
                      onClick={handleSelectAll}
                      size="small"
                      className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
                    >
                      {selectedLogs.length === scanResult?.logs.filter(l => l.accessible).length ? t('common.clearAll') : t('common.selectAll')}
                    </Button>
                  </div>
                </div>
              </div>

              {/* 分类列表 - 可滚动区域 */}
              <div className="flex-1 overflow-y-auto custom-scrollbar space-y-3 pr-2">
                {sortedCategories.map((category) => {
                  const config = categoryConfig[category] || { icon: <FolderOpen size={18} />, color: 'text-slate-400', gradient: 'from-slate-500 to-slate-600' };
                  const stats = getCategoryStats(category);
                  const isExpanded = expandedCategories.includes(category);
                  const logs = groupedLogs[category] || [];

                  return (
                    <div
                      key={category}
                      className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 overflow-hidden"
                    >
                      {/* 分类标题 */}
                      <div
                        className="flex items-center justify-between p-3 cursor-pointer hover:bg-white/5 transition-colors"
                        onClick={() => toggleCategoryExpand(category)}
                      >
                        <div className="flex items-center gap-3">
                          {isExpanded ? <ChevronDown size={16} className="text-slate-400" /> : <ChevronRight size={16} className="text-slate-400" />}
                          <div className={config.color}>{config.icon}</div>
                          <span className="font-bold text-white">{translateCategory(category)}</span>
                          <span className="text-xs text-slate-500">
                            ({stats.selected}/{stats.total})
                          </span>
                          {stats.size > 0 && (
                            <span className="text-xs text-accent">
                              {formatTotalSize(stats.size)}
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
                          <Button
                            size="small"
                            onClick={() => handleToggleCategory(category)}
                            className="text-xs bg-slate-700/50 border-slate-600 text-white hover:bg-slate-600"
                          >
                            {stats.selected === stats.total && stats.total > 0 ? t('systemLogs.deselect') : t('common.selectAll')}
                          </Button>
                        </div>
                      </div>

                      {/* 分类内容 */}
                      {isExpanded && (
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 p-3 pt-0">
                          {logs.map((log) => {
                            const isSelected = selectedLogs.includes(log.log_type);

                            return (
                              <button
                                key={log.log_type}
                                onClick={() => log.accessible && handleToggleLog(log.log_type)}
                                disabled={!log.accessible}
                                className={`
                                  relative p-3 rounded-lg border transition-all duration-200 text-left overflow-hidden
                                  ${!log.accessible
                                    ? 'border-white/5 bg-slate-800/20 opacity-50 cursor-not-allowed'
                                    : isSelected
                                      ? 'border-accent/50 bg-accent/10'
                                      : 'border-white/10 hover:border-accent/30 bg-slate-800/30 hover:bg-slate-800/50'
                                  }
                                `}
                              >
                                <div className="flex items-center justify-between mb-1">
                                  <div className="text-sm text-white font-medium truncate pr-2">{translateLogLabel(log.log_type, log.label)}</div>
                                  {log.accessible ? (
                                    <div onClick={(e) => e.stopPropagation()}>
                                      <Switch
                                        checked={isSelected}
                                        size="small"
                                        onChange={() => handleToggleLog(log.log_type)}
                                      />
                                    </div>
                                  ) : (
                                    <Lock size={14} className="text-slate-500 flex-shrink-0" />
                                  )}
                                </div>
                                <div className="text-xs text-slate-500 mb-1 truncate">{translateLogDescription(log.log_type, log.description)}</div>
                                <div className="flex items-center justify-between text-xs">
                                  <div className="flex items-center gap-1">
                                    <HardDrive size={10} className="text-slate-500" />
                                    <span className={log.accessible ? 'text-accent' : 'text-slate-500'}>
                                      {log.size_display}
                                    </span>
                                  </div>
                                  <span className="text-slate-500">
                                    {log.file_count} {t('systemLogs.files')}
                                  </span>
                                </div>
                              </button>
                            );
                          })}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>

            {/* 右侧：控制面板 */}
            <div className="space-y-4">
              {/* 统计信息 */}
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 p-4 shadow-2xl animate-slideInRight">
                <h3 className="text-lg font-bold text-white mb-3">{t('systemLogs.cleanupStats')}</h3>

                <div className="space-y-3">
                  <div className="p-3 bg-purple-500/10 rounded-lg border border-purple-500/20">
                    <div className="text-xs text-slate-400 mb-1">{t('systemLogs.estimatedSpace')}</div>
                    <div className="text-2xl font-bold text-purple-400">
                      {formatTotalSize(selectedSize)}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-slate-400">{t('systemLogs.selectedItems')}</span>
                      <span className="text-white font-medium">{selectedLogs.length} {t('systemLogs.items')}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-slate-400">{t('systemLogs.fileCount')}</span>
                      <span className="text-white font-medium">{selectedFileCount.toLocaleString()} {t('systemLogs.files')}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-slate-400">{t('systemLogs.totalScanned')}</span>
                      <span className="text-accent font-medium">
                        {formatTotalSize(scanResult?.total_size || 0)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              {/* 操作按钮 */}
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 p-4 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.1s' }}>
                <Button
                  type="primary"
                  size="large"
                  block
                  onClick={handleClearLogs}
                  disabled={selectedLogs.length === 0 || loading}
                  className="h-12 bg-gradient-to-r from-purple-600 to-purple-500 border-none hover:from-purple-500 hover:to-purple-400 text-white font-bold"
                  icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Trash2 size={20} />}
                >
                  {loading ? t('systemLogs.clearing') : t('systemLogs.startClear')}
                </Button>

                {loading && (
                  <div className="mt-3 space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-slate-300">{t('systemLogs.cleanupProgress')}</span>
                      <span className="text-accent font-mono">{Math.floor(progress)}%</span>
                    </div>
                    <Progress
                      percent={progress}
                      strokeColor={{
                        '0%': '#a855f7',
                        '100%': '#8b5cf6',
                      }}
                      trailColor="rgba(255,255,255,0.05)"
                      showInfo={false}
                      strokeWidth={8}
                    />
                    <div className="text-xs text-slate-400 truncate">
                      {currentLog}
                    </div>
                  </div>
                )}

                {scanResult?.needs_permission && (
                  <div className="mt-3 p-2 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                    <div className="flex items-start gap-2">
                      <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
                      <div className="text-xs text-amber-300">
                        {t('systemLogs.warnings.adminRequired')}
                      </div>
                    </div>
                  </div>
                )}
              </div>

              {/* 分类快速选择 */}
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 p-4 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.2s' }}>
                <h3 className="text-sm font-bold text-white mb-3">{t('systemLogs.quickSelect')}</h3>
                <div className="grid grid-cols-2 gap-2">
                  {sortedCategories.slice(0, 6).map((category) => {
                    const config = categoryConfig[category] || { icon: <FolderOpen size={14} />, color: 'text-slate-400', gradient: 'from-slate-500 to-slate-600' };
                    const stats = getCategoryStats(category);
                    const allSelected = stats.selected === stats.total && stats.total > 0;

                    return (
                      <button
                        key={category}
                        onClick={() => handleToggleCategory(category)}
                        className={`
                          flex items-center gap-2 p-2 rounded-lg text-xs transition-all
                          ${allSelected
                            ? 'bg-accent/20 border border-accent/30 text-accent'
                            : 'bg-slate-800/30 border border-white/5 text-slate-400 hover:bg-slate-800/50'
                          }
                        `}
                      >
                        <span className={config.color}>{config.icon}</span>
                        <span className="truncate">{translateCategory(category)}</span>
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        )}
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

          <h2 className="text-2xl font-bold text-white mb-2">{t('systemLogs.clearComplete')}</h2>
          <p className="text-slate-400 mb-2">
            {t('systemLogs.clearCompleteDesc1')} <span className="text-accent font-bold">{selectedLogs.length}</span> {t('systemLogs.clearCompleteDesc2')}
          </p>
          <p className="text-green-400 text-sm mb-6">
            {t('systemLogs.spaceFreed')}: {formatTotalSize(selectedSize)}
          </p>

          <Button
            type="primary"
            size="large"
            onClick={() => {
              setShowSuccess(false);
              setSelectedLogs([]);
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

export default SystemLogs;
