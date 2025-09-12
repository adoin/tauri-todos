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

export interface TodoData {
  data: TodoItem[]
  lastUpdate: string
  source: 'manual' | 'import' | 'sync'
}

export interface ArchivedTodoData {
  todos: TodoItem[]
  archivedAt: string
}

export type TodoTimeStatus = 'normal' | 'warning' | 'urgent'
