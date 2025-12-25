import React, { useState, useEffect } from 'react';
import { Button, Progress, Modal, DatePicker, TimePicker, Input, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { ProFeatureGate } from '../components/ProFeatureGate';
import { LicenseActivationDialog } from '../components/LicenseActivation';
import {
  Clock,
  Calendar,
  File,
  CheckCircle2,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  FolderOpen,
  Edit3,
  FileText,
  Eye,
  Pencil,
  Shuffle,
  Zap,
  RefreshCw,
  HardDrive,
  Info,
  Copy,
} from 'lucide-react';
import dayjs, { Dayjs } from 'dayjs';

interface TimestampData {
  modified?: string;
  accessed?: string;
  created?: string;
}

interface FileInfo {
  name: string;
  path: string;
  size: number;
  is_directory: boolean;
}

interface TimestampField {
  key: 'created' | 'modified' | 'accessed';
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  gradient: string;
  supported: boolean;
}

const TimestampModifier: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [currentTimestamps, setCurrentTimestamps] = useState<TimestampData | null>(null);
  const [fileInfo, setFileInfo] = useState<FileInfo | null>(null);
  const [platform, setPlatform] = useState<string>('macos');
  const [showLicenseDialog, setShowLicenseDialog] = useState(false);

  const [modifiedDate, setModifiedDate] = useState<Dayjs | null>(null);
  const [modifiedTime, setModifiedTime] = useState<Dayjs | null>(null);
  const [accessedDate, setAccessedDate] = useState<Dayjs | null>(null);
  const [accessedTime, setAccessedTime] = useState<Dayjs | null>(null);
  const [createdDate, setCreatedDate] = useState<Dayjs | null>(null);
  const [createdTime, setCreatedTime] = useState<Dayjs | null>(null);

  useEffect(() => {
    invoke<string>('get_platform').then((p) => {
      setPlatform(p.toLowerCase());
    }).catch(() => {
      setPlatform('macos');
    });
  }, []);

  // 创建时间修改支持情况
  const createdTimeSupported = platform === 'windows' || platform === 'macos';

  const timestampFields: TimestampField[] = [
    {
      key: 'created',
      label: t('timestampModifier.createdTime'),
      description: 'Birth Time / Creation Time',
      icon: <FileText size={24} />,
      color: 'text-green-400',
      gradient: 'from-green-500 to-emerald-500',
      supported: createdTimeSupported,
    },
    {
      key: 'modified',
      label: t('timestampModifier.modifiedTime'),
      description: 'Last Modified Time (mtime)',
      icon: <Pencil size={24} />,
      color: 'text-blue-400',
      gradient: 'from-blue-500 to-cyan-500',
      supported: true,
    },
    {
      key: 'accessed',
      label: t('timestampModifier.accessedTime'),
      description: 'Last Access Time (atime)',
      icon: <Eye size={24} />,
      color: 'text-purple-400',
      gradient: 'from-purple-500 to-pink-500',
      supported: true,
    },
  ];

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const loadFileInfo = async (filePath: string) => {
    try {
      const info = await invoke<FileInfo>('get_file_info', { filePath });
      setFileInfo(info);
    } catch {
      // 手动构建基本信息
      const pathParts = filePath.split(/[/\\]/);
      setFileInfo({
        name: pathParts[pathParts.length - 1] || filePath,
        path: filePath,
        size: 0,
        is_directory: false,
      });
    }
  };

  const loadTimestamps = async (filePath: string) => {
    try {
      const timestamps = await invoke<TimestampData>('get_file_timestamps', {
        filePath,
      });
      setCurrentTimestamps(timestamps);

      if (timestamps.modified) {
        const dt = dayjs(timestamps.modified);
        setModifiedDate(dt);
        setModifiedTime(dt);
      }
      if (timestamps.accessed) {
        const dt = dayjs(timestamps.accessed);
        setAccessedDate(dt);
        setAccessedTime(dt);
      }
      if (timestamps.created) {
        const dt = dayjs(timestamps.created);
        setCreatedDate(dt);
        setCreatedTime(dt);
      }
    } catch {
      // Silently fail
    }
  };

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: t('timestampModifier.selectFile'),
      });

      if (selected && typeof selected === 'string') {
        setSelectedFile(selected);
        await loadFileInfo(selected);
        await loadTimestamps(selected);
      }
    } catch {
      // Silently fail
    }
  };

  const handleRefresh = async () => {
    if (!selectedFile) return;
    setRefreshing(true);
    await loadTimestamps(selectedFile);
    setTimeout(() => setRefreshing(false), 500);
  };

  const handleSetCurrentTime = () => {
    const now = dayjs();
    setModifiedDate(now);
    setModifiedTime(now);
    setAccessedDate(now);
    setAccessedTime(now);
    if (createdTimeSupported) {
      setCreatedDate(now);
      setCreatedTime(now);
    }
  };

  const handleRandomTime = () => {
    const randomDays = Math.floor(Math.random() * 365);
    const randomHours = Math.floor(Math.random() * 24);
    const randomMinutes = Math.floor(Math.random() * 60);
    const randomSeconds = Math.floor(Math.random() * 60);

    const randomTime = dayjs()
      .subtract(randomDays, 'day')
      .hour(randomHours)
      .minute(randomMinutes)
      .second(randomSeconds);

    setModifiedDate(randomTime);
    setModifiedTime(randomTime);
    setAccessedDate(randomTime);
    setAccessedTime(randomTime);
    if (createdTimeSupported) {
      setCreatedDate(randomTime);
      setCreatedTime(randomTime);
    }
  };

  const handleCopyFromTimestamp = (sourceKey: 'created' | 'modified' | 'accessed') => {
    let sourceDate: Dayjs | null = null;
    let sourceTime: Dayjs | null = null;

    switch (sourceKey) {
      case 'created':
        sourceDate = createdDate;
        sourceTime = createdTime;
        break;
      case 'modified':
        sourceDate = modifiedDate;
        sourceTime = modifiedTime;
        break;
      case 'accessed':
        sourceDate = accessedDate;
        sourceTime = accessedTime;
        break;
    }

    if (sourceDate && sourceTime) {
      setModifiedDate(sourceDate);
      setModifiedTime(sourceTime);
      setAccessedDate(sourceDate);
      setAccessedTime(sourceTime);
      if (createdTimeSupported) {
        setCreatedDate(sourceDate);
        setCreatedTime(sourceTime);
      }
    }
  };

  const handleModify = async () => {
    if (!selectedFile) {
      Modal.warning({
        title: t('common.warning'),
        content: t('timestampModifier.errors.selectFile'),
        centered: true,
      });
      return;
    }

    const timestamps: Record<string, string> = {};

    if (modifiedDate && modifiedTime) {
      const combined = modifiedDate
        .hour(modifiedTime.hour())
        .minute(modifiedTime.minute())
        .second(modifiedTime.second());
      timestamps.modified = combined.toISOString();
    }

    if (accessedDate && accessedTime) {
      const combined = accessedDate
        .hour(accessedTime.hour())
        .minute(accessedTime.minute())
        .second(accessedTime.second());
      timestamps.accessed = combined.toISOString();
    }

    if (createdDate && createdTime && createdTimeSupported) {
      const combined = createdDate
        .hour(createdTime.hour())
        .minute(createdTime.minute())
        .second(createdTime.second());
      timestamps.created = combined.toISOString();
    }

    if (Object.keys(timestamps).length === 0) {
      Modal.warning({
        title: t('common.warning'),
        content: t('timestampModifier.errors.noTimestamp'),
        centered: true,
      });
      return;
    }

    Modal.confirm({
      title: t('timestampModifier.warnings.confirmModify'),
      icon: <AlertTriangle className="text-amber-500" size={24} />,
      content: (
        <div>
          <p className="text-slate-400 mb-2">
            {t('timestampModifier.warnings.cannotUndo')}
          </p>
          <p className="text-amber-400 text-sm font-mono truncate">
            {fileInfo?.name || selectedFile.split('/').pop() || selectedFile.split('\\').pop()}
          </p>
        </div>
      ),
      okText: t('timestampModifier.startModify'),
      okType: 'danger',
      cancelText: t('common.cancel'),
      centered: true,
      onOk: async () => {
        setLoading(true);

        try {
          await invoke<string>('modify_file_timestamps', {
            filePath: selectedFile,
            timestamps,
          });

          // 刷新时间戳显示
          await loadTimestamps(selectedFile);

          setLoading(false);
          setShowSuccess(true);
        } catch (error) {
          setLoading(false);
          Modal.error({
            title: t('timestampModifier.errors.modifyFailed'),
            content: String(error),
            centered: true,
          });
        }
      },
    });
  };

  const formatTimestamp = (timestamp?: string) => {
    if (!timestamp) return '-';
    return dayjs(timestamp).format('YYYY-MM-DD HH:mm:ss');
  };

  const getDateValue = (key: 'created' | 'modified' | 'accessed') => {
    switch (key) {
      case 'created': return createdDate;
      case 'modified': return modifiedDate;
      case 'accessed': return accessedDate;
    }
  };

  const getTimeValue = (key: 'created' | 'modified' | 'accessed') => {
    switch (key) {
      case 'created': return createdTime;
      case 'modified': return modifiedTime;
      case 'accessed': return accessedTime;
    }
  };

  const setDateValue = (key: 'created' | 'modified' | 'accessed', value: Dayjs | null) => {
    switch (key) {
      case 'created': setCreatedDate(value); break;
      case 'modified': setModifiedDate(value); break;
      case 'accessed': setAccessedDate(value); break;
    }
  };

  const setTimeValue = (key: 'created' | 'modified' | 'accessed', value: Dayjs | null) => {
    switch (key) {
      case 'created': setCreatedTime(value); break;
      case 'modified': setModifiedTime(value); break;
      case 'accessed': setAccessedTime(value); break;
    }
  };

  return (
    <ProFeatureGate
      feature="timestamp_modifier"
      onUpgradeClick={() => setShowLicenseDialog(true)}
    >
      <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute bottom-1/4 left-1/4 w-96 h-96 bg-indigo-500/5 rounded-full blur-3xl animate-pulse" />
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
                <Clock className="w-7 h-7 text-indigo-400" />
                {t('timestampModifier.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('timestampModifier.subtitle')}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {selectedFile && (
              <button
                onClick={handleRefresh}
                className={`p-2 rounded-lg bg-slate-800/50 border border-white/5 hover:border-accent/30 text-slate-400 hover:text-white transition-all ${refreshing ? 'animate-spin' : ''}`}
              >
                <RefreshCw size={18} />
              </button>
            )}
            <div className="flex items-center gap-2 px-4 py-2 bg-indigo-500/10 border border-indigo-500/20 rounded-lg">
              <Calendar className="w-4 h-4 text-indigo-400" />
              <span className="text-sm text-indigo-300 capitalize">{platform}</span>
            </div>
          </div>
        </div>

        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-hidden">
          {/* 左侧：文件选择 + 时间轴 */}
          <div className="lg:col-span-2 flex flex-col space-y-6 min-h-0 overflow-y-auto custom-scrollbar">
            {/* 文件选择 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="flex items-center gap-2 mb-4">
                <File className="w-5 h-5 text-indigo-400" />
                <h3 className="text-lg font-bold text-white">{t('timestampModifier.file')}</h3>
              </div>

              <div className="flex gap-3">
                <Input
                  value={selectedFile}
                  readOnly
                  placeholder={t('timestampModifier.noFileSelected')}
                  className="flex-1 bg-slate-800/50 border-white/10 text-white"
                />
                <Button
                  type="primary"
                  size="large"
                  icon={<FolderOpen size={20} />}
                  onClick={handleSelectFile}
                  className="bg-indigo-600 hover:bg-indigo-500 border-none"
                >
                  {t('common.browse')}
                </Button>
              </div>

              {fileInfo && selectedFile && (
                <div className="mt-4 p-4 bg-slate-800/30 border border-white/5 rounded-lg">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div className="text-xs text-slate-500 mb-1">{t('timestampModifier.fileName')}</div>
                      <div className="text-sm text-white font-medium truncate">{fileInfo.name}</div>
                    </div>
                    <div>
                      <div className="text-xs text-slate-500 mb-1">{t('timestampModifier.fileSize')}</div>
                      <div className="text-sm text-white font-medium">
                        {fileInfo.size > 0 ? formatFileSize(fileInfo.size) : t('timestampModifier.unknown')}
                      </div>
                    </div>
                    <div className="col-span-2">
                      <div className="text-xs text-slate-500 mb-1">{t('timestampModifier.filePath')}</div>
                      <div className="text-xs text-indigo-300 font-mono truncate">{fileInfo.path}</div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 时间轴 */}
            <div className="flex-1 bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.1s' }}>
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Clock className="w-5 h-5 text-accent" />
                  <h3 className="text-lg font-bold text-white">{t('timestampModifier.currentTimestamps')}</h3>
                </div>
                {!createdTimeSupported && (
                  <Tooltip title={t('timestampModifier.linuxNotSupported')}>
                    <div className="flex items-center gap-1 px-2 py-1 bg-amber-500/10 border border-amber-500/20 rounded text-xs text-amber-400">
                      <Info size={12} />
                      <span>{t('timestampModifier.partialSupport')}</span>
                    </div>
                  </Tooltip>
                )}
              </div>

              <div className="space-y-4">
                {timestampFields.map((field) => (
                  <div key={field.key} className="relative pl-8">
                    {/* Timeline dot */}
                    <div className={`absolute left-0 top-6 w-3 h-3 rounded-full border-2 ${field.supported ? field.color : 'text-slate-600'} border-current`} />
                    {/* Timeline line */}
                    {field.key !== 'accessed' && (
                      <div className="absolute left-[5px] top-9 w-0.5 h-full bg-gradient-to-b from-white/20 to-transparent" />
                    )}

                    <div className={`p-4 bg-slate-800/30 rounded-xl border transition-all ${field.supported ? 'border-white/10 hover:border-white/20' : 'border-slate-700/50 opacity-60'}`}>
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center gap-3">
                          <div className={`p-2 rounded-lg bg-slate-700/50 ${field.supported ? field.color : 'text-slate-500'}`}>
                            {field.icon}
                          </div>
                          <div>
                            <div className="text-white font-bold flex items-center gap-2">
                              {field.label}
                              {!field.supported && (
                                <span className="text-xs text-slate-500">({t('timestampModifier.notSupported')})</span>
                              )}
                            </div>
                            <div className="text-xs text-slate-400">{field.description}</div>
                          </div>
                        </div>
                        {field.supported && currentTimestamps && currentTimestamps[field.key] && (
                          <Tooltip title={t('timestampModifier.copyToAll')}>
                            <button
                              onClick={() => handleCopyFromTimestamp(field.key)}
                              className="p-1.5 rounded bg-slate-700/50 text-slate-400 hover:text-white hover:bg-slate-600/50 transition-colors"
                            >
                              <Copy size={14} />
                            </button>
                          </Tooltip>
                        )}
                      </div>

                      {currentTimestamps && currentTimestamps[field.key] && (
                        <div className="mb-3 p-2 bg-black/20 rounded-lg border border-white/5">
                          <div className="text-xs text-slate-400 mb-1">{t('timestampModifier.currentValue')}</div>
                          <div className="text-sm text-slate-300 font-mono">
                            {formatTimestamp(currentTimestamps[field.key])}
                          </div>
                        </div>
                      )}

                      <div className="grid grid-cols-2 gap-2">
                        <DatePicker
                          value={getDateValue(field.key)}
                          onChange={(value) => setDateValue(field.key, value)}
                          placeholder={t('timestampModifier.selectDate')}
                          className="w-full"
                          disabled={!field.supported}
                        />
                        <TimePicker
                          value={getTimeValue(field.key)}
                          onChange={(value) => setTimeValue(field.key, value)}
                          placeholder={t('timestampModifier.selectTime')}
                          className="w-full"
                          disabled={!field.supported}
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* 右侧：快捷操作 + 控制 */}
          <div className="space-y-6">
            {/* 快捷操作 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight">
              <h3 className="text-lg font-bold text-white mb-4">{t('timestampModifier.quickActions')}</h3>

              <div className="space-y-3">
                <Button
                  size="large"
                  block
                  onClick={handleSetCurrentTime}
                  className="bg-slate-700 hover:bg-slate-600 text-white border-none h-12"
                  icon={<Zap size={20} />}
                >
                  {t('timestampModifier.setCurrentTime')}
                </Button>
                <Button
                  size="large"
                  block
                  onClick={handleRandomTime}
                  className="bg-slate-700 hover:bg-slate-600 text-white border-none h-12"
                  icon={<Shuffle size={20} />}
                >
                  {t('timestampModifier.setRandomTime')}
                </Button>
              </div>

              {/* 平台支持说明 */}
              <div className="mt-4 p-3 bg-slate-800/50 border border-white/5 rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <HardDrive className="w-4 h-4 text-slate-400" />
                  <span className="text-xs text-slate-400">{t('timestampModifier.platformSupport')}</span>
                </div>
                <div className="space-y-1 text-xs">
                  <div className="flex justify-between">
                    <span className="text-slate-500">{t('timestampModifier.createdTime')}</span>
                    <span className={createdTimeSupported ? 'text-green-400' : 'text-slate-500'}>
                      {createdTimeSupported ? t('timestampModifier.supported') : t('timestampModifier.notSupportedShort')}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-500">{t('timestampModifier.modifiedTime')}</span>
                    <span className="text-green-400">{t('timestampModifier.supported')}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-500">{t('timestampModifier.accessedTime')}</span>
                    <span className="text-green-400">{t('timestampModifier.supported')}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* 控制面板 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.1s' }}>
              <h3 className="text-lg font-bold text-white mb-4">{t('common.operation')}</h3>

              <Button
                type="primary"
                size="large"
                block
                onClick={handleModify}
                disabled={!selectedFile || loading}
                className="h-12 bg-gradient-to-r from-indigo-600 to-indigo-500 border-none hover:from-indigo-500 hover:to-indigo-400 text-white font-bold"
                icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Edit3 size={20} />}
              >
                {loading ? t('timestampModifier.modifying') : t('timestampModifier.startModify')}
              </Button>

              {loading && (
                <div className="mt-4">
                  <Progress
                    percent={100}
                    status="active"
                    strokeColor={{
                      '0%': '#6366f1',
                      '100%': '#8b5cf6',
                    }}
                    trailColor="rgba(255,255,255,0.05)"
                    showInfo={false}
                    strokeWidth={10}
                  />
                </div>
              )}

              <div className="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                <div className="flex items-start gap-2">
                  <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
                  <div className="text-xs text-amber-300">
                    <div className="font-bold mb-1">{t('timestampModifier.notice')}</div>
                    {t('timestampModifier.noticeDesc')}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
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

          <h2 className="text-2xl font-bold text-white mb-2">{t('timestampModifier.modifyComplete')}</h2>
          <p className="text-slate-400 mb-6">
            {t('timestampModifier.successMessage')}
          </p>

          <Button
            type="primary"
            size="large"
            onClick={() => setShowSuccess(false)}
            className="bg-accent hover:bg-accent/80 border-none"
          >
            {t('fileCleanup.status.done')}
          </Button>
        </div>
      </Modal>
      <LicenseActivationDialog isOpen={showLicenseDialog} onClose={() => setShowLicenseDialog(false)} />
      </div>
    </ProFeatureGate>
  );
};

export default TimestampModifier;
