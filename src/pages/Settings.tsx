import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '../contexts/SettingsContext';
import {
  Settings as SettingsIcon,
  Globe,
  Bell,
  Palette,
  Database,
  Info,
  Save,
  RotateCcw,
  Trash2,
  FileText,
  Cpu,
  Network,
  Clock,
  Eye,
  Lock,
  ShieldCheck,
  ShieldX,
  RefreshCw,
  Loader2,
  HardDrive,
  CheckCircle2,
  Sun,
  Moon,
  Monitor,
} from 'lucide-react';
import { Switch, Select, Button, Card, Divider, InputNumber, App } from 'antd';

const { Option } = Select;

interface SettingsDbInfo {
  path: string;
  size_bytes: number;
  exists: boolean;
  encrypted: boolean;
}

interface SystemInfo {
  os: string;
  version: string;
  total_memory: number; // MB
  used_memory: number;  // MB
  cpu_usage: number;    // percentage
  cpu_count: number;
}

const Settings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSettings, resetSettings, markAsSaved, loading: settingsLoading } = useSettings();
  const { message } = App.useApp();

  // 保存状态
  const [saving, setSaving] = useState(false);

  // 数据库信息
  const [dbInfo, setDbInfo] = useState<SettingsDbInfo | null>(null);

  // 系统信息
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  // 权限状态
  const [permissionStatus, setPermissionStatus] = useState<{
    initialized: boolean;
    isAdmin: boolean;
    checking: boolean;
    requesting: boolean;
  }>({
    initialized: false,
    isAdmin: false,
    checking: true,
    requesting: false,
  });

  // 检查权限状态
  const checkPermissionStatus = async () => {
    setPermissionStatus(prev => ({ ...prev, checking: true }));
    try {
      const [initialized, isAdmin] = await Promise.all([
        invoke<boolean>('check_permission_initialized'),
        invoke<boolean>('check_admin_permission'),
      ]);
      setPermissionStatus({
        initialized,
        isAdmin,
        checking: false,
        requesting: false,
      });
    } catch {
      setPermissionStatus(prev => ({ ...prev, checking: false }));
    }
  };

  // 请求权限
  const handleRequestPermission = async () => {
    setPermissionStatus(prev => ({ ...prev, requesting: true }));
    try {
      await invoke<string>('initialize_permissions');
      message.success(t('settings.messages.permissionSuccess'));
      await checkPermissionStatus();
    } catch (error) {
      message.error(typeof error === 'string' ? error : t('settings.messages.permissionFailed'));
      setPermissionStatus(prev => ({ ...prev, requesting: false }));
    }
  };

  // 加载数据库信息
  const loadDbInfo = async () => {
    try {
      const info = await invoke<SettingsDbInfo>('get_settings_info');
      setDbInfo(info);
    } catch {
      // Silently fail
    }
  };

  // 加载系统信息
  const loadSystemInfo = async () => {
    try {
      const info = await invoke<SystemInfo>('get_system_info_api');
      setSystemInfo(info);
    } catch {
      // Silently fail
    }
  };

  // 初始化
  useEffect(() => {
    loadDbInfo();
    loadSystemInfo();
    checkPermissionStatus();
  }, []);

  // 处理设置变更
  const handleSettingChange = (key: string, value: any) => {
    updateSettings({ [key]: value });
  };

  // 保存设置到后端 (加密 SQLite) - 直接调用 invoke
  const handleSave = async () => {
    setSaving(true);
    try {
      // 直接调用 invoke，参考 FileCleanup.tsx 的实现方式
      await invoke('save_settings', { settings });
      message.success(t('settings.messages.savedSuccess'));
      // 刷新数据库信息
      loadDbInfo();
      // 标记 context 中的设置已保存
      markAsSaved();
    } catch (error) {
      message.error(typeof error === 'string' ? error : t('settings.messages.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  // 重置设置
  const handleReset = async () => {
    setSaving(true);
    try {
      await resetSettings();
      message.success(t('settings.messages.resetSuccess'));
    } catch {
      message.error(t('settings.messages.resetFailed'));
    } finally {
      setSaving(false);
    }
  };

  // 格式化文件大小
  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  // 获取主题图标
  const getThemeIcon = (theme: string) => {
    switch (theme) {
      case 'light': return <Sun className="w-4 h-4 text-yellow-400" />;
      case 'dark': return <Moon className="w-4 h-4 text-blue-400" />;
      case 'auto': return <Monitor className="w-4 h-4 text-purple-400" />;
      default: return <Palette className="w-4 h-4" />;
    }
  };

  if (settingsLoading) {
    return (
      <div className="h-full flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="text-center">
          <Loader2 className="w-12 h-12 text-accent animate-spin mx-auto mb-4" />
          <p className="text-slate-400">{t('common.loading')}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
              <SettingsIcon className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">{t('settings.title', '偏好设置')}</h1>
              <p className="text-gray-400 text-sm">{t('settings.subtitle', '自定义您的应用体验 (加密存储)')}</p>
            </div>
          </div>

          {/* 操作按钮 */}
          <div className="flex gap-3">
            <Button
              icon={<RotateCcw className="w-4 h-4" />}
              onClick={handleReset}
              disabled={saving}
              className="flex items-center gap-2"
            >
              {t('settings.reset', '重置')}
            </Button>
            <Button
              type="primary"
              icon={saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
              onClick={handleSave}
              disabled={saving}
              className="flex items-center gap-2"
            >
              {saving ? t('settings.saving', '保存中...') : t('settings.save', '保存设置')}
            </Button>
          </div>
        </div>

        {/* 设置卡片网格 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* 外观设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Palette className="w-5 h-5 text-purple-400" />
                <span>{t('settings.appearance', '外观设置')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 主题设置 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium flex items-center gap-2">
                    {getThemeIcon(settings.theme)}
                    {t('settings.theme', '主题模式')}
                  </div>
                  <div className="text-gray-400 text-sm">{t('settings.themeDesc', '选择应用主题')}</div>
                </div>
                <Select
                  value={settings.theme}
                  onChange={(value) => handleSettingChange('theme', value)}
                  className="w-32"
                >
                  <Option value="dark">
                    <div className="flex items-center gap-2">
                      <Moon className="w-4 h-4" />
                      {t('settings.themeDark', '暗色')}
                    </div>
                  </Option>
                  <Option value="light">
                    <div className="flex items-center gap-2">
                      <Sun className="w-4 h-4" />
                      {t('settings.themeLight', '亮色')}
                    </div>
                  </Option>
                  <Option value="auto">
                    <div className="flex items-center gap-2">
                      <Monitor className="w-4 h-4" />
                      {t('settings.themeAuto', '自动')}
                    </div>
                  </Option>
                </Select>
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 语言设置 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium flex items-center gap-2">
                    <Globe className="w-4 h-4 text-blue-400" />
                    {t('settings.language', '语言')}
                  </div>
                  <div className="text-gray-400 text-sm">{t('settings.languageDesc', '选择界面语言')}</div>
                </div>
                <Select
                  value={settings.language}
                  onChange={(value) => handleSettingChange('language', value)}
                  className="w-32"
                >
                  <Option value="auto">
                    <div className="flex items-center gap-2">
                      <Monitor className="w-4 h-4" />
                      {t('settings.languageAuto', '跟随系统')}
                    </div>
                  </Option>
                  <Option value="zh-CN">简体中文</Option>
                  <Option value="en-US">English</Option>
                </Select>
              </div>
            </div>
          </Card>

          {/* 通知设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Bell className="w-5 h-5 text-yellow-400" />
                <span>{t('settings.notifications', '通知设置')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 启用通知 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.enableNotifications', '启用通知')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.enableNotificationsDesc', '接收系统通知')}</div>
                </div>
                <Switch
                  checked={settings.notifications}
                  onChange={(checked) => handleSettingChange('notifications', checked)}
                />
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 自动清理通知 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoCleanupNotify', '自动清理通知')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoCleanupNotifyDesc', '完成自动清理时通知')}</div>
                </div>
                <Switch
                  checked={settings.auto_cleanup}
                  onChange={(checked) => handleSettingChange('auto_cleanup', checked)}
                  disabled={!settings.notifications}
                />
              </div>
            </div>
          </Card>

          {/* 文件粉碎设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Trash2 className="w-5 h-5 text-red-400" />
                <span>{t('settings.fileShredder', '文件粉碎')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 安全删除 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.secureDelete', '启用安全删除')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.secureDeleteDesc', '使用多次覆写删除文件')}</div>
                </div>
                <Switch
                  checked={settings.secure_delete}
                  onChange={(checked) => handleSettingChange('secure_delete', checked)}
                />
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 删除方法 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.deleteMethod', '删除方法')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.deleteMethodDesc', '选择文件擦除标准')}</div>
                </div>
                <Select
                  value={settings.delete_method}
                  onChange={(value) => handleSettingChange('delete_method', value)}
                  className="w-40"
                  disabled={!settings.secure_delete}
                >
                  <Option value="zero">{t('settings.methodZero', '零填充 (1次)')}</Option>
                  <Option value="random">{t('settings.methodRandom', '随机数据 (3次)')}</Option>
                  <Option value="dod">{t('settings.methodDod', 'DoD 5220.22-M (7次)')}</Option>
                  <Option value="gutmann">{t('settings.methodGutmann', 'Gutmann (35次)')}</Option>
                </Select>
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 删除前确认 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.confirmDelete', '删除前确认')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.confirmDeleteDesc', '粉碎文件前显示确认对话框')}</div>
                </div>
                <Switch
                  checked={settings.confirm_before_delete}
                  onChange={(checked) => handleSettingChange('confirm_before_delete', checked)}
                />
              </div>
            </div>
          </Card>

          {/* 系统日志设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <FileText className="w-5 h-5 text-purple-400" />
                <span>{t('settings.systemLogs', '系统日志')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 自动清理日志 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoCleanLogs', '自动清理日志')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoCleanLogsDesc', '定期清理系统日志')}</div>
                </div>
                <Switch
                  checked={settings.auto_clean_logs}
                  onChange={(checked) => handleSettingChange('auto_clean_logs', checked)}
                />
              </div>

              {settings.auto_clean_logs && (
                <>
                  <Divider className="bg-gray-700 my-3" />
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-white font-medium">{t('settings.logRetention', '日志保留天数')}</div>
                      <div className="text-gray-400 text-sm">{t('settings.logRetentionDesc', '保留最近N天的日志')}</div>
                    </div>
                    <InputNumber
                      min={1}
                      max={365}
                      value={settings.log_retention_days}
                      onChange={(value) => handleSettingChange('log_retention_days', value || 30)}
                      className="w-24"
                    />
                  </div>
                </>
              )}
            </div>
          </Card>

          {/* 内存清理设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Cpu className="w-5 h-5 text-orange-400" />
                <span>{t('settings.memoryCleanup', '内存清理')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 自动清理内存 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoCleanMemory', '自动清理内存')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoCleanMemoryDesc', '定期自动清理内存痕迹')}</div>
                </div>
                <Switch
                  checked={settings.auto_clean_memory}
                  onChange={(checked) => handleSettingChange('auto_clean_memory', checked)}
                />
              </div>

              {settings.auto_clean_memory && (
                <>
                  <Divider className="bg-gray-700 my-3" />
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-white font-medium">{t('settings.cleanInterval', '清理间隔')}</div>
                      <div className="text-gray-400 text-sm">{t('settings.minutes', '分钟')}</div>
                    </div>
                    <InputNumber
                      min={5}
                      max={1440}
                      value={settings.memory_clean_interval}
                      onChange={(value) => handleSettingChange('memory_clean_interval', value || 30)}
                      className="w-24"
                    />
                  </div>
                </>
              )}
            </div>
          </Card>

          {/* 网络清理设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Network className="w-5 h-5 text-green-400" />
                <span>{t('settings.networkCleanup', '网络清理')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 自动清理DNS缓存 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoClearDns', '自动清理DNS缓存')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoClearDnsDesc', '定期清除DNS解析缓存')}</div>
                </div>
                <Switch
                  checked={settings.auto_clear_dns_cache}
                  onChange={(checked) => handleSettingChange('auto_clear_dns_cache', checked)}
                />
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 自动清理连接历史 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoClearHistory', '自动清理连接历史')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoClearHistoryDesc', '定期清除网络连接历史记录')}</div>
                </div>
                <Switch
                  checked={settings.auto_clear_network_history}
                  onChange={(checked) => handleSettingChange('auto_clear_network_history', checked)}
                />
              </div>
            </div>
          </Card>

          {/* 注册表清理设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Database className="w-5 h-5 text-pink-400" />
                <span>{t('settings.registryCleanup', '注册表清理 (Windows)')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 自动清理注册表 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoCleanRegistry', '自动清理注册表')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoCleanRegistryDesc', '定期清理注册表痕迹')}</div>
                </div>
                <Switch
                  checked={settings.auto_clean_registry}
                  onChange={(checked) => handleSettingChange('auto_clean_registry', checked)}
                />
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 清理级别 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.cleanLevel', '清理级别')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.cleanLevelDesc', '选择清理的注册表项')}</div>
                </div>
                <Select
                  value={settings.registry_clean_level}
                  onChange={(value) => handleSettingChange('registry_clean_level', value)}
                  className="w-32"
                  disabled={!settings.auto_clean_registry}
                >
                  <Option value="high">{t('settings.levelHigh', '仅高风险')}</Option>
                  <Option value="medium">{t('settings.levelMedium', '中高风险')}</Option>
                  <Option value="all">{t('settings.levelAll', '全部清理')}</Option>
                </Select>
              </div>
            </div>
          </Card>

          {/* 时间戳修改设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Clock className="w-5 h-5 text-indigo-400" />
                <span>{t('settings.timestampModifier', '时间戳修改')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.randomTimeRange', '随机时间范围')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.randomTimeRangeDesc', '生成随机时间的天数范围')}</div>
                </div>
                <InputNumber
                  min={1}
                  max={3650}
                  value={settings.random_time_range_days}
                  onChange={(value) => handleSettingChange('random_time_range_days', value || 365)}
                  className="w-24"
                  addonAfter={t('settings.days', '天')}
                />
              </div>
            </div>
          </Card>

          {/* 反分析检测设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Eye className="w-5 h-5 text-cyan-400" />
                <span>{t('settings.antiAnalysis', '反分析检测')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 自动检测 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.autoCheck', '自动环境检测')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.autoCheckDesc', '定期检测虚拟机和调试器')}</div>
                </div>
                <Switch
                  checked={settings.auto_anti_analysis_check}
                  onChange={(checked) => handleSettingChange('auto_anti_analysis_check', checked)}
                />
              </div>

              {settings.auto_anti_analysis_check && (
                <>
                  <Divider className="bg-gray-700 my-3" />
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-white font-medium">{t('settings.checkInterval', '检测间隔')}</div>
                      <div className="text-gray-400 text-sm">{t('settings.minutes', '分钟')}</div>
                    </div>
                    <InputNumber
                      min={5}
                      max={1440}
                      value={settings.anti_analysis_check_interval}
                      onChange={(value) => handleSettingChange('anti_analysis_check_interval', value || 60)}
                      className="w-24"
                    />
                  </div>
                </>
              )}

              <Divider className="bg-gray-700 my-3" />

              {/* 威胁警报 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.threatAlert', '威胁警报')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.threatAlertDesc', '检测到威胁时发出警报')}</div>
                </div>
                <Switch
                  checked={settings.alert_on_threat_detected}
                  onChange={(checked) => handleSettingChange('alert_on_threat_detected', checked)}
                />
              </div>
            </div>
          </Card>

          {/* 磁盘加密设置 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <Lock className="w-5 h-5 text-yellow-400" />
                <span>{t('settings.diskEncryption', '磁盘加密')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 加密提醒 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.encryptionReminder', '加密提醒')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.encryptionReminderDesc', '未加密磁盘时显示提醒')}</div>
                </div>
                <Switch
                  checked={settings.remind_disk_encryption}
                  onChange={(checked) => handleSettingChange('remind_disk_encryption', checked)}
                />
              </div>
            </div>
          </Card>

          {/* 权限管理 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <ShieldCheck className="w-5 h-5 text-green-400" />
                <span>{t('settings.permissionManagement', '权限管理')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 权限状态 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.permissionStatus', '权限状态')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.permissionStatusDesc', '应用当前的权限授权状态')}</div>
                </div>
                {permissionStatus.checking ? (
                  <div className="flex items-center gap-2 text-gray-400">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span className="text-sm">{t('settings.checking', '检查中...')}</span>
                  </div>
                ) : permissionStatus.initialized ? (
                  <div className="flex items-center gap-2 text-green-400">
                    <ShieldCheck className="w-5 h-5" />
                    <span className="text-sm font-medium">{t('settings.authorized', '已授权')}</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 text-amber-400">
                    <ShieldX className="w-5 h-5" />
                    <span className="text-sm font-medium">{t('settings.notAuthorized', '未授权')}</span>
                  </div>
                )}
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 管理员权限 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.adminPermission', '管理员权限')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.adminPermissionDesc', '是否具有系统管理员权限')}</div>
                </div>
                {permissionStatus.checking ? (
                  <div className="flex items-center gap-2 text-gray-400">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span className="text-sm">{t('settings.checking', '检查中...')}</span>
                  </div>
                ) : permissionStatus.isAdmin ? (
                  <div className="flex items-center gap-2 text-green-400">
                    <ShieldCheck className="w-5 h-5" />
                    <span className="text-sm font-medium">{t('settings.yes', '是')}</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 text-amber-400">
                    <ShieldX className="w-5 h-5" />
                    <span className="text-sm font-medium">{t('settings.no', '否')}</span>
                  </div>
                )}
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 操作按钮 */}
              <div className="flex items-center gap-3">
                <Button
                  icon={<RefreshCw className="w-4 h-4" />}
                  onClick={checkPermissionStatus}
                  disabled={permissionStatus.checking}
                  className="flex items-center gap-2"
                >
                  {t('settings.refreshStatus', '刷新状态')}
                </Button>
                {(!permissionStatus.initialized || !permissionStatus.isAdmin) && !permissionStatus.checking && (
                  <Button
                    type="primary"
                    icon={<Lock className="w-4 h-4" />}
                    onClick={handleRequestPermission}
                    loading={permissionStatus.requesting}
                    className="flex items-center gap-2"
                  >
                    {permissionStatus.requesting ? t('settings.authorizing', '授权中...') : t('settings.authorize', '立即授权')}
                  </Button>
                )}
              </div>

              {/* 权限不足提示 */}
              {!permissionStatus.checking && (!permissionStatus.initialized || !permissionStatus.isAdmin) && (
                <div className="mt-3 p-4 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                  <div className="flex items-start gap-3">
                    <ShieldX className="w-5 h-5 text-amber-400 flex-shrink-0 mt-0.5" />
                    <div className="flex-1">
                      <p className="text-amber-400 font-medium mb-2">{t('settings.insufficientPermissions', '权限不足')}</p>
                      <p className="text-amber-300/80 text-sm mb-3">
                        {t('settings.insufficientPermissionsDesc', '当前应用缺少必要的管理员权限，以下功能可能无法正常使用：')}
                      </p>
                      <ul className="text-amber-300/70 text-sm space-y-1 mb-3">
                        <li className="flex items-center gap-2">
                          <span className="w-1 h-1 bg-amber-400 rounded-full"></span>
                          {t('settings.permissionItem1', '清理系统日志 (/var/log, /Library/Logs)')}
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="w-1 h-1 bg-amber-400 rounded-full"></span>
                          {t('settings.permissionItem2', '清理系统临时文件')}
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="w-1 h-1 bg-amber-400 rounded-full"></span>
                          {t('settings.permissionItem3', '刷新 DNS 缓存')}
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="w-1 h-1 bg-amber-400 rounded-full"></span>
                          {t('settings.permissionItem4', '清理统一日志数据库')}
                        </li>
                      </ul>
                      <p className="text-amber-300/80 text-sm">
                        {t('settings.authorizeTip', '点击上方"立即授权"按钮，在弹出的系统对话框中输入管理员密码即可完成授权。')}
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {/* 权限正常提示 */}
              {!permissionStatus.checking && permissionStatus.initialized && permissionStatus.isAdmin && (
                <div className="mt-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                  <div className="flex items-center gap-3">
                    <ShieldCheck className="w-5 h-5 text-green-400" />
                    <p className="text-green-400 text-sm">
                      {t('settings.allPermissionsReady', '所有权限已就绪，应用可以正常使用全部功能。')}
                    </p>
                  </div>
                </div>
              )}
            </div>
          </Card>

          {/* 数据存储信息 */}
          <Card
            className="bg-gray-800/50 border-gray-700"
            title={
              <div className="flex items-center gap-2 text-white">
                <HardDrive className="w-5 h-5 text-blue-400" />
                <span>{t('settings.dataStorage', '数据存储')}</span>
              </div>
            }
          >
            <div className="space-y-4">
              {/* 加密状态 */}
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-white font-medium">{t('settings.storageEncryption', '存储加密')}</div>
                  <div className="text-gray-400 text-sm">{t('settings.storageEncryptionDesc', '设置数据使用 SQLCipher 加密')}</div>
                </div>
                <div className="flex items-center gap-2 text-green-400">
                  <CheckCircle2 className="w-5 h-5" />
                  <span className="text-sm font-medium">{t('settings.encrypted', '已加密')}</span>
                </div>
              </div>

              <Divider className="bg-gray-700 my-3" />

              {/* 数据库信息 */}
              {dbInfo && (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-white font-medium">{t('settings.databaseSize', '数据库大小')}</div>
                      <div className="text-gray-400 text-sm">{t('settings.databaseSizeDesc', '当前设置文件大小')}</div>
                    </div>
                    <span className="text-sm text-slate-300 font-mono">
                      {formatBytes(dbInfo.size_bytes)}
                    </span>
                  </div>

                  <Divider className="bg-gray-700 my-3" />

                  <div>
                    <div className="text-white font-medium mb-2">{t('settings.storagePath', '存储路径')}</div>
                    <div className="text-xs text-slate-500 font-mono bg-slate-900/50 p-2 rounded break-all">
                      {dbInfo.path}
                    </div>
                  </div>
                </>
              )}
            </div>
          </Card>
        </div>

        {/* 关于信息 - 跨越整个宽度 */}
        <Card
          className="bg-gray-800/50 border-gray-700 mt-6"
          title={
            <div className="flex items-center gap-2 text-white">
              <Info className="w-5 h-5 text-cyan-400" />
              <span>{t('settings.about', '关于')}</span>
            </div>
          }
        >
          <div className="space-y-6">
            {/* Top Row: All info using grid for alignment */}
            <div className="grid grid-cols-4 gap-4">
              <div>
                <div className="text-gray-400 text-sm mb-1">{t('settings.appName', '应用名称')}</div>
                <div className="text-white font-semibold text-lg">{t('app.name', '无痕')}</div>
              </div>
              <div>
                <div className="text-gray-400 text-sm mb-1">{t('settings.version', '版本')}</div>
                <div className="text-white font-medium text-lg">v1.0.0</div>
              </div>
              <div>
                <div className="text-gray-400 text-sm mb-1">{t('settings.lastUpdate', '最后更新')}</div>
                <div className="text-white font-medium text-lg">2025-01-15</div>
              </div>
              <div>
                <div className="text-gray-400 text-sm mb-1">{t('settings.platform', '运行平台')}</div>
                <div className="text-white font-medium text-lg">
                  {systemInfo ? `${systemInfo.os} ${systemInfo.version}` : t('common.loading', '加载中...')}
                </div>
              </div>
            </div>

            {/* System Stats Cards */}
            <div className="grid grid-cols-3 gap-4">
              {/* System Info Card */}
              <div className="bg-slate-700/50 rounded-lg p-4">
                <div className="flex items-center gap-2 text-slate-400 text-sm mb-2">
                  <Monitor className="w-4 h-4 flex-shrink-0" />
                  <span className="whitespace-nowrap">{t('settings.systemInfo', '系统详情')}</span>
                </div>
                <div className="text-white font-medium text-lg">
                  {systemInfo ? `${systemInfo.os} ${systemInfo.version}` : t('common.loading', '加载中...')}
                </div>
              </div>

              {/* CPU Card */}
              <div className="bg-slate-700/50 rounded-lg p-4">
                <div className="flex items-center gap-2 text-slate-400 text-sm mb-2">
                  <Cpu className="w-4 h-4 flex-shrink-0" />
                  <span className="whitespace-nowrap">{t('settings.cpuCores', 'CPU 核心')}</span>
                </div>
                <div className="text-white font-medium text-lg">
                  {systemInfo ? `${systemInfo.cpu_count} ${t('settings.cores', '核心')}` : t('common.loading', '加载中...')}
                </div>
              </div>

              {/* Memory Card */}
              <div className="bg-slate-700/50 rounded-lg p-4">
                <div className="flex items-center gap-2 text-slate-400 text-sm mb-2">
                  <HardDrive className="w-4 h-4 flex-shrink-0" />
                  <span className="whitespace-nowrap">{t('settings.totalMemory', '总内存')}</span>
                </div>
                <div className="text-white font-medium text-lg">
                  {systemInfo ? `${(systemInfo.total_memory / 1024).toFixed(1)} GB` : t('common.loading', '加载中...')}
                </div>
              </div>
            </div>

            {/* Footer: Copyright */}
            <div className="text-gray-400 text-sm pt-2 text-center">
              <p>© 2025 Anti-Forensics Tool. {t('settings.copyright', '隐私保护工具。')}</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};

export default Settings;
