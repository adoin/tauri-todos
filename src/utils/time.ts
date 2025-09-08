// 时间工具函数
export const timeUtils = {
  // 检查是否接近截止时间
  getTimeStatus: (deadline: string): 'normal' | 'warning' | 'urgent' => {
    const now = new Date()
    const deadlineDate = new Date(deadline)
    const diffHours = (deadlineDate.getTime() - now.getTime()) / (1000 * 60 * 60)

    if (diffHours <= 1)
      return 'urgent'
    if (diffHours <= 24)
      return 'warning'
    return 'normal'
  },

  // 检查是否需要归档
  shouldArchive: (completedAt: string, archiveDays: number): boolean => {
    const completed = new Date(completedAt)
    const now = new Date()
    const diffDays = (now.getTime() - completed.getTime()) / (1000 * 60 * 60 * 24)
    return diffDays >= archiveDays
  },

  // 格式化时间显示
  formatTime: (isoString: string): string => {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  },
}
