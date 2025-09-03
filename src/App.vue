<script setup lang="ts">
import FloatingWindow from './components/FloatingWindow.vue'
import SettingsModal from './components/SettingsModal.vue'
</script>

<template>
  <div id="app">
    <FloatingWindow />
    <SettingsModal />
  </div>
</template>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

#app {
  background: transparent;
}
</style>
