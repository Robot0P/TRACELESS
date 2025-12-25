import React, { useState, useEffect } from 'react';
import { Button, Progress, Modal, Badge, Tooltip, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Wifi,
  Network,
  Globe,
  CheckCircle2,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  Activity,
  Radio,
  Router,
  Server,
  Trash2,
  RefreshCw,
  Shield,
  Link,
  Unlink,
  Eye,
  EyeOff,
  Monitor,
} from 'lucide-react';

// 后端数据结构
interface ConnectionInfo {
  protocol: string;
  local_address: string;
  local_port: number;
  remote_address: string;
  remote_port: number;
  state: string;
  pid?: number;
  process_name?: string;
}

interface WifiNetwork {
  ssid: string;
  security: string;
  auto_connect: boolean;
  last_connected?: string;
}

interface VpnConnection {
  name: string;
  vpn_type: string;
  server: string;
  connected: boolean;
}

interface ProxySettings {
  http_proxy?: string;
  https_proxy?: string;
  socks_proxy?: string;
  proxy_enabled: boolean;
  pac_url?: string;
}

interface NetworkInterface {
  name: string;
  ip_address?: string;
  mac_address?: string;
  status: string;
  interface_type: string;
}

interface NetworkInfo {
  active_connections: ConnectionInfo[];
  dns_servers: string[];
  wifi_networks: WifiNetwork[];
  vpn_connections: VpnConnection[];
  proxy_settings: ProxySettings;
  network_interfaces: NetworkInterface[];
  dns_cache_count: number;
  arp_cache_count: number;
  routing_entries: number;
}

interface NetworkItem {
  value: string;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  gradient: string;
  category: string;
  count: number;
  platform: 'all' | 'macos' | 'windows' | 'linux';
}

// 扫描项接口
interface NetworkCleanItem {
  item_type: string;
  label: string;
  description: string;
  count: number;
  count_display: string;
  accessible: boolean;
  category: string;
}

// 扫描结果接口
interface NetworkScanResult {
  items: NetworkCleanItem[];
  total_items: number;
  network_info: NetworkInfo;
}

