import React from 'react';
import { useTranslation } from 'react-i18next';
import { Globe } from 'lucide-react';

const LanguageSwitcher: React.FC = () => {
  const { i18n, t } = useTranslation();

  const toggleLanguage = () => {
    const newLang = i18n.language === 'zh-CN' ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(newLang);
  };

  return (
    <button
      onClick={toggleLanguage}
      className="flex items-center justify-center p-3 rounded-xl text-gray-500 hover:text-gray-300 hover:bg-white/5 transition-all cursor-pointer"
      title={t('settings.language')}
    >
      <Globe size={24} />
    </button>
  );
};

export default LanguageSwitcher;
