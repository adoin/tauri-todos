// 应用相关的类型定义

export interface WindowConfig {
  borderRadius: number
  borderColor: string
  borderWidth: number
}
export interface AppSettings {
  locale: string
  isTransparent: boolean
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
  windowConfig: WindowConfig
}
export interface WindowPosition {
  x: number
  y: number
}