const NetworkCleanup: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedItems, setSelectedItems] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [showSuccess, setShowSuccess] = useState(false);
  const [currentItem, setCurrentItem] = useState('');
  const [refreshing, setRefreshing] = useState(false);
  const [platform, setPlatform] = useState<string>('macos');
  const [scanning, setScanning] = useState(true);
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  // 真实网络数据
  const [networkInfo, setNetworkInfo] = useState<NetworkInfo | null>(null);

  // 扫描结果 (保留用于未来扩展显示扫描项详情)
  const [_scanResult, setScanResult] = useState<NetworkScanResult | null>(null);

  // 显示详情面板
  const [showConnections, setShowConnections] = useState(false);
  const [showWifi, setShowWifi] = useState(false);
  const [showVpn, setShowVpn] = useState(false);

  // 根据真实数据动态生成网络项目
  const getNetworkItems = (): NetworkItem[] => {
    const items: NetworkItem[] = [
      {
        value: 'dns_cache',
        label: t('networkCleanup.types.dns.label'),
        description: t('networkCleanup.types.dns.desc'),
        icon: <Globe size={24} />,
        color: 'text-blue-400',
        gradient: 'from-blue-500 to-cyan-500',
        category: 'DNS',
        count: networkInfo?.dns_cache_count || 0,
        platform: 'all',
      },
      {
        value: 'arp_table',
        label: t('networkCleanup.types.arp.label'),
        description: t('networkCleanup.types.arp.desc'),
        icon: <Network size={24} />,
        color: 'text-green-400',
        gradient: 'from-green-500 to-emerald-500',
        category: 'ARP',
        count: networkInfo?.arp_cache_count || 0,
        platform: 'all',
      },
      {
        value: 'routing_table',
        label: t('networkCleanup.types.routing.label'),
        description: t('networkCleanup.types.routing.desc'),
        icon: <Router size={24} />,
        color: 'text-orange-400',
        gradient: 'from-orange-500 to-red-500',
        category: 'Routing',
        count: networkInfo?.routing_entries || 0,
        platform: 'all',
      },
      {
        value: 'wifi_profiles',
        label: t('networkCleanup.wifiProfiles'),
        description: t('networkCleanup.wifiProfilesDesc'),
        icon: <Wifi size={24} />,
        color: 'text-cyan-400',
        gradient: 'from-cyan-500 to-teal-500',
        category: 'WiFi',
        count: networkInfo?.wifi_networks?.length || 0,
        platform: 'all',
      },
      {
        value: 'connection_history',
        label: t('networkCleanup.connectionHistory'),
        description: t('networkCleanup.connectionHistoryDesc'),
        icon: <Activity size={24} />,
        color: 'text-pink-400',
        gradient: 'from-pink-500 to-rose-500',
        category: 'History',
        count: networkInfo?.active_connections?.length || 0,
        platform: 'all',
      },
      {
        value: 'proxy_settings',
        label: t('networkCleanup.proxySettings'),
        description: t('networkCleanup.proxySettingsDesc'),
        icon: <Shield size={24} />,
        color: 'text-purple-400',
        gradient: 'from-purple-500 to-indigo-500',
        category: 'Proxy',
        count: networkInfo?.proxy_settings?.proxy_enabled ? 1 : 0,
        platform: 'all',
      },
      {
        value: 'vpn_disconnect',
        label: t('networkCleanup.vpnConnection'),
        description: t('networkCleanup.vpnConnectionDesc'),
        icon: <Link size={24} />,
        color: 'text-amber-400',
        gradient: 'from-amber-500 to-yellow-500',
        category: 'VPN',
        count: networkInfo?.vpn_connections?.filter(v => v.connected).length || 0,
        platform: 'all',
      },
    ];

    // Windows 专有
    if (platform === 'windows') {
      items.push({
        value: 'netbios_cache',
        label: t('networkCleanup.types.netbios.label'),
        description: t('networkCleanup.types.netbios.desc'),
        icon: <Server size={24} />,
        color: 'text-indigo-400',
        gradient: 'from-indigo-500 to-purple-500',
        category: 'NetBIOS',
        count: 0,
        platform: 'windows',
      });
    }

    return items.filter(item => item.platform === 'all' || item.platform === platform);
  };

  const networkItems = getNetworkItems();

  useEffect(() => {
    // 获取平台信息
    invoke<string>('get_platform').then((p) => {
      setPlatform(p.toLowerCase());
    }).catch(() => {
      setPlatform('macos');
    });

    // 执行扫描
    performScan();
  }, []);

  // 执行网络扫描
  const performScan = async () => {
    setScanning(true);
    const startTime = Date.now();
    const minDisplayTime = 800; // 最小显示时间 800ms，确保用户能看到扫描动画

    try {
      // 调用扫描命令
      const result = await invoke<NetworkScanResult>('scan_network_items');
      setScanResult(result);
      setNetworkInfo(result.network_info);

      // 默认选中所有可访问的项目
      const accessibleItems = result.items
        .filter(item => item.accessible)
        .map(item => item.item_type);
      setSelectedItems(accessibleItems);
    } catch {
      // 如果扫描失败，回退到基本模式
      fetchNetworkInfo();
    } finally {
      // 确保扫描动画至少显示 minDisplayTime
      const elapsed = Date.now() - startTime;
      if (elapsed < minDisplayTime) {
        await new Promise(resolve => setTimeout(resolve, minDisplayTime - elapsed));
      }
      setScanning(false);
    }
  };

  // 定时刷新网络信息
  useEffect(() => {
    if (!scanning) {
      const interval = setInterval(fetchNetworkInfo, 5000);
      return () => clearInterval(interval);
    }
  }, [scanning]);

  const fetchNetworkInfo = async () => {
    try {
      const info = await invoke<NetworkInfo>('get_network_info');
      setNetworkInfo(info);
    } catch {
      // Silently fail
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await performScan();
    setTimeout(() => setRefreshing(false), 500);
  };

  const handleToggleItem = (value: string) => {
    setSelectedItems(prev =>
      prev.includes(value)
        ? prev.filter(v => v !== value)
        : [...prev, value]
    );
  };

  const handleSelectAll = () => {
    if (selectedItems.length === networkItems.length) {
      setSelectedItems([]);
    } else {
      setSelectedItems(networkItems.map(item => item.value));
    }
  };

  const handleClean = async () => {
    if (selectedItems.length === 0) {
      Modal.warning({
        title: t('common.warning'),
        content: t('networkCleanup.warnings.selectItems'),
        centered: true,
      });
      return;
    }

    setLoading(true);
    setProgress(0);

    // 参数映射：前端值 -> 后端值
    const paramMap: Record<string, string> = {
      'dns_cache': 'dns',
      'arp_table': 'arp',
      'netbios_cache': 'netbios',
      'routing_table': 'routing',
      'wifi_profiles': 'wifi',
      'connection_history': 'history',
      'proxy_settings': 'proxy',
      'vpn_disconnect': 'vpn',
    };

    try {
      for (let i = 0; i < selectedItems.length; i++) {
        const itemValue = selectedItems[i];
        const itemData = networkItems.find(item => item.value === itemValue);

        setCurrentItem(itemData?.label || itemValue);

        // 进度动画
        for (let j = 0; j <= 10; j++) {
          await new Promise(resolve => setTimeout(resolve, 80));
          const itemProgress = ((i + j / 10) / selectedItems.length) * 100;
          setProgress(Math.min(itemProgress, 100));
        }

        // 映射参数并调用实际的清理命令
        const backendParam = paramMap[itemValue] || itemValue;
        await invoke('clean_network', {
          types: [backendParam],
        });
      }

      // 刷新网络信息
      await fetchNetworkInfo();

      setLoading(false);
      setShowSuccess(true);
    } catch (error) {
      setLoading(false);
      Modal.error({
        title: t('networkCleanup.errors.cleanFailed'),
        content: String(error),
        centered: true,
      });
    }
  };

  const totalCount = networkItems
    .filter(item => selectedItems.includes(item.value))
    .reduce((sum, item) => sum + item.count, 0);

  const getStateColor = (state: string) => {
    switch (state.toLowerCase()) {
      case 'established': return 'text-green-400';
      case 'listen': return 'text-blue-400';
      case 'time_wait': return 'text-yellow-400';
      case 'close_wait': return 'text-orange-400';
      default: return 'text-slate-400';
    }
  };

  return (
    <ProFeatureGate
      feature="network_cleanup"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/3 left-1/2 w-96 h-96 bg-green-500/5 rounded-full blur-3xl animate-pulse" />
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
                <Network className="w-7 h-7 text-green-400" />
                {t('networkCleanup.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('networkCleanup.subtitle')}
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
            <Badge count={networkInfo?.active_connections?.length || 0} overflowCount={99} color="#10b981">
              <div className="flex items-center gap-2 px-4 py-2 bg-green-500/10 border border-green-500/20 rounded-lg">
                <Radio className="w-4 h-4 text-green-400 animate-pulse" />
                <span className="text-sm text-green-300">{t('networkCleanup.activeConnections')}</span>
              </div>
            </Badge>
          </div>
        </div>

        {scanning ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Loader2 className="w-12 h-12 text-green-400 animate-spin mx-auto mb-4" />
              <p className="text-slate-400">{t('networkCleanup.scanning')}</p>
            </div>
          </div>
        ) : (
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden min-h-0">
          {/* 左侧：网络状态概览 + 项目列表 */}
          <div className="lg:col-span-2 flex flex-col space-y-6 min-h-0 overflow-y-auto custom-scrollbar">
            {/* 网络状态概览 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Monitor className="w-5 h-5 text-green-400" />
                  <h3 className="text-lg font-bold text-white">{t('networkCleanup.networkStatus')}</h3>
                </div>
                <span className="text-xs text-slate-400 capitalize">{platform}</span>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {/* 活动连接 */}
                <Tooltip title={t('networkCleanup.clickDetails')}>
                  <button
                    onClick={() => setShowConnections(!showConnections)}
                    className="p-3 bg-green-500/10 rounded-lg border border-green-500/20 hover:bg-green-500/20 transition-colors text-left"
                  >
                    <div className="flex items-center justify-between mb-1">
                      <Activity className="w-4 h-4 text-green-400" />
                      {showConnections ? <EyeOff size={12} className="text-green-400" /> : <Eye size={12} className="text-green-400" />}
                    </div>
                    <div className="text-xl font-bold text-green-400">
                      {networkInfo?.active_connections?.length || 0}
                    </div>
                    <div className="text-xs text-slate-400">{t('networkCleanup.activeConnections')}</div>
                  </button>
                </Tooltip>

                {/* WiFi 网络 */}
                <Tooltip title={t('networkCleanup.clickDetails')}>
                  <button
                    onClick={() => setShowWifi(!showWifi)}
                    className="p-3 bg-cyan-500/10 rounded-lg border border-cyan-500/20 hover:bg-cyan-500/20 transition-colors text-left"
                  >
                    <div className="flex items-center justify-between mb-1">
                      <Wifi className="w-4 h-4 text-cyan-400" />
                      {showWifi ? <EyeOff size={12} className="text-cyan-400" /> : <Eye size={12} className="text-cyan-400" />}
                    </div>
                    <div className="text-xl font-bold text-cyan-400">
                      {networkInfo?.wifi_networks?.length || 0}
                    </div>
                    <div className="text-xs text-slate-400">{t('networkCleanup.wifiProfiles')}</div>
                  </button>
                </Tooltip>

                {/* VPN */}
                <Tooltip title={t('networkCleanup.clickDetails')}>
                  <button
                    onClick={() => setShowVpn(!showVpn)}
                    className="p-3 bg-amber-500/10 rounded-lg border border-amber-500/20 hover:bg-amber-500/20 transition-colors text-left"
                  >
                    <div className="flex items-center justify-between mb-1">
                      <Shield className="w-4 h-4 text-amber-400" />
                      {showVpn ? <EyeOff size={12} className="text-amber-400" /> : <Eye size={12} className="text-amber-400" />}
                    </div>
                    <div className="text-xl font-bold text-amber-400">
                      {networkInfo?.vpn_connections?.length || 0}
                    </div>
                    <div className="text-xs text-slate-400">{t('networkCleanup.vpnProfiles')}</div>
                  </button>
                </Tooltip>

                {/* DNS 服务器 */}
                <div className="p-3 bg-blue-500/10 rounded-lg border border-blue-500/20">
                  <Globe className="w-4 h-4 text-blue-400 mb-1" />
                  <div className="text-xl font-bold text-blue-400">
                    {networkInfo?.dns_servers?.length || 0}
                  </div>
                  <div className="text-xs text-slate-400">{t('networkCleanup.dnsServers')}</div>
                </div>

                {/* 代理状态 */}
                <div className={`p-3 rounded-lg border ${networkInfo?.proxy_settings?.proxy_enabled ? 'bg-purple-500/10 border-purple-500/20' : 'bg-slate-700/30 border-slate-600/20'}`}>
                  <Shield className={`w-4 h-4 mb-1 ${networkInfo?.proxy_settings?.proxy_enabled ? 'text-purple-400' : 'text-slate-500'}`} />
                  <div className={`text-xl font-bold ${networkInfo?.proxy_settings?.proxy_enabled ? 'text-purple-400' : 'text-slate-500'}`}>
                    {networkInfo?.proxy_settings?.proxy_enabled ? t('networkCleanup.enabled') : t('networkCleanup.disabled')}
                  </div>
                  <div className="text-xs text-slate-400">{t('networkCleanup.proxyStatus')}</div>
                </div>

                {/* 网络接口 */}
                <div className="p-3 bg-indigo-500/10 rounded-lg border border-indigo-500/20">
                  <Router className="w-4 h-4 text-indigo-400 mb-1" />
                  <div className="text-xl font-bold text-indigo-400">
                    {networkInfo?.network_interfaces?.filter(i => i.status === 'active').length || 0}
                  </div>
                  <div className="text-xs text-slate-400">{t('networkCleanup.activeInterfaces')}</div>
                </div>

                {/* ARP 缓存 */}
                <div className="p-3 bg-orange-500/10 rounded-lg border border-orange-500/20">
                  <Network className="w-4 h-4 text-orange-400 mb-1" />
                  <div className="text-xl font-bold text-orange-400">
                    {networkInfo?.arp_cache_count || 0}
                  </div>
                  <div className="text-xs text-slate-400">{t('networkCleanup.arpCache')}</div>
                </div>

                {/* 路由表 */}
                <div className="p-3 bg-pink-500/10 rounded-lg border border-pink-500/20">
                  <Router className="w-4 h-4 text-pink-400 mb-1" />
                  <div className="text-xl font-bold text-pink-400">
                    {networkInfo?.routing_entries || 0}
                  </div>
                  <div className="text-xs text-slate-400">{t('networkCleanup.routingEntries')}</div>
                </div>
              </div>
            </div>

            {/* 连接详情 */}
            {showConnections && networkInfo?.active_connections && networkInfo.active_connections.length > 0 && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
                <div className="flex items-center gap-2 mb-4">
                  <Activity className="w-5 h-5 text-green-400" />
                  <h3 className="text-lg font-bold text-white">{t('networkCleanup.connectionDetails')}</h3>
                  <span className="text-xs text-slate-400">{t('networkCleanup.first15')}</span>
                </div>
                <div className="space-y-2 max-h-48 overflow-y-auto custom-scrollbar">
                  {networkInfo.active_connections.slice(0, 15).map((conn, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-2 bg-slate-800/30 rounded-lg text-xs"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-slate-500 w-4">{index + 1}</span>
                        <span className="text-blue-400 font-mono">{conn.protocol}</span>
                        <span className="text-white font-mono truncate max-w-[120px]">
                          {conn.local_address}:{conn.local_port}
                        </span>
                        <span className="text-slate-500">→</span>
                        <span className="text-slate-300 font-mono truncate max-w-[120px]">
                          {conn.remote_address}:{conn.remote_port}
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-0.5 rounded ${getStateColor(conn.state)} bg-slate-700/50`}>
                          {conn.state}
                        </span>
                        {conn.pid && (
                          <span className="text-slate-500">PID: {conn.pid}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* WiFi 详情 */}
            {showWifi && networkInfo?.wifi_networks && networkInfo.wifi_networks.length > 0 && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
                <div className="flex items-center gap-2 mb-4">
                  <Wifi className="w-5 h-5 text-cyan-400" />
                  <h3 className="text-lg font-bold text-white">{t('networkCleanup.savedWifi')}</h3>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                  {networkInfo.wifi_networks.map((wifi, index) => (
                    <div
                      key={index}
                      className="p-3 bg-slate-800/30 rounded-lg border border-cyan-500/10"
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <Wifi size={14} className="text-cyan-400" />
                        <span className="text-white font-medium truncate">{wifi.ssid}</span>
                      </div>
                      <div className="flex items-center gap-2 text-xs text-slate-400">
                        <span>{wifi.security}</span>
                        {wifi.auto_connect && (
                          <span className="text-green-400">{t('networkCleanup.autoConnect')}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* VPN 详情 */}
            {showVpn && networkInfo?.vpn_connections && networkInfo.vpn_connections.length > 0 && (
              <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
                <div className="flex items-center gap-2 mb-4">
                  <Shield className="w-5 h-5 text-amber-400" />
                  <h3 className="text-lg font-bold text-white">{t('networkCleanup.vpnProfiles')}</h3>
                </div>
                <div className="space-y-2">
                  {networkInfo.vpn_connections.map((vpn, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-3 bg-slate-800/30 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        {vpn.connected ? (
                          <Link className="w-4 h-4 text-green-400" />
                        ) : (
                          <Unlink className="w-4 h-4 text-slate-500" />
                        )}
                        <div>
                          <div className="text-white font-medium">{vpn.name}</div>
                          <div className="text-xs text-slate-400">
                            {vpn.vpn_type} {vpn.server && `• ${vpn.server}`}
                          </div>
                        </div>
                      </div>
                      <span className={`px-2 py-1 rounded text-xs ${vpn.connected ? 'bg-green-500/20 text-green-400' : 'bg-slate-700/50 text-slate-400'}`}>
                        {vpn.connected ? t('networkCleanup.connected') : t('networkCleanup.notConnected')}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 网络项目卡片 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Trash2 className="w-5 h-5 text-green-400" />
                  <h3 className="text-lg font-bold text-white">{t('networkCleanup.cleanupItems')}</h3>
                  <span className="text-sm text-slate-400">({selectedItems.length}/{networkItems.length})</span>
                </div>
                <Button
                  onClick={handleSelectAll}
                  size="small"
                  className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
                >
                  {selectedItems.length === networkItems.length ? t('networkCleanup.deselectAll') : t('networkCleanup.selectAll')}
                </Button>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                {networkItems.map((item) => {
                  const isSelected = selectedItems.includes(item.value);
                  return (
                    <Tooltip key={item.value} title={item.description}>
                      <button
                        onClick={() => handleToggleItem(item.value)}
                        className={`
                          relative p-4 rounded-xl border-2 transition-all duration-300 text-left overflow-hidden
                          ${isSelected
                            ? 'border-green-400/30 bg-green-500/10 shadow-lg shadow-green-500/20'
                            : 'border-white/10 hover:border-white/20 bg-slate-800/30'
                          }
                        `}
                      >
                        <div className={`absolute inset-0 bg-gradient-to-br ${item.gradient} opacity-0 ${isSelected ? 'opacity-10' : ''} transition-opacity`} />

                        <div className="relative z-10">
                          <div className="flex items-center justify-between mb-2">
                            <div className={`${item.color}`}>{item.icon}</div>
                            <div onClick={(e) => e.stopPropagation()}>
                              <Switch
                                checked={isSelected}
                                size="small"
                                onChange={() => handleToggleItem(item.value)}
                              />
                            </div>
                          </div>
                          <div className="flex items-center justify-between mb-1">
                            <div className="text-white font-bold text-sm">{item.label}</div>
                            <Badge
                              count={item.count}
                              overflowCount={999}
                              color={isSelected ? '#10b981' : '#64748b'}
                            />
                          </div>
                          <div className="text-xs text-slate-400 line-clamp-1">{item.description}</div>
                        </div>
                      </button>
                    </Tooltip>
                  );
                })}
              </div>
            </div>
          </div>

          {/* 右侧：控制面板 */}
          <div className="flex flex-col space-y-6 min-h-0">
            {/* 控制面板 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight">
              <h3 className="text-lg font-bold text-white mb-4">{t('networkCleanup.controlPanel')}</h3>

              <div className="space-y-4">
                <div className="p-4 bg-green-500/10 rounded-lg border border-green-500/20">
                  <div className="text-sm text-slate-400 mb-1">{t('networkCleanup.estimatedCleanup')}</div>
                  <div className="text-3xl font-bold text-green-400">{totalCount}</div>
                  <div className="text-xs text-green-300 mt-1">{t('networkCleanup.networkRecords')}</div>
                </div>

                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('networkCleanup.selectedItems')}</span>
                    <span className="text-white font-medium">{selectedItems.length} {t('networkCleanup.items')}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('networkCleanup.currentPlatform')}</span>
                    <span className="text-accent font-medium capitalize">{platform}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('networkCleanup.cleanupMethod')}</span>
                    <span className="text-green-400 font-medium">{t('networkCleanup.safeCleanup')}</span>
                  </div>
                </div>

                <Button
                  type="primary"
                  size="large"
                  block
                  onClick={handleClean}
                  disabled={selectedItems.length === 0 || loading}
                  className="h-12 bg-gradient-to-r from-green-600 to-green-500 border-none hover:from-green-500 hover:to-green-400 text-white font-bold mt-4"
                  icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Trash2 size={20} />}
                >
                  {loading ? t('networkCleanup.cleaning') : t('networkCleanup.startClean')}
                </Button>

                {loading && (
                  <div className="mt-4 space-y-3">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-slate-300">{t('networkCleanup.cleanupProgress')}</span>
                      <span className="text-green-400 font-mono">{Math.floor(progress)}%</span>
                    </div>
                    <Progress
                      percent={progress}
                      strokeColor={{
                        '0%': '#10b981',
                        '100%': '#059669',
                      }}
                      trailColor="rgba(255,255,255,0.05)"
                      showInfo={false}
                      strokeWidth={10}
                    />
                    <div className="text-xs text-slate-400 truncate">
                      {currentItem}
                    </div>
                  </div>
                )}

                <div className="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                  <div className="flex items-start gap-2">
                    <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
                    <div className="text-xs text-amber-300">
                      {t('networkCleanup.warnings.wifiWarning')}
                    </div>
                  </div>
                </div>
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

          <h2 className="text-2xl font-bold text-white mb-2">{t('networkCleanup.cleanComplete')}</h2>
          <p className="text-slate-400 mb-2">
            {t('networkCleanup.cleanedItems')} <span className="text-accent font-bold">{selectedItems.length}</span>
          </p>
          <p className="text-green-400 text-sm mb-6">
            {t('networkCleanup.networkRecordsCleared')} {totalCount} {t('networkCleanup.networkRecords')}
          </p>

          <Button
            type="primary"
            size="large"
            onClick={() => {
              setShowSuccess(false);
              setSelectedItems([]);
              fetchNetworkInfo();
            }}
            className="bg-accent hover:bg-accent/80 border-none"
          >
            {t('common.confirm')}
          </Button>
        </div>
      </Modal>
      <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </ProFeatureGate>
  );
};

export default NetworkCleanup;
