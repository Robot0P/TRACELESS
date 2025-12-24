import React from 'react';
import { Modal } from 'antd';
import { useTranslation } from 'react-i18next';

interface AboutDialogProps {
  open: boolean;
  onClose: () => void;
}

const AboutDialog: React.FC<AboutDialogProps> = ({ open, onClose }) => {
  const { t } = useTranslation();

  return (
    <Modal
      open={open}
      onCancel={onClose}
      footer={null}
      centered
      width={400}
      className="about-dialog"
    >
      <div className="text-center py-6">
        {/* 应用图标 */}
        <div className="relative inline-block mb-6">
          <div className="absolute inset-0 bg-accent/20 rounded-full blur-2xl opacity-50" />
          <img
            src="/logo-shield.png"
            alt="App Icon"
            className="relative w-24 h-24 object-contain mx-auto drop-shadow-2xl"
          />
        </div>

        {/* 应用名称 */}
        <h2 className="text-xl font-bold text-white mb-1">
          {t('about.appName')}
        </h2>
        <p className="text-slate-400 text-sm mb-4">
          Traceless Protection
        </p>

        {/* 版本信息 */}
        <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-accent/10 border border-accent/20 rounded-lg mb-4">
          <span className="text-accent text-sm font-medium">v1.0.0</span>
        </div>

        {/* 描述 */}
        <p className="text-slate-400 text-sm mb-6 px-4">
          {t('about.description')}
        </p>

        {/* 版权信息 */}
        <p className="text-slate-500 text-xs">
          © 2025 Traceless Team
        </p>

        {/* 确定按钮 */}
        <button
          onClick={onClose}
          className="mt-6 px-8 py-2 bg-accent hover:bg-accent/80 text-white font-medium rounded-lg transition-colors"
        >
          {t('about.confirm')}
        </button>
      </div>
    </Modal>
  );
};

export default AboutDialog;
