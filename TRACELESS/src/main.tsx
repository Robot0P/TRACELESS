import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './i18n/config'
import App from './App.tsx'

// 禁用右键菜单
document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  return false;
});

// 禁用开发者工具快捷键
document.addEventListener('keydown', (e) => {
  // 禁用 F12
  if (e.key === 'F12') {
    e.preventDefault();
    return false;
  }
  // 禁用 Ctrl+Shift+I / Cmd+Option+I (开发者工具)
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'I') {
    e.preventDefault();
    return false;
  }
  // 禁用 Ctrl+Shift+J / Cmd+Option+J (控制台)
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'J') {
    e.preventDefault();
    return false;
  }
  // 禁用 Ctrl+Shift+C / Cmd+Option+C (元素检查)
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'C') {
    e.preventDefault();
    return false;
  }
  // 禁用 Ctrl+U / Cmd+U (查看源代码)
  if ((e.ctrlKey || e.metaKey) && e.key === 'u') {
    e.preventDefault();
    return false;
  }
});

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
