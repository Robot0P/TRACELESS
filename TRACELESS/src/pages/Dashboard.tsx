import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
  Activity,
  Cpu,
  Scan,
  AlertTriangle,
  Zap,
  Fingerprint,
  Wifi,
  Monitor,
  MemoryStick,
  CheckCircle2,
  Home,
  XCircle,
} from 'lucide-react';

interface SystemInfo {
  os: string;
  version: string;
  totalMemory: number;
  usedMemory: number;
  cpuUsage: number;
}

interface HistoryData {
  cpu: number[];
  memory: number[];
  download: number[];
  upload: number[];
}

// 威胁检测结果接口
interface ThreatDetectionResult {
  vm_detected: boolean;
  debugger_detected: boolean;
  sandbox_detected: boolean;
  forensic_tools_detected: boolean;
  details: Array<{
    name: string;
    detected: boolean;
    details?: string;
    category: string;
    confidence: string;
  }>;
}

// 格式化网络速度，自动选择合适的单位
const formatNetworkSpeed = (speedInMB: number): { value: string; unit: string; fullText: string } => {
  const bytes = speedInMB * 1024 * 1024; // 转换为字节

  if (bytes < 1024) {
    return {
      value: bytes.toFixed(0),
      unit: 'B/s',
      fullText: `${bytes.toFixed(0)} B/s`
    };
  } else if (bytes < 1024 * 1024) {
    const kb = bytes / 1024;
    return {
      value: kb.toFixed(1),
      unit: 'KB/s',
      fullText: `${kb.toFixed(1)} KB/s`
    };
  } else if (bytes < 1024 * 1024 * 1024) {
    const mb = bytes / (1024 * 1024);
    return {
      value: mb.toFixed(2),
      unit: 'MB/s',
      fullText: `${mb.toFixed(2)} MB/s`
    };
  } else {
    const gb = bytes / (1024 * 1024 * 1024);
    return {
      value: gb.toFixed(2),
      unit: 'GB/s',
      fullText: `${gb.toFixed(2)} GB/s`
    };
  }
};


interface RadarBlip {
  id: number;
  x: number;
  y: number;
  size: number;
  color: string;
}

