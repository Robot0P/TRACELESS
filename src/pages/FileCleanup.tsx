import React, { useState } from 'react';
import { Button, Progress, Modal } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  File,
  Trash2,
  FolderOpen,
  CheckCircle2,
  Shield,
  Zap,
  Loader2,
  AlertTriangle,
  ArrowLeft,
  Lock,
  Flame,
  Target,
  X
} from 'lucide-react';

interface WipeMethod {
  value: string;
  label: string;
  description: string;
  passes: number;
  icon: React.ReactNode;
  color: string;
  gradient: string;
}

const FileCleanup: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedMethod, setSelectedMethod] = useState<string>('random');
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [showSuccess, setShowSuccess] = useState(false);
  const [showWarning, setShowWarning] = useState(false);

  const wipeMethods: WipeMethod[] = [
    {
      value: 'zero',
      label: t('fileCleanup.methods.zero.name'),
      description: t('fileCleanup.methods.zero.desc'),
      passes: 1,
      icon: <Zap size={28} />,
      color: 'text-green-400',
      gradient: 'from-green-500 to-emerald-500',
    },
    {
      value: 'random',
      label: t('fileCleanup.methods.random.name'),
      description: t('fileCleanup.methods.random.desc'),
      passes: 3,
      icon: <Shield size={28} />,
      color: 'text-blue-400',
      gradient: 'from-blue-500 to-cyan-500',
    },
    {
      value: 'dod',
      label: t('fileCleanup.methods.dod.name'),
      description: t('fileCleanup.methods.dod.desc'),
      passes: 7,
      icon: <Lock size={28} />,
      color: 'text-orange-400',
      gradient: 'from-orange-500 to-amber-500',
    },
    {
      value: 'gutmann',
      label: t('fileCleanup.methods.gutmann.name'),
      description: t('fileCleanup.methods.gutmann.desc'),
      passes: 35,
      icon: <Flame size={28} />,
      color: 'text-red-400',
      gradient: 'from-red-500 to-rose-500',
    },
  ];

  const handleSelectFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });

      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        setSelectedFiles(files);
      }
    } catch {
      // Silently fail
    }
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });

      if (selected && typeof selected === 'string') {
        setSelectedFiles([selected]);
      }
    } catch {
      // Silently fail
    }
  };

  const handleDelete = () => {
    if (loading) {
      Modal.warning({
        title: t('common.warning'),
        content: t('fileCleanup.status.processing'),
        centered: true,
      });
      return;
    }

    if (selectedFiles.length === 0) {
      Modal.warning({
        title: t('common.warning'),
        content: t('fileCleanup.errors.selectFiles'),
        centered: true,
      });
      return;
    }
    setShowWarning(true);
  };

  const confirmDelete = async () => {
    // 防止重复执行
    if (loading) {
      return;
    }

    setShowWarning(false);
    setLoading(true);
    setProgress(0);

    const totalSelectedFiles = selectedFiles.length;
    let currentFileIndex = 0;

    // 设置进度事件监听器
    const unlisten = await listen<{
      current_file: string;
      total_files: number;
      completed_files: number;
      current_pass: number;
      total_passes: number;
      percentage: number;
    }>('delete-progress', (event) => {
      const { current_file, percentage } = event.payload;

      // 计算总体进度：已完成的文件 + 当前文件的进度
      const overallProgress = ((currentFileIndex + (percentage / 100)) / totalSelectedFiles) * 100;

      setCurrentFile(current_file);
      setProgress(Math.min(overallProgress, 100));
    });

    try {
      for (let i = 0; i < selectedFiles.length; i++) {
        currentFileIndex = i;
        const file = selectedFiles[i];

        // 调用后端删除命令,后端会发送进度事件
        await invoke('secure_delete_file', {
          path: file,
          method: selectedMethod,
        });
      }

      setProgress(100);
      setLoading(false);
      setShowSuccess(true);
      setSelectedFiles([]);
    } catch (error) {
      setLoading(false);
      Modal.error({
        title: t('fileCleanup.errors.deleteFailedTitle'),
        content: String(error),
        centered: true,
      });
    } finally {
      // 清理事件监听器
      unlisten();
    }
  };

  const removeFile = (index: number) => {
    setSelectedFiles(prev => prev.filter((_, i) => i !== index));
  };

  const selectedMethodData = wipeMethods.find(m => m.value === selectedMethod);

  return (
    <div className="h-full flex flex-col relative overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* 背景效果 */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(rgba(217,148,63,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(217,148,63,0.03)_1px,transparent_1px)] bg-[size:50px_50px]" />
        <div className="absolute top-1/4 right-1/4 w-96 h-96 bg-red-500/5 rounded-full blur-3xl animate-pulse" />
      </div>

      <div className="flex-1 flex flex-col p-6 relative z-10 min-h-0">
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
                <Trash2 className="w-7 h-7 text-red-400" />
                {t('fileCleanup.title')}
              </h1>
              <p className="text-sm text-slate-400 mt-1">
                {t('fileCleanup.warnings.irreversible')}
              </p>
            </div>
          </div>
        </div>

        <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-y-auto custom-scrollbar min-h-0">
          {/* 左侧：文件选择区域 */}
          <div className="lg:col-span-2 space-y-6">
            {/* 选择文件卡片 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <FolderOpen className="w-5 h-5 text-accent" />
                  <h3 className="text-lg font-bold text-white">{t('fileCleanup.selectFiles')}</h3>
                </div>
                <div className="flex gap-2">
                  <Button
                    onClick={handleSelectFiles}
                    icon={<File size={16} />}
                    className="bg-blue-500/20 border-blue-500/30 text-blue-400 hover:bg-blue-500/30"
                  >
                    {t('fileCleanup.selectFiles')}
                  </Button>
                  <Button
                    onClick={handleSelectFolder}
                    icon={<FolderOpen size={16} />}
                    className="bg-purple-500/20 border-purple-500/30 text-purple-400 hover:bg-purple-500/30"
                  >
                    {t('fileCleanup.selectFolder')}
                  </Button>
                </div>
              </div>

              {/* 文件列表 */}
              <div className="space-y-2 max-h-64 overflow-y-auto custom-scrollbar">
                {selectedFiles.length === 0 ? (
                  <div className="text-center py-12">
                    <Target className="w-16 h-16 text-slate-600 mx-auto mb-4" />
                    <p className="text-slate-400">{t('fileCleanup.noFilesSelected')}</p>
                    <p className="text-sm text-slate-500 mt-2">{t('fileCleanup.addFiles')}</p>
                  </div>
                ) : (
                  selectedFiles.map((file, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-3 bg-slate-800/50 rounded-lg border border-white/5 hover:border-accent/20 transition-all group"
                    >
                      <div className="flex items-center gap-3 flex-1 min-w-0">
                        <File className="w-5 h-5 text-blue-400 flex-shrink-0" />
                        <span className="text-white truncate">{file.split('/').pop()}</span>
                      </div>
                      <button
                        onClick={() => removeFile(index)}
                        className="p-1 rounded-lg hover:bg-red-500/20 text-slate-400 hover:text-red-400 transition-all opacity-0 group-hover:opacity-100"
                      >
                        <X size={16} />
                      </button>
                    </div>
                  ))
                )}
              </div>

              {selectedFiles.length > 0 && (
                <div className="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg flex items-start gap-2">
                  <AlertTriangle className="w-5 h-5 text-amber-400 flex-shrink-0 mt-0.5" />
                  <div className="text-sm text-amber-300">
                    <p className="font-medium">{t('fileCleanup.warnings.title')}</p>
                    <p className="text-amber-400/80 mt-1">{t('fileCleanup.warnings.irreversible')}</p>
                  </div>
                </div>
              )}
            </div>

            {/* 粉碎方法选择 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInLeft" style={{ animationDelay: '0.1s' }}>
              <div className="flex items-center gap-2 mb-4">
                <Shield className="w-5 h-5 text-accent" />
                <h3 className="text-lg font-bold text-white">{t('fileCleanup.deleteMethod')}</h3>
              </div>

              <div className="grid grid-cols-2 gap-3">
                {wipeMethods.map((method) => (
                  <button
                    key={method.value}
                    onClick={() => setSelectedMethod(method.value)}
                    className={`
                      relative p-4 rounded-xl border-2 transition-all duration-300 text-left overflow-hidden
                      ${selectedMethod === method.value
                        ? 'border-accent bg-accent/10 shadow-lg shadow-accent/20'
                        : 'border-white/10 hover:border-white/20 bg-slate-800/30'
                      }
                    `}
                  >
                    <div className={`absolute inset-0 bg-gradient-to-br ${method.gradient} opacity-0 ${selectedMethod === method.value ? 'opacity-10' : ''} transition-opacity`} />

                    <div className="relative z-10">
                      <div className="flex items-center justify-between mb-2">
                        <div className={method.color}>
                          {method.icon}
                        </div>
                        {selectedMethod === method.value && (
                          <CheckCircle2 className="w-5 h-5 text-accent" />
                        )}
                      </div>
                      <div className="text-white font-bold mb-1">{method.label}</div>
                      <div className="text-xs text-slate-400">{method.description}</div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* 右侧：操作面板 */}
          <div className="space-y-6">
            {/* 当前算法详情 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight">
              <h3 className="text-lg font-bold text-white mb-4">{t('fileCleanup.methodDetails')}</h3>

              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <div className={`p-3 rounded-lg bg-slate-700/50 ${selectedMethodData?.color}`}>
                    {selectedMethodData?.icon}
                  </div>
                  <div>
                    <div className="text-white font-medium">{selectedMethodData?.label}</div>
                    <div className="text-xs text-slate-400">{selectedMethodData?.description}</div>
                  </div>
                </div>

                <div className="space-y-2 pt-4 border-t border-white/5">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('fileCleanup.overwritePasses')}</span>
                    <span className="text-white font-medium">{selectedMethodData?.passes} {t('fileCleanup.times')}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('fileCleanup.securityLevel')}</span>
                    <span className="text-accent font-medium">
                      {selectedMethodData?.passes === 1 && t('fileCleanup.levelStandard')}
                      {selectedMethodData?.passes === 3 && t('fileCleanup.levelHigh')}
                      {selectedMethodData?.passes === 7 && t('fileCleanup.levelMilitary')}
                      {selectedMethodData?.passes === 35 && t('fileCleanup.levelExtreme')}
                    </span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">{t('fileCleanup.selectedFiles')}</span>
                    <span className="text-white font-medium">{selectedFiles.length} {t('fileCleanup.items')}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* 操作按钮 */}
            <div className="bg-gradient-to-br from-slate-800/50 to-slate-900/50 backdrop-blur-sm rounded-2xl border border-white/5 p-6 shadow-2xl animate-slideInRight" style={{ animationDelay: '0.1s' }}>
              <Button
                type="primary"
                size="large"
                block
                onClick={handleDelete}
                disabled={selectedFiles.length === 0 || loading}
                className="h-12 bg-gradient-to-r from-red-600 to-red-500 border-none hover:from-red-500 hover:to-red-400 text-white font-bold"
                icon={loading ? <Loader2 className="animate-spin" size={20} /> : <Flame size={20} />}
              >
                {loading ? t('fileCleanup.deleting') : t('fileCleanup.startDelete')}
              </Button>

              {loading && (
                <div className="mt-4 space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-slate-300">{t('fileCleanup.deleting')}</span>
                    <span className="text-accent font-mono">{Math.floor(progress)}%</span>
                  </div>
                  <Progress
                    percent={progress}
                    strokeColor={{
                      '0%': '#ef4444',
                      '100%': '#dc2626',
                    }}
                    trailColor="rgba(255,255,255,0.05)"
                    showInfo={false}
                    strokeWidth={10}
                  />
                  <div className="text-xs text-slate-400 truncate">
                    {currentFile}
                  </div>
                  <div className="text-xs text-slate-500 text-center mt-2">
                    {t('fileCleanup.pleaseWait')}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* 警告确认弹窗 */}
      <Modal
        open={showWarning}
        onCancel={() => setShowWarning(false)}
        footer={null}
        centered
        width={500}
      >
        <div className="text-center py-6">
          <div className="relative inline-block mb-6">
            <div className="absolute inset-0 bg-red-500/20 rounded-full blur-2xl animate-pulse" />
            <div className="relative p-6 bg-gradient-to-br from-red-500/20 to-red-500/5 rounded-full border border-red-500/30">
              <AlertTriangle className="w-16 h-16 text-red-400" />
            </div>
          </div>

          <h2 className="text-2xl font-bold text-white mb-2">{t('fileCleanup.warnings.confirmTitle')}</h2>
          <p className="text-slate-400 mb-2">
            {t('fileCleanup.warnings.aboutToShred')} <span className="text-accent font-bold">{selectedFiles.length}</span> {t('fileCleanup.warnings.files')}
          </p>
          <p className="text-red-400 text-sm mb-6">
            ⚠️ {t('fileCleanup.warnings.cannotRecover')}
          </p>

          <div className="flex gap-3 justify-center">
            <Button
              size="large"
              onClick={() => setShowWarning(false)}
              className="bg-slate-700 hover:bg-slate-600 text-white border-none"
            >
              {t('common.cancel')}
            </Button>
            <Button
              type="primary"
              size="large"
              danger
              onClick={confirmDelete}
              className="bg-red-600 hover:bg-red-500"
            >
              {t('fileCleanup.status.confirmShred')}
            </Button>
          </div>
        </div>
      </Modal>

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

          <h2 className="text-2xl font-bold text-white mb-2">{t('fileCleanup.status.complete')}</h2>
          <p className="text-slate-400 mb-6">
            {t('fileCleanup.status.safelyDeleted')}
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
    </div>
  );
};

export default FileCleanup;
