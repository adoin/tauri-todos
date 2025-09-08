<script setup lang="ts">
import type { LocaleKey } from './constants/locale'
import { ElConfigProvider } from 'element-plus'
import { computed } from 'vue'
import FloatingWindow from './components/FloatingWindow.vue'
import SettingsModal from './components/SettingsModal.vue'
import { locales } from './constants/locale'
import { useAppStore } from './store/app'
import { useTodoStore } from './store/todo'
import { hexToRGB } from './utils/color'

const appStore = useAppStore()
const todoStore = useTodoStore()

// 计算当前语言配置
const locale = computed(() => {
  const currentLocale = appStore.locale as LocaleKey
  return locales[currentLocale] || locales['zh-cn']
})
/* rgb() => rgba() */
function rgb2rgba(rgb: string, opacity: number) {
  if (rgb.startsWith('#')) {
    rgb = hexToRGB(rgb)
  }
  return rgb.replace('RGB(', 'RGBA(').replace(')', `, ${opacity})`).toLowerCase()
}
// 计算 CSS 变量
const cssVariables = computed(() => ({
  // 窗口配置
  '--window-width': `${appStore.windowConfig.width}px`,
  '--window-height': `${appStore.windowConfig.height}px`,
  '--window-opacity': appStore.isTransparent ? appStore.windowConfig.opacity : 1,
  '--window-border-radius': `${appStore.windowConfig.borderRadius}px`,
  '--window-border-width': `${appStore.windowConfig.borderWidth}px`,
  '--window-border-color': appStore.windowConfig.borderColor,
  '--window-background': appStore.isTransparent ? 'transparent' : rgb2rgba(todoStore.settings.colors.background, appStore.windowConfig.opacity),
  // 待办事项颜色配置
  '--todo-normal-color': todoStore.settings.colors.normal,
  '--todo-warning-color': todoStore.settings.colors.warning,
  '--todo-urgent-color': todoStore.settings.colors.urgent,
  '--todo-completed-color': todoStore.settings.colors.completed,
  '--todo-border-color': todoStore.settings.colors.border,
}))
</script>

<template>
  <div id="app" :style="cssVariables">
    <ElConfigProvider :locale="locale">
      <FloatingWindow />
      <SettingsModal />
    </ElConfigProvider>
  </div>
</template>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: transparent;
}

#app {
  width: 100vw;
  height: 100vh;
  background: transparent;
  border-radius: 8px;
  overflow: hidden;
}
</style>
