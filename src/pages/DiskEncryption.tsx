import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Shield,
  Lock,
  Unlock,
  HardDrive,
  CheckCircle2,
  AlertTriangle,
  ArrowLeft,
  Key,
  AlertCircle,
  Loader2,
  RefreshCw,
  Database,
  Monitor,
} from 'lucide-react';
import { Button, Modal, Alert, Progress } from 'antd';

interface DiskInfo {
  name: string;
  path: string;
  encrypted: boolean;
  encryption_type: string | null;
  size: string;
  size_bytes: number;
  used: string;
  used_bytes: number;
  available: string;
  available_bytes: number;
  usage_percent: number;
  file_system: string;
  mount_point: string;
  encryption_progress: number | null;
  is_system_disk: boolean;
}

interface EncryptionStatus {
  enabled: boolean;
  encryption_method: string;
  disks: DiskInfo[];
  platform: string;
  supported: boolean;
  recovery_key_exists: boolean;
  encryption_in_progress: boolean;
}

const DiskEncryption: React.FC = () => {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [status, setStatus] = useState<EncryptionStatus | null>(null);
  const [processing, setProcessing] = useState(false);
  const [showWarningModal, setShowWarningModal] = useState(false);
  const [selectedDisk, setSelectedDisk] = useState<string | null>(null);
  const [operationType, setOperationType] = useState<'enable' | 'disable'>('enable');
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  // 根据平台获取推荐内容
  const getRecommendations = (): string[] => {
    if (!status) return [];
    const platform = status.platform.toLowerCase();
    if (platform.includes('macos') || platform.includes('darwin')) {
      return [
        t('diskEncryption.recommendations.macos.filevault'),
        t('diskEncryption.recommendations.macos.strongPassword'),
        t('diskEncryption.recommendations.macos.recoveryKey'),
        t('diskEncryption.recommendations.macos.findMyMac'),
        t('diskEncryption.recommendations.macos.firmwarePassword'),
      ];
    } else if (platform.includes('windows')) {
      return [
        t('diskEncryption.recommendations.windows.bitlocker'),
        t('diskEncryption.recommendations.windows.tpm'),
        t('diskEncryption.recommendations.windows.recoveryKey'),
        t('diskEncryption.recommendations.windows.checkStatus'),
        t('diskEncryption.recommendations.windows.encryptAll'),
      ];
    } else {
      return [
        t('diskEncryption.recommendations.linux.luks'),
        t('diskEncryption.recommendations.linux.installEncrypt'),
        t('diskEncryption.recommendations.linux.strongPassphrase'),
        t('diskEncryption.recommendations.linux.backupHeader'),
        t('diskEncryption.recommendations.linux.autoUnlock'),
      ];
    }
  };

  // 加载加密状态
  useEffect(() => {
    loadEncryptionStatus();
  }, []);

  const loadEncryptionStatus = async () => {
    setLoading(true);
    try {
      const result = await invoke<EncryptionStatus>('check_disk_encryption');
      setStatus(result);
    } catch {
      // Silently fail
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      const result = await invoke<EncryptionStatus>('check_disk_encryption');
      setStatus(result);
    } catch {
      // Silently fail
    } finally {
      setTimeout(() => setRefreshing(false), 500);
    }
  };

  const handleEnableEncryption = (diskPath: string) => {
    setSelectedDisk(diskPath);
    setOperationType('enable');
    setShowWarningModal(true);
  };

  const handleDisableEncryption = (diskPath: string) => {
    setSelectedDisk(diskPath);
    setOperationType('disable');
    setShowWarningModal(true);
  };

  const confirmOperation = async () => {
    if (!selectedDisk) return;

    setShowWarningModal(false);
    setProcessing(true);

    try {
      let result: string;
      if (operationType === 'enable') {
        result = await invoke<string>('enable_disk_encryption', {
          diskPath: selectedDisk,
        });
      } else {
        result = await invoke<string>('disable_disk_encryption', {
          diskPath: selectedDisk,
        });
      }

      Modal.success({
        title: t('diskEncryption.operationComplete'),
        content: result,
        onOk: () => {
          loadEncryptionStatus();
        },
      });
    } catch (error) {
      Modal.error({
        title: t('diskEncryption.operationFailed'),
        content: String(error),
      });
    } finally {
      setProcessing(false);
      setSelectedDisk(null);
    }
  };

  // 获取加密磁盘数量
  const encryptedCount = status?.disks.filter(d => d.encrypted).length || 0;
  const totalDisks = status?.disks.length || 0;

  return (
    <ProFeatureGate
      feature="disk_encryption"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-accent/5 rounded-full blur-3xl animate-pulse" />
      </div>

      <div className="flex-1 flex flex-col p-6 relative z-10 max-w-7xl mx-auto w-full min-h-0 overflow-hidden">
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
                <Lock className="w-7 h-7 text-accent" />
                {t('diskEncryption.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('diskEncryption.subtitle')}
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
            {status && (
              <div
                className={`flex items-center gap-2 px-4 py-2 rounded-lg border ${status.enabled
                  ? 'bg-green-500/10 border-green-500/30'
                  : 'bg-amber-500/10 border-amber-500/30'
                  }`}
              >
                {status.enabled ? (
                  <>
                    <Lock className="w-4 h-4 text-green-400" />
                    <span className="text-sm text-green-400 font-medium">
                      {encryptedCount}/{totalDisks} {t('diskEncryption.encrypted')}
                    </span>
                  </>
                ) : (
                  <>
                    <Unlock className="w-4 h-4 text-amber-400" />
                    <span className="text-sm text-amber-400 font-medium">{t('diskEncryption.notEncrypted')}</span>
                  </>
                )}
              </div>
            )}
          </div>
        </div>

        {/* 主内容 */}
        {loading ? (
          <div className="flex-1 flex items-center justify-center">
            <Loader2 className="w-12 h-12 text-accent animate-spin" />
          </div>
        ) : status ? (
          <div className="flex-1 space-y-6 animate-slideInUp overflow-auto custom-scrollbar">
            {/* 状态概览 */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
              <div className="bg-slate-800/30 border border-white/5 rounded-xl p-3">
                <div className="flex items-center gap-2 mb-1">
                  <div className="p-1.5 bg-blue-500/10 rounded-lg">
                    <Monitor className="w-4 h-4 text-blue-400" />
                  </div>
                  <div className="text-[10px] text-slate-500">{t('diskEncryption.operatingSystem')}</div>
                </div>
                <div className="text-base text-white font-bold">{status.platform}</div>
              </div>

              <div className="bg-slate-800/30 border border-white/5 rounded-xl p-3">
                <div className="flex items-center gap-2 mb-1">
                  <div className="p-1.5 bg-accent/10 rounded-lg">
                    <Shield className="w-4 h-4 text-accent" />
                  </div>
                  <div className="text-[10px] text-slate-500">{t('diskEncryption.encryptionMethod')}</div>
                </div>
                <div className="text-base text-white font-bold">{status.encryption_method}</div>
              </div>

              <div className="bg-slate-800/30 border border-white/5 rounded-xl p-3">
                <div className="flex items-center gap-2 mb-1">
                  <div className={`p-1.5 rounded-lg ${status.recovery_key_exists ? 'bg-green-500/10' : 'bg-amber-500/10'}`}>
                    <Key className={`w-4 h-4 ${status.recovery_key_exists ? 'text-green-400' : 'text-amber-400'}`} />
                  </div>
                  <div className="text-[10px] text-slate-500">{t('diskEncryption.recoveryKey')}</div>
                </div>
                <div className={`text-base font-bold ${status.recovery_key_exists ? 'text-green-400' : 'text-amber-400'}`}>
                  {status.recovery_key_exists ? t('diskEncryption.keySet') : t('diskEncryption.keyNotSet')}
                </div>
              </div>

              <div className="bg-slate-800/30 border border-white/5 rounded-xl p-3">
                <div className="flex items-center gap-2 mb-1">
                  <div className={`p-1.5 rounded-lg ${status.encryption_in_progress ? 'bg-blue-500/10' : 'bg-slate-700/50'}`}>
                    <Database className={`w-4 h-4 ${status.encryption_in_progress ? 'text-blue-400 animate-pulse' : 'text-slate-400'}`} />
                  </div>
                  <div className="text-[10px] text-slate-500">{t('diskEncryption.encryptionStatus')}</div>
                </div>
                <div className={`text-base font-bold ${status.encryption_in_progress ? 'text-blue-400' : 'text-slate-300'}`}>
                  {status.encryption_in_progress ? t('diskEncryption.inProgress') : t('diskEncryption.idle')}
                </div>
              </div>
            </div>

            {/* 磁盘列表 */}
            <div className="space-y-4">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <HardDrive className="w-5 h-5 text-accent" />
                {t('diskEncryption.diskStatus')}
                <span className="text-sm text-slate-400 font-normal">
                  ({totalDisks} {t('diskEncryption.disks')})
                </span>
              </h3>

              {status.disks.map((disk, index) => (
                <div
                  key={index}
                  className="bg-slate-800/30 border border-white/5 rounded-xl p-6 hover:border-accent/20 transition-all"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex items-start gap-4 flex-1">
                      <div
                        className={`p-3 rounded-lg flex-shrink-0 ${disk.encrypted
                          ? 'bg-green-500/10'
                          : 'bg-amber-500/10'
                          }`}
                      >
                        {disk.encrypted ? (
                          <Lock className="w-6 h-6 text-green-400" />
                        ) : (
                          <Unlock className="w-6 h-6 text-amber-400" />
                        )}
                      </div>

                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="text-white font-medium text-lg truncate">
                            {disk.name}
                          </h4>
                          {disk.is_system_disk && (
                            <span className="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded-full flex-shrink-0">
                              {t('diskEncryption.systemDisk')}
                            </span>
                          )}
                        </div>

                        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-3">
                          <div>
                            <div className="text-xs text-slate-500">{t('diskEncryption.mountPoint')}</div>
                            <div className="text-sm text-slate-300 truncate">{disk.mount_point}</div>
                          </div>
                          <div>
                            <div className="text-xs text-slate-500">{t('diskEncryption.devicePath')}</div>
                            <div className="text-sm text-slate-300 truncate">{disk.path}</div>
                          </div>
                          <div>
                            <div className="text-xs text-slate-500">{t('diskEncryption.fileSystem')}</div>
                            <div className="text-sm text-slate-300">{disk.file_system}</div>
                          </div>
                          <div>
                            <div className="text-xs text-slate-500">{t('diskEncryption.encryptionType')}</div>
                            <div className={`text-sm ${disk.encrypted ? 'text-green-400' : 'text-slate-500'}`}>
                              {disk.encryption_type || t('diskEncryption.notEncrypted')}
                            </div>
                          </div>
                        </div>

                        {/* 容量条 */}
                        <div className="space-y-2">
                          <div className="flex items-center justify-between text-xs">
                            <span className="text-slate-400">
                              {t('diskEncryption.used')} {disk.used} / {disk.size}
                            </span>
                            <span className="text-slate-400">
                              {t('diskEncryption.available')} {disk.available}
                            </span>
                          </div>
                          <Progress
                            percent={disk.usage_percent}
                            strokeColor={{
                              '0%': disk.usage_percent > 90 ? '#ef4444' : disk.usage_percent > 75 ? '#f97316' : '#22c55e',
                              '100%': disk.usage_percent > 90 ? '#dc2626' : disk.usage_percent > 75 ? '#ea580c' : '#16a34a',
                            }}
                            trailColor="rgba(255,255,255,0.05)"
                            showInfo={false}
                            strokeWidth={8}
                          />
                          <div className="text-right text-xs text-slate-500">
                            {disk.usage_percent.toFixed(1)}% {t('diskEncryption.usedPercent')}
                          </div>
                        </div>

                        {/* 加密进度 */}
                        {disk.encryption_progress !== null && (
                          <div className="mt-3 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                            <div className="flex items-center justify-between text-xs mb-2">
                              <span className="text-blue-400 flex items-center gap-1">
                                <Loader2 className="w-3 h-3 animate-spin" />
                                {t('diskEncryption.encryptionInProgress')}
                              </span>
                              <span className="text-blue-400">{disk.encryption_progress}%</span>
                            </div>
                            <Progress
                              percent={disk.encryption_progress}
                              strokeColor="#3b82f6"
                              trailColor="rgba(59, 130, 246, 0.2)"
                              showInfo={false}
                              strokeWidth={6}
                            />
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="flex gap-2 flex-shrink-0">
                      {disk.encrypted ? (
                        <Button
                          danger
                          onClick={() => handleDisableEncryption(disk.path)}
                          disabled={processing || disk.encryption_progress !== null}
                          icon={<Unlock size={16} />}
                        >
                          {t('diskEncryption.disableEncryption')}
                        </Button>
                      ) : (
                        <Button
                          type="primary"
                          onClick={() => handleEnableEncryption(disk.path)}
                          disabled={processing}
                          className="bg-accent hover:bg-accent/80 border-none"
                          icon={<Lock size={16} />}
                        >
                          {t('diskEncryption.enableEncryption')}
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* 安全建议 */}
            {getRecommendations().length > 0 && (
              <div className="bg-slate-800/30 border border-white/5 rounded-xl p-6">
                <h3 className="text-lg font-bold text-white flex items-center gap-2 mb-4">
                  <Shield className="w-5 h-5 text-accent" />
                  {t('diskEncryption.securityRecommendations')}
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {getRecommendations().map((rec, index) => (
                    <div
                      key={index}
                      className="flex items-start gap-3 text-sm text-slate-300 p-3 bg-slate-800/30 rounded-lg"
                    >
                      <CheckCircle2 className="w-4 h-4 text-accent mt-0.5 flex-shrink-0" />
                      <span>{rec}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 警告信息 */}
            {!status.enabled && (
              <Alert
                message={t('diskEncryption.securityWarning')}
                description={t('diskEncryption.notEncryptedWarning')}
                type="warning"
                showIcon
                icon={<AlertTriangle className="w-5 h-5" />}
                className="bg-amber-500/10 border-amber-500/30"
              />
            )}

            {/* 恢复密钥警告 */}
            {status.enabled && !status.recovery_key_exists && (
              <Alert
                message={t('diskEncryption.recoveryKeyReminder')}
                description={t('diskEncryption.recoveryKeyWarning')}
                type="info"
                showIcon
                icon={<Key className="w-5 h-5" />}
                className="bg-blue-500/10 border-blue-500/30"
              />
            )}
          </div>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <AlertCircle className="w-16 h-16 text-slate-500 mx-auto mb-4" />
              <p className="text-slate-400">{t('diskEncryption.cannotGetStatus')}</p>
              <Button
                onClick={loadEncryptionStatus}
                className="mt-4"
                icon={<RefreshCw size={16} />}
              >
                {t('diskEncryption.retry')}
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* 确认对话框 */}
      <Modal
        title={operationType === 'enable' ? t('diskEncryption.enableDiskEncryption') : t('diskEncryption.disableDiskEncryption')}
        open={showWarningModal}
        onOk={confirmOperation}
        onCancel={() => setShowWarningModal(false)}
        okText={t('common.confirm')}
        cancelText={t('common.cancel')}
        okButtonProps={{
          danger: operationType === 'disable',
        }}
      >
        <div className="py-4">
          {operationType === 'enable' ? (
            <div className="space-y-3">
              <Alert
                message={t('diskEncryption.importantNotice')}
                description={t('diskEncryption.enableWarningDesc')}
                type="info"
                showIcon
              />
              <p className="text-slate-600">
                • {t('diskEncryption.backupData')}<br />
                • {t('diskEncryption.saveRecoveryKey')}<br />
                • {t('diskEncryption.encryptionTime')}<br />
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              <Alert
                message={t('diskEncryption.securityWarning')}
                description={t('diskEncryption.disableWarningDesc')}
                type="warning"
                showIcon
              />
              <p className="text-slate-600">
                {t('diskEncryption.confirmDisable')} {selectedDisk} {t('diskEncryption.encryptionQuestion')}
              </p>
            </div>
          )}
        </div>
      </Modal>
      <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </ProFeatureGate>
  );
};

export default DiskEncryption;
