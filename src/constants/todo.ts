import type { TodoSettings } from '../types/todo'

// 默认设置
export const defaultTodoSettings: TodoSettings = {
  colors: {
    normal: '#4f2937',
    warning: '#f59e0b', // 黄色
    urgent: '#ef4444', // 红色
    completed: '#f5dbd6', // 浅灰色
    background: '#60a5fa88',
    border: '#29cdcd',
  },
  archiveDays: 30, // 默认30天后归档
}
