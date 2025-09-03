<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '../store/app'

const appStore = useAppStore()

const closeSettings = () => {
  appStore.closeSettings()
}

const updateOpacity = (value: number) => {
  appStore.updateWindowConfig({ opacity: value / 100 })
}

const updateBorderRadius = (value: number) => {
  appStore.updateWindowConfig({ borderRadius: value })
}

const updateBorderWidth = (value: number) => {
  appStore.updateWindowConfig({ borderWidth: value })
}

const updateBorderColor = (color: string) => {
  appStore.updateWindowConfig({ borderColor: color })
}

// 阻止事件冒泡
const stopPropagation = (event: Event) => {
  event.stopPropagation()
}
</script>

<template>
  <Teleport to="body">
    <!-- 遮罩层 -->
    <div
      v-if="appStore.isSettingsOpen"
      class="modal-overlay"
      @click="closeSettings"
    >
      <!-- 模态框 -->
      <div
        class="settings-modal"
        @click="stopPropagation"
      >
        <!-- 头部 -->
        <div class="modal-header">
          <h2 class="modal-title">Ton 设置</h2>
          <button
            class="close-modal-btn"
            @click="closeSettings"
          >
            ✕
          </button>
        </div>

        <!-- 设置内容 -->
        <div class="modal-content">
          <!-- 外观设置 -->
          <div class="setting-group">
            <h3 class="group-title">外观设置</h3>

            <div class="setting-item">
              <label class="setting-label">窗口透明度</label>
              <div class="slider-container">
                <input
                  v-model="appStore.windowConfig.opacity"
                  type="range"
                  min="0.1"
                  max="1"
                  step="0.1"
                  class="opacity-slider"
                  @input="(e) => updateOpacity(Number((e.target as HTMLInputElement).value) * 100)"
                >
                <span class="slider-value">{{ Math.round(appStore.windowConfig.opacity * 100) }}%</span>
              </div>
            </div>

            <div class="setting-item">
              <label class="setting-label">圆角半径</label>
              <div class="slider-container">
                <input
                  v-model="appStore.windowConfig.borderRadius"
                  type="range"
                  min="0"
                  max="20"
                  step="1"
                  class="radius-slider"
                  @input="(e) => updateBorderRadius(Number((e.target as HTMLInputElement).value))"
                >
                <span class="slider-value">{{ appStore.windowConfig.borderRadius }}px</span>
              </div>
            </div>

            <div class="setting-item">
              <label class="setting-label">边框宽度</label>
              <div class="slider-container">
                <input
                  v-model="appStore.windowConfig.borderWidth"
                  type="range"
                  min="0"
                  max="5"
                  step="1"
                  class="border-slider"
                  @input="(e) => updateBorderWidth(Number((e.target as HTMLInputElement).value))"
                >
                <span class="slider-value">{{ appStore.windowConfig.borderWidth }}px</span>
              </div>
            </div>

            <div class="setting-item">
              <label class="setting-label">边框颜色</label>
              <input
                v-model="appStore.windowConfig.borderColor"
                type="color"
                class="color-picker"
                @input="(e) => updateBorderColor((e.target as HTMLInputElement).value)"
              >
            </div>
          </div>

          <!-- 行为设置 -->
          <div class="setting-group">
            <h3 class="group-title">行为设置</h3>

            <div class="setting-item">
              <label class="checkbox-label">
                <input
                  v-model="appStore.isTransparent"
                  type="checkbox"
                  @change="appStore.toggleTransparency"
                >
                <span class="checkmark"></span>
                透明背景
              </label>
            </div>
          </div>
        </div>

        <!-- 底部按钮 -->
        <div class="modal-footer">
          <button
            class="btn btn-secondary"
            @click="closeSettings"
          >
            取消
          </button>
          <button
            class="btn btn-primary"
            @click="closeSettings"
          >
            确定
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-modal {
  background: white;
  border-radius: 12px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #111827;
}

.close-modal-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  transition: all 0.2s ease;
}

.close-modal-btn:hover {
  background: #f3f4f6;
  color: #374151;
}

.modal-content {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.setting-group {
  margin-bottom: 32px;
}

.group-title {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: #374151;
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 8px;
}

.setting-item {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  flex: 1;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 200px;
}

.opacity-slider,
.radius-slider,
.border-slider {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: #e5e7eb;
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}

.opacity-slider::-webkit-slider-thumb,
.radius-slider::-webkit-slider-thumb,
.border-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.slider-value {
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  min-width: 40px;
  text-align: right;
}

.color-picker {
  width: 60px;
  height: 32px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: none;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.checkbox-label input[type="checkbox"] {
  display: none;
}

.checkmark {
  width: 18px;
  height: 18px;
  border: 2px solid #d1d5db;
  border-radius: 4px;
  position: relative;
  transition: all 0.2s ease;
}

.checkbox-label input[type="checkbox"]:checked + .checkmark {
  background: #3b82f6;
  border-color: #3b82f6;
}

.checkbox-label input[type="checkbox"]:checked + .checkmark::after {
  content: '✓';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: white;
  font-size: 12px;
  font-weight: bold;
}

.modal-footer {
  padding: 20px 24px;
  border-top: 1px solid #e5e7eb;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn-secondary {
  background: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d5db;
}

.btn-secondary:hover {
  background: #e5e7eb;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background: #2563eb;
}
</style>
