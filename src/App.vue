<script setup lang="ts">
import type { LocaleKey } from './constants/locale'
import { ElConfigProvider } from 'element-plus'
import { computed, onMounted, watch } from 'vue'
import FloatingWindow from './components/FloatingWindow.vue'
import SettingsModal from './components/SettingsModal.vue'
import { locales } from './constants/locale'
import { useAppStore } from './store/app'

const appStore = useAppStore()

// 计算当前语言配置
const locale = computed(() => {
  const currentLocale = appStore.appSettings.locale as LocaleKey
  return locales[currentLocale] || locales['zh-cn']
})

// 计算 CSS 变量
const cssVariables = computed(() => ({
  // 窗口配置
  '--window-border-radius': `${appStore.appSettings.windowConfig.borderRadius}px`,
  '--window-border-width': `${appStore.appSettings.windowConfig.borderWidth}px`,
  '--window-border-color': appStore.appSettings.windowConfig.borderColor,
  '--window-background': appStore.appSettings.isTransparent ? 'transparent' : appStore.appSettings.colors.background,
  // 待办事项颜色配置
  '--todo-normal-color': appStore.appSettings.colors.normal,
  '--todo-warning-color': appStore.appSettings.colors.warning,
  '--todo-urgent-color': appStore.appSettings.colors.urgent,
  '--todo-completed-color': appStore.appSettings.colors.completed,
  '--todo-border-color': appStore.appSettings.colors.border,
}))

// 将CSS变量应用到html元素
function applyCssVariablesToHtml() {
  const htmlElement = document.documentElement
  const variables = cssVariables.value

  Object.entries(variables).forEach(([key, value]) => {
    htmlElement.style.setProperty(key, value)
  })
}

// 组件挂载时应用CSS变量
onMounted(() => {
  applyCssVariablesToHtml()
})

// 监听CSS变量变化，实时更新到html元素
watch(() => cssVariables.value, () => {
  applyCssVariablesToHtml()
}, { deep: true, immediate: true })
</script>

<template>
  <div id="app">
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
  width: 100%;
  height: 100vh;
  background: transparent;
  border-radius: 8px;
  overflow: hidden;
}
</style>
