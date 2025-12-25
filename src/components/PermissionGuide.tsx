import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Shield, HardDrive, Key, CheckCircle2, AlertCircle, ExternalLink, Loader2 } from 'lucide-react';

interface PermissionStatus {
  is_admin: boolean;
  has_full_disk_access: boolean;
  platform: string;
}

interface PermissionGuideProps {
  onComplete: () => void;
}

const PermissionGuide: React.FC<PermissionGuideProps> = ({ onComplete }) => {
  const { i18n } = useTranslation();
  const [status, setStatus] = useState<PermissionStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [requestingAdmin, setRequestingAdmin] = useState(false);
  const [adminGranted, setAdminGranted] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isEnglish = i18n.language === 'en-US';

  useEffect(() => {
    checkPermissions();
  }, []);

  const checkPermissions = async () => {
    try {
      setLoading(true);
      const permStatus = await invoke<PermissionStatus>('get_permission_status');
      setStatus(permStatus);

      // Check if already initialized
      const initialized = await invoke<boolean>('check_permission_initialized');
      if (initialized) {
        setAdminGranted(true);
      }
    } catch (err) {
      console.error('Failed to check permissions:', err);
    } finally {
      setLoading(false);
    }
  };

  const requestAdminPermission = async () => {
    try {
      setRequestingAdmin(true);
      setError(null);
      await invoke<string>('initialize_permissions');
      setAdminGranted(true);
      await checkPermissions();
    } catch (err) {
      setError(String(err));
    } finally {
      setRequestingAdmin(false);
    }
  };

  const openFullDiskAccess = async () => {
    try {
      await invoke('open_full_disk_access_settings');
    } catch (err) {
      console.error('Failed to open settings:', err);
    }
  };

  const handleContinue = () => {
    onComplete();
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-slate-950/95 backdrop-blur-xl flex items-center justify-center z-50">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-8 h-8 text-accent animate-spin" />
          <p className="text-slate-400">{isEnglish ? 'Checking permissions...' : '检查权限中...'}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-slate-950/95 backdrop-blur-xl flex items-center justify-center z-50 p-6">
      <div className="bg-slate-900/90 border border-white/10 rounded-2xl p-8 max-w-lg w-full shadow-2xl">
        {/* Header */}
        <div className="flex items-center gap-4 mb-6">
          <div className="w-14 h-14 rounded-xl bg-accent/20 flex items-center justify-center">
            <Shield className="w-7 h-7 text-accent" />
          </div>
          <div>
            <h2 className="text-xl font-bold text-white">
              {isEnglish ? 'Permission Setup' : '权限设置'}
            </h2>
            <p className="text-sm text-slate-400">
              {isEnglish ? 'Grant permissions for full functionality' : '授予权限以获得完整功能'}
            </p>
          </div>
        </div>

        {/* Permission Items */}
        <div className="space-y-4 mb-6">
          {/* Admin Permission */}
          <div className={`p-4 rounded-xl border transition-all ${
            adminGranted
              ? 'bg-green-500/10 border-green-500/30'
              : 'bg-slate-800/50 border-white/10'
          }`}>
            <div className="flex items-start gap-3">
              <div className={`w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 ${
                adminGranted ? 'bg-green-500/20' : 'bg-accent/20'
              }`}>
                <Key className={`w-5 h-5 ${adminGranted ? 'text-green-400' : 'text-accent'}`} />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <h3 className="font-medium text-white">
                    {isEnglish ? 'Administrator Privileges' : '管理员权限'}
                  </h3>
                  {adminGranted ? (
                    <CheckCircle2 className="w-5 h-5 text-green-400" />
                  ) : (
                    <button
                      onClick={requestAdminPermission}
                      disabled={requestingAdmin}
                      className="px-3 py-1 text-xs font-medium bg-accent hover:bg-accent/80 text-white rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1"
                    >
                      {requestingAdmin && <Loader2 className="w-3 h-3 animate-spin" />}
                      {isEnglish ? 'Grant' : '授权'}
                    </button>
                  )}
                </div>
                <p className="text-sm text-slate-400 mt-1">
                  {isEnglish
                    ? 'Required for system cleanup, log clearing, and memory management'
                    : '系统清理、日志清除和内存管理所需'}
                </p>
                {error && (
                  <p className="text-xs text-red-400 mt-2 flex items-center gap-1">
                    <AlertCircle className="w-3 h-3" />
                    {error}
                  </p>
                )}
              </div>
            </div>
          </div>

          {/* Full Disk Access */}
          {status?.platform === 'macOS' && (
            <div className={`p-4 rounded-xl border transition-all ${
              status?.has_full_disk_access
                ? 'bg-green-500/10 border-green-500/30'
                : 'bg-yellow-500/10 border-yellow-500/30'
            }`}>
              <div className="flex items-start gap-3">
                <div className={`w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 ${
                  status?.has_full_disk_access ? 'bg-green-500/20' : 'bg-yellow-500/20'
                }`}>
                  <HardDrive className={`w-5 h-5 ${status?.has_full_disk_access ? 'text-green-400' : 'text-yellow-400'}`} />
                </div>
                <div className="flex-1">
                  <div className="flex items-center justify-between">
                    <h3 className="font-medium text-white">
                      {isEnglish ? 'Full Disk Access' : '完全磁盘访问权限'}
                    </h3>
                    {status?.has_full_disk_access ? (
                      <CheckCircle2 className="w-5 h-5 text-green-400" />
                    ) : (
                      <button
                        onClick={openFullDiskAccess}
                        className="px-3 py-1 text-xs font-medium bg-yellow-500/20 hover:bg-yellow-500/30 text-yellow-400 rounded-lg transition-colors flex items-center gap-1"
                      >
                        <ExternalLink className="w-3 h-3" />
                        {isEnglish ? 'Open Settings' : '打开设置'}
                      </button>
                    )}
                  </div>
                  <p className="text-sm text-slate-400 mt-1">
                    {isEnglish
                      ? 'Required for accessing protected files and browser data'
                      : '访问受保护文件和浏览器数据所需'}
                  </p>
                  {!status?.has_full_disk_access && (
                    <div className="mt-3 p-3 bg-slate-800/50 rounded-lg">
                      <p className="text-xs text-slate-300 mb-2">
                        {isEnglish ? 'Manual steps:' : '手动步骤:'}
                      </p>
                      <ol className="text-xs text-slate-400 space-y-1 list-decimal list-inside">
                        <li>{isEnglish ? 'Click "Open Settings" above' : '点击上方"打开设置"'}</li>
                        <li>{isEnglish ? 'Click the lock icon to make changes' : '点击锁图标进行更改'}</li>
                        <li>{isEnglish ? 'Add Traceless to the list' : '将 Traceless 添加到列表'}</li>
                        <li>{isEnglish ? 'Restart the app' : '重启应用'}</li>
                      </ol>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Info Box */}
        <div className="p-4 bg-slate-800/30 rounded-xl border border-white/5 mb-6">
          <p className="text-xs text-slate-400 leading-relaxed">
            {isEnglish
              ? 'These permissions are required for the app to perform system cleanup operations effectively. Your data stays local and is never transmitted.'
              : '这些权限是应用有效执行系统清理操作所必需的。您的数据保留在本地，绝不会传输。'}
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex gap-3">
          <button
            onClick={handleContinue}
            className={`flex-1 py-3 px-4 rounded-xl font-medium transition-all ${
              adminGranted
                ? 'bg-accent hover:bg-accent/80 text-white'
                : 'bg-slate-700 hover:bg-slate-600 text-slate-300'
            }`}
          >
            {adminGranted
              ? (isEnglish ? 'Continue' : '继续')
              : (isEnglish ? 'Skip for Now' : '暂时跳过')}
          </button>
          {!adminGranted && (
            <button
              onClick={checkPermissions}
              className="py-3 px-4 rounded-xl font-medium bg-slate-800 hover:bg-slate-700 text-slate-300 transition-all"
            >
              {isEnglish ? 'Refresh' : '刷新'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default PermissionGuide;
