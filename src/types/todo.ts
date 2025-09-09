// 待办事项相关的类型定义

export interface TodoItem {
  id: string
  text: string
  completed: boolean
  createdAt: string // ISO 时间字符串
  completedAt?: string // 完成时间
  deadline?: string // 截止时间 ISO 字符串
  parentId?: string // 父项ID，用于树形结构
  children?: TodoItem[] // 子项列表
}

export interface TodoSettings {
  // 颜色配置
  colors: {
    normal: string // 正常状态文字颜色
    warning: string // 24小时内截止的颜色（黄色）
    urgent: string // 1小时内截止的颜色（红色）
    completed: string // 已完成的颜色（浅灰色）
    background: string // 背景颜色
    border: string // 边框颜色
  }
  // 回收时间设置（天数）
  archiveDays: number
  // Git同步配置
  gitSync: {
    enabled: boolean // 是否启用Git同步
    repositoryUrl: string // Git仓库地址
    sshKeyPath?: string // SSH密钥文件路径
    lastSyncTime?: string // 最后同步时间
    autoSync: boolean // 是否自动同步
  }
}

export interface TodoData {
  data: TodoItem[]
  lastUpdate: string
  source: 'manual' | 'import' | 'sync'
}

export interface ArchivedTodoData {
  todos: TodoItem[]
  archivedAt: string
}
