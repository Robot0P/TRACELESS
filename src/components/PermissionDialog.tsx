import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Modal, Button, Steps } from 'antd';
import { useTranslation } from 'react-i18next';
import { ShieldCheck, Lock, CheckCircle2, AlertTriangle, Loader2, HardDrive, ExternalLink, Monitor, Terminal } from 'lucide-react';

interface PermissionStatus {
  is_admin: boolean;
  has_full_disk_access: boolean;
  has_authorization: boolean;
  has_system_privileges: boolean;
  platform: string;
}

interface PermissionDialogProps {
  onComplete: () => void;
}

const PermissionDialog: React.FC<PermissionDialogProps> = ({ onComplete }) => {
  const { i18n } = useTranslation();
  const [visible, setVisible] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentStep, setCurrentStep] = useState(0);
  const [adminAuthorized, setAdminAuthorized] = useState(false);
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus | null>(null);

  const isEnglish = i18n.language === 'en-US';

  useEffect(() => {
    const checkPermission = async () => {
      try {
        // Get platform-specific permission status
        const status = await invoke<PermissionStatus>('get_permission_status');
        setPermissionStatus(status);

        const initialized = await invoke<boolean>('check_permission_initialized');
        if (!initialized) {
          setVisible(true);
        } else {
          onComplete();
        }
      } catch {
        setVisible(true);
      }
    };

    checkPermission();
  }, [onComplete]);

  const handleInitializeAdmin = async () => {
    setLoading(true);
    setError(null);

    try {
      await invoke<string>('initialize_permissions');
      setAdminAuthorized(true);
      // Refresh permission status
      const status = await invoke<PermissionStatus>('get_permission_status');
      setPermissionStatus(status);
      // Auto proceed to next step on macOS, complete on other platforms
      if (status.platform === 'macOS') {
        setCurrentStep(1);
      } else {
        handleComplete();
      }
    } catch (err) {
      const errorStr = typeof err === 'string' ? err : String(err);
      // Parse error and provide user-friendly message
      if (errorStr.includes('cancelled') || errorStr.includes('User cancelled')) {
        setError(isEnglish ? 'Authorization was cancelled. Click to try again.' : '授权已取消。点击重试。');
      } else if (errorStr.includes('PERMISSION_DENIED')) {
        setError(isEnglish ? 'Please confirm the UAC prompt to grant administrator privileges.' : '请确认 UAC 提示以授予管理员权限。');
      } else {
        setError(isEnglish ? 'Permission initialization failed' : '权限初始化失败');
      }
    } finally {
      setLoading(false);
    }
  };

  const handleOpenFullDiskAccess = async () => {
    try {
      await invoke('open_full_disk_access_settings');
    } catch {
      try {
        await invoke('open_privacy_settings');
      } catch {
        // Silently fail
      }
    }
  };

  const handleComplete = () => {
    setVisible(false);
    onComplete();
  };

  const handleSkip = () => {
    setVisible(false);
    onComplete();
  };

  if (!visible) {
    return null;
  }

  // Platform-specific content
  const getPlatformIcon = () => {
    switch (permissionStatus?.platform) {
      case 'Windows':
        return <Monitor className="w-16 h-16 text-blue-400" />;
      case 'Linux':
        return <Terminal className="w-16 h-16 text-orange-400" />;
      default:
        return <Lock className="w-16 h-16 text-accent" />;
    }
  };

  const getPlatformTitle = () => {
    if (permissionStatus?.platform === 'Windows') {
      return isEnglish ? 'Administrator Privileges Required' : '需要管理员权限';
    } else if (permissionStatus?.platform === 'Linux') {
      return isEnglish ? 'Root Privileges Required' : '需要 Root 权限';
    }
    return isEnglish ? 'Authorization Required' : '需要授权';
  };

  const getPlatformDescription = () => {
    if (permissionStatus?.platform === 'Windows') {
      return isEnglish
        ? 'This application requires Administrator privileges for system-level cleanup operations. For advanced operations, SYSTEM or TrustedInstaller privileges may be used.'
        : '此应用需要管理员权限来执行系统级清理操作。对于高级操作，可能会使用 SYSTEM 或 TrustedInstaller 权限。';
    } else if (permissionStatus?.platform === 'Linux') {
      return isEnglish
        ? 'This application requires root privileges for system-level cleanup operations. Please run with sudo if not already elevated.'
        : '此应用需要 root 权限来执行系统级清理操作。如果尚未提权，请使用 sudo 运行。';
    }
    return isEnglish
      ? 'This application uses macOS Authorization Services to request administrator privileges for system cleanup operations.'
      : '此应用使用 macOS 授权服务请求管理员权限，用于系统清理操作。';
  };

  const getPlatformPermissions = () => {
    const common = [
      isEnglish ? 'System log cleanup' : '系统日志清理',
      isEnglish ? 'DNS cache flushing' : '清除 DNS 缓存',
      isEnglish ? 'Memory cleanup' : '内存清理',
    ];

    if (permissionStatus?.platform === 'Windows') {
      return [
        ...common,
        isEnglish ? 'Event log management' : '事件日志管理',
        isEnglish ? 'Prefetch cleanup' : 'Prefetch 清理',
        isEnglish ? 'Registry cleanup' : '注册表清理',
      ];
    } else if (permissionStatus?.platform === 'Linux') {
      return [
        ...common,
        isEnglish ? 'Journal log cleanup' : 'Journal 日志清理',
        isEnglish ? 'Package cache cleanup' : '软件包缓存清理',
      ];
    }
    return [
      ...common,
      isEnglish ? 'Unified log cleanup' : '统一日志清理',
    ];
  };

  const getButtonText = () => {
    if (loading) {
      return isEnglish ? 'Waiting for authorization...' : '等待授权...';
    }
    if (permissionStatus?.platform === 'Windows') {
      return permissionStatus.is_admin
        ? (isEnglish ? 'Confirm Privileges' : '确认权限')
        : (isEnglish ? 'Restart as Administrator' : '以管理员身份重启');
    } else if (permissionStatus?.platform === 'Linux') {
      return permissionStatus.is_admin
        ? (isEnglish ? 'Confirm Root Access' : '确认 Root 权限')
        : (isEnglish ? 'Run with sudo' : '使用 sudo 运行');
    }
    return isEnglish ? 'Authorize Now' : '立即授权';
  };

  // Determine if we need step 2 (Full Disk Access - macOS only)
  const showStep2 = permissionStatus?.platform === 'macOS';

  return (
    <Modal
      open={visible}
      footer={null}
      closable={false}
      centered
      width={560}
      className="permission-modal"
      maskClosable={false}
    >
      <div className="py-6">
        {/* Step indicator */}
        {showStep2 && (
          <div className="mb-6 px-4">
            <Steps
              current={currentStep}
              size="small"
              items={[
                {
                  title: isEnglish ? 'Admin Permission' : '管理员权限',
                  status: adminAuthorized ? 'finish' : currentStep === 0 ? 'process' : 'wait',
                },
                {
                  title: isEnglish ? 'Full Disk Access' : '完全磁盘访问',
                  status: currentStep === 1 ? 'process' : 'wait',
                },
              ]}
            />
          </div>
        )}

        {/* Step 1: Admin Permission (All Platforms) */}
        {currentStep === 0 && (
          <div className="text-center">
            {/* Platform Badge */}
            <div className="mb-4">
              <span className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-medium ${
                permissionStatus?.platform === 'Windows' ? 'bg-blue-500/20 text-blue-400' :
                permissionStatus?.platform === 'Linux' ? 'bg-orange-500/20 text-orange-400' :
                'bg-accent/20 text-accent'
              }`}>
                {permissionStatus?.platform || 'Unknown'}
              </span>
            </div>

            {/* Icon */}
            <div className="relative inline-block mb-6">
              <div className={`absolute inset-0 rounded-full blur-2xl transition-colors duration-300 ${
                error ? 'bg-red-500/20' :
                permissionStatus?.platform === 'Windows' ? 'bg-blue-500/20' :
                permissionStatus?.platform === 'Linux' ? 'bg-orange-500/20' :
                'bg-accent/20'
              }`} />
              <div className={`relative p-6 rounded-full border transition-all duration-300 ${
                error
                  ? 'bg-gradient-to-br from-red-500/20 to-red-500/5 border-red-500/30'
                  : permissionStatus?.platform === 'Windows'
                  ? 'bg-gradient-to-br from-blue-500/20 to-blue-500/5 border-blue-500/30'
                  : permissionStatus?.platform === 'Linux'
                  ? 'bg-gradient-to-br from-orange-500/20 to-orange-500/5 border-orange-500/30'
                  : 'bg-gradient-to-br from-accent/20 to-accent/5 border-accent/30'
              }`}>
                {loading ? (
                  <Loader2 className="w-16 h-16 text-accent animate-spin" />
                ) : error ? (
                  <AlertTriangle className="w-16 h-16 text-red-400" />
                ) : (
                  getPlatformIcon()
                )}
              </div>
            </div>

            {/* Title and description */}
            {error ? (
              <>
                <h2 className="text-2xl font-bold text-white mb-2">
                  {isEnglish ? 'Authorization Failed' : '授权失败'}
                </h2>
                <p className="text-red-400 mb-6">{error}</p>
              </>
            ) : loading ? (
              <>
                <h2 className="text-2xl font-bold text-white mb-2">
                  {isEnglish ? 'Authorizing...' : '正在授权...'}
                </h2>
                <p className="text-slate-400 mb-6">
                  {permissionStatus?.platform === 'Windows'
                    ? (isEnglish ? 'Please confirm the UAC prompt' : '请确认 UAC 提示')
                    : permissionStatus?.platform === 'Linux'
                    ? (isEnglish ? 'Please enter your password' : '请输入密码')
                    : (isEnglish ? 'Please enter your administrator password' : '请输入管理员密码')}
                </p>
              </>
            ) : (
              <>
                <h2 className="text-2xl font-bold text-white mb-2">{getPlatformTitle()}</h2>
                <p className="text-slate-400 mb-6 px-4">
                  {getPlatformDescription()}
                </p>
              </>
            )}

            {/* Permission list */}
            {!loading && (
              <div className="text-left bg-slate-800/50 rounded-xl p-4 mb-6 border border-white/5 mx-4">
                <div className="text-sm text-slate-300 font-medium mb-3">
                  {isEnglish ? 'Required for:' : '用于以下操作:'}
                </div>
                <ul className="space-y-2 text-sm text-slate-400">
                  {getPlatformPermissions().map((perm, index) => (
                    <li key={index} className="flex items-center gap-2">
                      <ShieldCheck className="w-4 h-4 text-accent" />
                      {perm}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* Current status indicator */}
            {permissionStatus && !loading && (
              <div className="mb-4 px-4">
                <div className={`inline-flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${
                  permissionStatus.is_admin
                    ? 'bg-green-500/10 text-green-400 border border-green-500/20'
                    : 'bg-yellow-500/10 text-yellow-400 border border-yellow-500/20'
                }`}>
                  {permissionStatus.is_admin ? (
                    <>
                      <CheckCircle2 className="w-4 h-4" />
                      {isEnglish ? 'Running with elevated privileges' : '已以提升权限运行'}
                    </>
                  ) : (
                    <>
                      <AlertTriangle className="w-4 h-4" />
                      {isEnglish ? 'Elevation required' : '需要提升权限'}
                    </>
                  )}
                </div>
              </div>
            )}

            {/* Buttons */}
            <div className="flex gap-3 justify-center">
              <Button
                size="large"
                onClick={handleSkip}
                disabled={loading}
                className="bg-slate-700 hover:bg-slate-600 text-white border-none min-w-[100px]"
              >
                {isEnglish ? 'Later' : '稍后'}
              </Button>
              <Button
                type="primary"
                size="large"
                onClick={handleInitializeAdmin}
                loading={loading}
                className="bg-accent hover:bg-accent/80 border-none min-w-[140px]"
                icon={!loading && <Lock size={16} />}
              >
                {getButtonText()}
              </Button>
            </div>

            {error && (
              <div className="mt-4 text-xs text-slate-500">
                {isEnglish ? 'Click to retry authorization' : '点击重试授权'}
              </div>
            )}
          </div>
        )}

        {/* Step 2: Full Disk Access (macOS only) */}
        {currentStep === 1 && showStep2 && (
          <div className="text-center">
            {/* Icon */}
            <div className="relative inline-block mb-6">
              <div className="absolute inset-0 rounded-full blur-2xl bg-blue-500/20" />
              <div className="relative p-6 rounded-full border bg-gradient-to-br from-blue-500/20 to-blue-500/5 border-blue-500/30">
                <HardDrive className="w-16 h-16 text-blue-400" />
              </div>
            </div>

            {/* Title and description */}
            <h2 className="text-2xl font-bold text-white mb-2">
              {isEnglish ? 'Full Disk Access' : '完全磁盘访问权限'}
            </h2>
            <p className="text-slate-400 mb-6">
              {isEnglish
                ? 'Full Disk Access is required to clean browser data and access protected files.'
                : '完全磁盘访问权限用于清理浏览器数据和访问受保护的文件。'}
              <br />
              <span className="text-blue-400">
                {isEnglish ? 'This is optional but recommended.' : '此权限是可选的，但建议开启。'}
              </span>
            </p>

            {/* Instructions */}
            <div className="text-left bg-slate-800/50 rounded-xl p-4 mb-6 border border-white/5 mx-4">
              <div className="text-sm text-slate-300 font-medium mb-3">
                {isEnglish ? 'Follow these steps:' : '请按照以下步骤操作:'}
              </div>
              <ol className="space-y-3 text-sm text-slate-400">
                <li className="flex items-start gap-3">
                  <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">1</span>
                  <span>{isEnglish ? 'Click "Open Settings" below' : '点击下方"打开设置"'}</span>
                </li>
                <li className="flex items-start gap-3">
                  <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">2</span>
                  <span>{isEnglish ? 'Click the lock icon to make changes' : '点击锁图标进行更改'}</span>
                </li>
                <li className="flex items-start gap-3">
                  <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">3</span>
                  <span>{isEnglish ? 'Add Traceless to the list' : '将 Traceless 添加到列表'}</span>
                </li>
                <li className="flex items-start gap-3">
                  <span className="flex-shrink-0 w-5 h-5 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">4</span>
                  <span>{isEnglish ? 'Restart the app' : '重启应用'}</span>
                </li>
              </ol>
            </div>

            {/* Notice */}
            <div className="text-left bg-amber-500/10 rounded-xl p-4 mb-6 border border-amber-500/20 mx-4">
              <div className="flex items-start gap-2">
                <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
                <div className="text-sm text-amber-300/80">
                  <p className="font-medium text-amber-400 mb-1">
                    {isEnglish ? 'Important' : '重要提示'}
                  </p>
                  <p>
                    {isEnglish
                      ? 'Without Full Disk Access, some browser cleanup features may not work.'
                      : '如果没有完全磁盘访问权限，某些浏览器清理功能可能无法正常工作。'}
                  </p>
                </div>
              </div>
            </div>

            {/* Buttons */}
            <div className="flex gap-3 justify-center">
              <Button
                size="large"
                onClick={handleComplete}
                className="bg-slate-700 hover:bg-slate-600 text-white border-none min-w-[100px]"
              >
                {isEnglish ? 'Setup Later' : '稍后设置'}
              </Button>
              <Button
                type="primary"
                size="large"
                onClick={handleOpenFullDiskAccess}
                className="bg-blue-600 hover:bg-blue-500 border-none min-w-[160px]"
                icon={<ExternalLink size={16} />}
              >
                {isEnglish ? 'Open Settings' : '打开设置'}
              </Button>
            </div>

            {/* Complete prompt */}
            <div className="mt-4">
              <Button
                type="link"
                onClick={handleComplete}
                className="text-green-400 hover:text-green-300"
              >
                <CheckCircle2 size={14} className="mr-1" />
                {isEnglish ? 'I\'ve completed the setup' : '我已完成设置'}
              </Button>
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
};

export default PermissionDialog;
