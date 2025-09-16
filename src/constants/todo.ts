import type { AppSettings } from '../types/app'

// 默认设置
export const defaultAppSettings: AppSettings = {
  locale: 'zh-cn',
  isTransparent: false,
  windowConfig: {
    borderRadius: 8,
    borderColor: '#3b82f6',
    borderWidth: 2,
  },
  colors: {
    normal: '#4f2937',
    warning: '#f59e0b', // 黄色
    urgent: '#ef4444', // 红色
    completed: '#f5dbd6', // 浅灰色
    background: '#60a5fa',
    border: '#29cdcd',
  },
  archiveDays: 30, // 默认30天后归档
  autoSync: '0', // 默认不自动同步
}
