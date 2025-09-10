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
  gitSync: {
    enabled: false, // 默认关闭Git同步
    repositoryUrl: '', // 空的仓库地址
    authMethod: 'https' as const, // 默认使用HTTPS认证
    accessToken: '', // Personal Access Token
    sshKeyPath: '', // SSH私钥文件路径
    autoSync: true, // 默认启用自动同步
  },
}