const Dashboard: React.FC = () => {
  const navigate = useNavigate();
  const { t } = useTranslation();

  // 系统信息
  const [systemInfo, setSystemInfo] = useState<SystemInfo>({
    os: 'Unknown',
    version: '1.0.0',
    totalMemory: 16384,
    usedMemory: 8192,
    cpuUsage: 0,
  });
  const [networkSpeed, setNetworkSpeed] = useState({ download: 0, upload: 0 });

  // 历史数据用于图表（保留最近30个数据点）
  const [history, setHistory] = useState<HistoryData>({
    cpu: Array(30).fill(0),
    memory: Array(30).fill(0),
    download: Array(30).fill(0),
    upload: Array(30).fill(0),
  });

  // 背景粒子动画
  const [particles, setParticles] = useState<Array<{ x: number; y: number; size: number; delay: number }>>([]);

  // 雷达光点
  const [radarBlips, setRadarBlips] = useState<RadarBlip[]>([]);

  // 威胁检测状态
  const [threatStatus, setThreatStatus] = useState<{
    loading: boolean;
    threatCount: number;
    isSecure: boolean;
    details: ThreatDetectionResult | null;
  }>({
    loading: true,
    threatCount: 0,
    isSecure: true,
    details: null,
  });

  useEffect(() => {
    // 生成随机粒子
    const newParticles = Array.from({ length: 30 }, () => ({
      x: Math.random() * 100,
      y: Math.random() * 100,
      size: Math.random() * 3 + 1,
      delay: Math.random() * 5,
    }));
    setParticles(newParticles);

    // 获取系统信息
    const fetchSystemInfo = async () => {
      try {
        const info = await invoke<any>('get_system_info_api');
        setSystemInfo({
          os: info.os,
          version: info.version,
          totalMemory: info.total_memory,
          usedMemory: info.used_memory,
          cpuUsage: info.cpu_usage,
        });
      } catch {
        // Silently fail
      }
    };
    fetchSystemInfo();

    // 获取威胁检测状态
    const fetchThreatStatus = async () => {
      try {
        const result = await invoke<ThreatDetectionResult>('check_environment');
        const threatCount = [
          result.vm_detected,
          result.debugger_detected,
          result.sandbox_detected,
          result.forensic_tools_detected,
        ].filter(Boolean).length;

        setThreatStatus({
          loading: false,
          threatCount,
          isSecure: threatCount === 0,
          details: result,
        });
      } catch {
        setThreatStatus(prev => ({ ...prev, loading: false }));
      }
    };
    fetchThreatStatus();

    // 真实系统资源监控（实时更新）
    const interval = setInterval(async () => {
      try {
        // 获取系统信息
        const info = await invoke<any>('get_system_info_api');
        // 获取网络速度
        const netSpeed = await invoke<any>('get_network_speed_api');

        setSystemInfo(prev => ({
          ...prev,
          cpuUsage: info.cpu_usage,
          usedMemory: info.used_memory,
        }));

        setNetworkSpeed({
          download: netSpeed.download,
          upload: netSpeed.upload,
        });

        // 更新历史数据
        setHistory(prev => ({
          cpu: [...prev.cpu.slice(1), info.cpu_usage],
          memory: [...prev.memory.slice(1), (info.used_memory / info.total_memory) * 100],
          download: [...prev.download.slice(1), netSpeed.download],
          upload: [...prev.upload.slice(1), netSpeed.upload],
        }));
      } catch {
        // Silently fail
      }
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  // 动态生成雷达光点 - 根据威胁状态调整颜色
  useEffect(() => {
    const addBlip = () => {
      const angle = Math.random() * Math.PI * 2;
      const distance = Math.random() * 80 + 10; // 10-90 px radius
      const x = 100 + Math.cos(angle) * distance;
      const y = 100 + Math.sin(angle) * distance;

      // 根据威胁状态选择颜色
      let color: string;
      if (threatStatus.loading) {
        color = '#3b82f6'; // 蓝色 - 加载中
      } else if (threatStatus.isSecure) {
        color = Math.random() > 0.7 ? '#14b8a6' : '#10b981'; // 绿色/青色 - 安全
      } else {
        color = Math.random() > 0.5 ? '#ef4444' : '#f97316'; // 红色/橙色 - 有威胁
      }

      const newBlip: RadarBlip = {
        id: Date.now(),
        x,
        y,
        size: threatStatus.isSecure ? Math.random() * 2 + 2 : Math.random() * 3 + 3, // 威胁时光点更大
        color,
      };

      setRadarBlips(prev => [...prev, newBlip]);

      // Remove blip after animation
      setTimeout(() => {
        setRadarBlips(prev => prev.filter(b => b.id !== newBlip.id));
      }, 2000);
    };

    const interval = setInterval(addBlip, threatStatus.isSecure ? 800 : 400); // 威胁时更新更快
    return () => clearInterval(interval);
  }, [threatStatus.loading, threatStatus.isSecure]);

  const handleSmartScan = () => {
    // 跳转到扫描页面，传递扫描模式
    navigate('/scan', { state: { mode: 'smart' } });
  };

  const handleFullScan = () => {
    // 跳转到扫描页面，传递扫描模式
    navigate('/scan', { state: { mode: 'full' } });
  };

  // 绘制迷你实时图表
  const renderMiniChart = (data: number[], color: string, max?: number) => {
    // 如果没有指定max，使用动态最大值（至少为10，以保证图表有合理的刻度）
    const maxValue = max !== undefined ? max : Math.max(...data, 10);
    const points = data.map((value, index) => {
      const x = (index / (data.length - 1)) * 100;
      const y = 100 - (value / maxValue) * 80; // 使用80%高度，留出顶部空间
      return `${x},${y} `;
    }).join(' ');

    return (
      <svg className="w-full h-16 mt-2" viewBox="0 0 100 100" preserveAspectRatio="none">
        {/* 网格线 */}
        <line x1="0" y1="25" x2="100" y2="25" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" />
        <line x1="0" y1="50" x2="100" y2="50" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" />
        <line x1="0" y1="75" x2="100" y2="75" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5" />

        {/* 渐变定义 */}
        <defs>
          <linearGradient id={`gradient-${color}`} x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor={color} stopOpacity="0.3" />
            <stop offset="100%" stopColor={color} stopOpacity="0.05" />
          </linearGradient>
        </defs>

        {/* 填充区域 */}
        <polygon
          points={`0,100 ${points} 100,100`}
          fill={`url(#gradient-${color})`}
        />

        {/* 线条 */}
        <polyline
          points={points}
          fill="none"
          stroke={color}
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    );
  };

  return (
    <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 动态背景粒子 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        {particles.map((particle, idx) => (
          <div
            key={idx}
            className="absolute rounded-full bg-accent/20 animate-float"
            style={{
              left: `${particle.x}%`,
              top: `${particle.y}%`,
              width: `${particle.size}px`,
              height: `${particle.size}px`,
              animationDelay: `${particle.delay}s`,
              animationDuration: `${15 + particle.delay * 2}s`,
            }}
          />
        ))}

        {/* 网格背景 */}
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px] [mask-image:radial-gradient(ellipse_at_center,black_20%,transparent_80%)]" />

        {/* 光晕效果 */}
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-accent/5 rounded-full blur-3xl animate-pulse" />
        <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-blue-500/5 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }} />
      </div>

      <div className="flex-1 flex flex-col p-6 pt-10 relative z-10 w-full">
        {/* 顶部标题栏 */}
        <div className="flex items-center justify-between mb-6">
          {/* 左侧：页面标题 */}
          <div className="flex items-center gap-3">
            <div className="p-2 bg-accent/10 rounded-lg">
              <Home className="w-6 h-6 text-accent" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">{t('dashboard.systemOverview')}</h1>
              <p className="text-sm text-slate-400">{t('dashboard.systemOverviewDesc')}</p>
            </div>
          </div>
        </div>

        {/* 主扫描区域 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-3 mb-3">
          {/* 扫描控制卡片 */}
          <div className="lg:col-span-2 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-4 shadow-2xl animate-slideInLeft">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-lg font-bold text-white mb-1">{t('dashboard.scanEngine')}</h2>
                <p className="text-xs text-slate-400">{t('dashboard.scanEngineDesc')}</p>
              </div>
              <Fingerprint className="w-8 h-8 text-accent/50" />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <button
                onClick={handleSmartScan}
                className="group relative bg-gradient-to-br from-accent/20 to-orange-500/20 hover:from-accent/30 hover:to-orange-500/30 border border-accent/30 rounded-xl p-3 transition-all duration-300 hover:scale-105 hover:shadow-[0_0_30px_rgba(217,148,63,0.3)]"
              >
                <div className="absolute inset-0 bg-gradient-to-br from-accent/0 to-accent/10 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity" />
                <Zap className="w-6 h-6 text-accent mb-2 group-hover:scale-110 transition-transform" />
                <h3 className="text-base font-bold text-white mb-0.5">{t('dashboard.smartScan')}</h3>
                <p className="text-[10px] text-slate-400">{t('dashboard.smartScanDesc')}</p>
              </button>

              <button
                onClick={handleFullScan}
                className="group relative bg-gradient-to-br from-blue-500/20 to-cyan-500/20 hover:from-blue-500/30 hover:to-cyan-500/30 border border-blue-400/30 rounded-xl p-3 transition-all duration-300 hover:scale-105 hover:shadow-[0_0_30px_rgba(59,130,246,0.3)]"
              >
                <div className="absolute inset-0 bg-gradient-to-br from-blue-500/0 to-blue-500/10 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity" />
                <Scan className="w-6 h-6 text-blue-400 mb-2 group-hover:scale-110 transition-transform" />
                <h3 className="text-base font-bold text-white mb-0.5">{t('dashboard.fullScan')}</h3>
                <p className="text-[10px] text-slate-400">{t('dashboard.fullScanDesc')}</p>
              </button>
            </div>
          </div>

          {/* 威胁雷达 */}
          <div className={`bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border p-4 shadow-2xl animate-slideInRight overflow-hidden ${
            threatStatus.isSecure ? 'border-white/5' : 'border-red-500/30'
          }`}>
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <div className={`p-1.5 rounded-md ${threatStatus.isSecure ? 'bg-green-500/10' : 'bg-red-500/10'}`}>
                  <Activity className={`w-4 h-4 ${threatStatus.isSecure ? 'text-green-400' : 'text-red-400'}`} />
                </div>
                <div>
                  <h3 className="text-base font-bold text-white">{t('dashboard.threatRadar')}</h3>
                  <p className="text-[10px] text-slate-400">
                    {threatStatus.loading ? t('dashboard.detecting') : t('dashboard.realtimeScan')}
                  </p>
                </div>
              </div>
              {!threatStatus.loading && threatStatus.threatCount > 0 && (
                <div className="flex items-center gap-1 px-2 py-1 bg-red-500/20 rounded-lg">
                  <AlertTriangle className="w-3 h-3 text-red-400" />
                  <span className="text-xs text-red-400 font-bold">{threatStatus.threatCount}</span>
                </div>
              )}
            </div>

            {/* 雷达显示 */}
            <div className="relative flex items-center justify-center w-full aspect-square max-w-[220px] mx-auto">
              {/* 雷达背景圈 */}
              <svg className="absolute inset-0 w-full h-full" viewBox="0 0 200 200">
                <defs>
                  <radialGradient id="radar-gradient">
                    <stop offset="0%" stopColor={threatStatus.isSecure ? "#10b981" : "#ef4444"} stopOpacity="0.2" />
                    <stop offset="100%" stopColor={threatStatus.isSecure ? "#10b981" : "#ef4444"} stopOpacity="0" />
                  </radialGradient>
                  <linearGradient id="scan-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor={threatStatus.isSecure ? "#10b981" : "#ef4444"} stopOpacity="0" />
                    <stop offset="50%" stopColor={threatStatus.isSecure ? "#10b981" : "#ef4444"} stopOpacity="0.6" />
                    <stop offset="100%" stopColor={threatStatus.isSecure ? "#10b981" : "#ef4444"} stopOpacity="0" />
                  </linearGradient>
                </defs>

                {/* 雷达圆圈 - 根据状态变色 */}
                <circle cx="100" cy="100" r="92" fill="none" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.08)" : "rgba(239,68,68,0.08)"} strokeWidth="0.5" />
                <circle cx="100" cy="100" r="69" fill="none" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.12)" : "rgba(239,68,68,0.12)"} strokeWidth="0.5" />
                <circle cx="100" cy="100" r="46" fill="none" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.16)" : "rgba(239,68,68,0.16)"} strokeWidth="0.5" />
                <circle cx="100" cy="100" r="23" fill="none" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.2)" : "rgba(239,68,68,0.2)"} strokeWidth="0.5" />

                {/* 雷达十字线 */}
                <line x1="100" y1="8" x2="100" y2="192" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.1)" : "rgba(239,68,68,0.1)"} strokeWidth="0.5" />
                <line x1="8" y1="100" x2="192" y2="100" stroke={threatStatus.isSecure ? "rgba(16,185,129,0.1)" : "rgba(239,68,68,0.1)"} strokeWidth="0.5" />

                {/* 雷达扫描线 - 旋转动画 */}
                <g className="animate-radar-spin" style={{ transformOrigin: '100px 100px' }}>
                  <path
                    d="M 100 100 L 100 8 A 92 92 0 0 1 192 100 Z"
                    fill="url(#radar-gradient)"
                    opacity="0.8"
                  />
                  <line
                    x1="100"
                    y1="100"
                    x2="100"
                    y2="8"
                    stroke={threatStatus.isSecure ? "#10b981" : "#ef4444"}
                    strokeWidth="1.5"
                    strokeLinecap="round"
                  />
                </g>


                {/* 动态威胁点 */}
                {radarBlips.map(blip => (
                  <circle
                    key={blip.id}
                    cx={blip.x}
                    cy={blip.y}
                    r={blip.size}
                    fill={blip.color}
                    className="animate-radar-blip"
                  />
                ))}
              </svg>

              {/* 中心信息 */}
              <div className="relative z-10 text-center">
                {threatStatus.loading ? (
                  <div className="text-3xl font-bold text-blue-400 animate-pulse">{t('dashboard.detecting')}</div>
                ) : threatStatus.isSecure ? (
                  <div className="text-5xl font-bold text-green-400">{t('dashboard.safe')}</div>
                ) : (
                  <div className="flex flex-col items-center">
                    <div className="text-4xl font-bold text-red-400">{t('dashboard.warning')}</div>
                    <div className="text-xs text-red-300 mt-1">
                      {threatStatus.threatCount} {t('dashboard.threats')}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* 系统状态 - 横排显示 */}
        <div className="mb-3 animate-slideInUp" style={{ animationDelay: '0.2s' }}>
          <div className="flex items-center gap-2 mb-3">
            <Activity className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-bold text-white">{t('dashboard.systemStatus')}</h2>
          </div>

          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 animate-slideInUp" style={{ animationDelay: '0.05s' }}>
            {/* 系统环境 */}
            <div className="flex items-center justify-between p-4 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 hover:border-blue-500/30 transition-all shadow-xl">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-blue-500/10">
                  <Monitor className="w-5 h-5 text-blue-400" />
                </div>
                <div>
                  <div className="text-xs text-slate-500">{t('dashboard.systemEnv')}</div>
                  <div className="text-sm font-bold text-blue-400">{systemInfo.os}</div>
                </div>
              </div>
            </div>

            {/* 反分析保护 */}
            <div className={`flex items-center justify-between p-4 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border transition-all shadow-xl cursor-pointer ${
              threatStatus.isSecure ? 'border-white/5 hover:border-green-500/30' : 'border-red-500/30 hover:border-red-500/50'
            }`} onClick={() => navigate('/anti-analysis')}>
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${threatStatus.isSecure ? 'bg-green-500/10' : 'bg-red-500/10'}`}>
                  {threatStatus.loading ? (
                    <Activity className="w-5 h-5 text-blue-400 animate-pulse" />
                  ) : threatStatus.isSecure ? (
                    <CheckCircle2 className="w-5 h-5 text-green-400" />
                  ) : (
                    <XCircle className="w-5 h-5 text-red-400" />
                  )}
                </div>
                <div>
                  <div className="text-xs text-slate-500">{t('dashboard.antiAnalysis')}</div>
                  <div className={`text-sm font-bold ${
                    threatStatus.loading ? 'text-blue-400' : threatStatus.isSecure ? 'text-green-400' : 'text-red-400'
                  }`}>
                    {threatStatus.loading ? t('dashboard.detecting') : threatStatus.isSecure ? t('dashboard.safe') : `${threatStatus.threatCount} ${t('dashboard.threats')}`}
                  </div>
                </div>
              </div>
            </div>

            {/* 痕迹清理 */}
            <div className="flex items-center justify-between p-4 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 hover:border-green-500/30 transition-all shadow-xl">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-green-500/10">
                  <CheckCircle2 className="w-5 h-5 text-green-400" />
                </div>
                <div>
                  <div className="text-xs text-slate-500">{t('dashboard.traceCleanup')}</div>
                  <div className="text-sm font-bold text-green-400">{t('dashboard.ready')}</div>
                </div>
              </div>
            </div>

            {/* 网络监控 */}
            <div className="flex items-center justify-between p-4 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-xl border border-white/5 hover:border-green-500/30 transition-all shadow-xl">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-green-500/10">
                  <Wifi className="w-5 h-5 text-green-400" />
                </div>
                <div>
                  <div className="text-xs text-slate-500">{t('dashboard.networkMonitor')}</div>
                  <div className="text-sm font-bold text-green-400">{t('dashboard.monitoring')}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 实时监控 - 系统信息 */}
        <div>
          <div className="flex items-center gap-2 mb-4 animate-slideInUp">
            <Activity className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-bold text-white">{t('dashboard.realtimeMonitor')}</h2>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 animate-slideInUp" style={{ animationDelay: '0.1s' }}>
            {/* CPU使用率 */}
            <div className="group relative bg-slate-800/30 backdrop-blur-sm hover:bg-slate-800/50 border border-white/5 hover:border-accent/30 rounded-xl p-5 transition-all duration-300 overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-br from-purple-500 to-pink-500 opacity-0 group-hover:opacity-10 transition-opacity duration-300" />

              <div className="relative z-10">
                <div className="flex items-center justify-between mb-2">
                  <div className="text-purple-400 transition-transform duration-300">
                    <Cpu size={24} />
                  </div>
                  <div className="text-xl font-bold text-white">{systemInfo.cpuUsage.toFixed(1)}%</div>
                </div>
                <div className="text-xs text-accent/60 mb-1 font-medium">{t('dashboard.cpuUsage')}</div>
                {renderMiniChart(history.cpu, '#a855f7')}
              </div>

              <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                <div className="absolute top-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-purple-400 to-transparent" />
                <div className="absolute bottom-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-purple-400 to-transparent" />
              </div>
            </div>

            {/* 内存使用 */}
            <div className="group relative bg-slate-800/30 backdrop-blur-sm hover:bg-slate-800/50 border border-white/5 hover:border-accent/30 rounded-xl p-5 transition-all duration-300 overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-br from-orange-500 to-red-500 opacity-0 group-hover:opacity-10 transition-opacity duration-300" />

              <div className="relative z-10">
                <div className="flex items-center justify-between mb-2">
                  <div className="text-orange-400 transition-transform duration-300">
                    <MemoryStick size={24} />
                  </div>
                  <div className="text-sm font-bold text-white">
                    {(systemInfo.usedMemory / 1024).toFixed(1)} GB
                  </div>
                </div>
                <div className="text-xs text-accent/60 mb-1 font-medium">
                  {t('dashboard.memoryUsage')} ({((systemInfo.usedMemory / systemInfo.totalMemory) * 100).toFixed(0)}%)
                </div>
                {renderMiniChart(history.memory, '#f97316')}
              </div>

              <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                <div className="absolute top-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-orange-400 to-transparent" />
                <div className="absolute bottom-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-orange-400 to-transparent" />
              </div>
            </div>

            {/* 网络速度 */}
            <div className="group relative bg-slate-800/30 backdrop-blur-sm hover:bg-slate-800/50 border border-white/5 hover:border-accent/30 rounded-xl p-5 transition-all duration-300 overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-br from-green-500 to-emerald-500 opacity-0 group-hover:opacity-10 transition-opacity duration-300" />

              <div className="relative z-10">
                <div className="flex items-center justify-between mb-2">
                  <div className="text-green-400 transition-transform duration-300">
                    <Wifi size={24} />
                  </div>
                  <div className="text-xs font-bold text-white">
                    ↓ {formatNetworkSpeed(networkSpeed.download).fullText}
                  </div>
                </div>
                <div className="text-xs text-accent/60 mb-1 font-medium">
                  {t('dashboard.networkSpeed')} (↑ {formatNetworkSpeed(networkSpeed.upload).fullText})
                </div>
                <div className="relative h-16 mt-2">
                  {/* 下载速度图表 */}
                  <svg className="absolute inset-0 w-full h-16" viewBox="0 0 100 100" preserveAspectRatio="none">
                    <defs>
                      <linearGradient id="gradient-download" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" stopColor="#10b981" stopOpacity="0.3" />
                        <stop offset="100%" stopColor="#10b981" stopOpacity="0.05" />
                      </linearGradient>
                    </defs>
                    <polygon
                      points={`0,100 ${history.download.map((value, index) => {
                        const maxDownload = Math.max(...history.download, 0.01);
                        const x = (index / (history.download.length - 1)) * 100;
                        const y = 100 - (value / maxDownload) * 80;
                        return `${x},${y}`;
                      }).join(' ')} 100,100`}
                      fill="url(#gradient-download)"
                    />
                    <polyline
                      points={history.download.map((value, index) => {
                        const maxDownload = Math.max(...history.download, 0.01);
                        const x = (index / (history.download.length - 1)) * 100;
                        const y = 100 - (value / maxDownload) * 80;
                        return `${x},${y}`;
                      }).join(' ')}
                      fill="none"
                      stroke="#10b981"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      opacity="0.8"
                    />
                  </svg>
                  {/* 上传速度图表（半透明覆盖） */}
                  <svg className="absolute inset-0 w-full h-16" viewBox="0 0 100 100" preserveAspectRatio="none">
                    <polyline
                      points={history.upload.map((value, index) => {
                        const maxUpload = Math.max(...history.upload, 0.01);
                        const x = (index / (history.upload.length - 1)) * 100;
                        const y = 100 - (value / maxUpload) * 80;
                        return `${x},${y}`;
                      }).join(' ')}
                      fill="none"
                      stroke="#34d399"
                      strokeWidth="1"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeDasharray="2,2"
                      opacity="0.5"
                    />
                  </svg>
                </div>
              </div>

              <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                <div className="absolute top-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-green-400 to-transparent" />
                <div className="absolute bottom-0 left-0 w-full h-0.5 bg-gradient-to-r from-transparent via-green-400 to-transparent" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
