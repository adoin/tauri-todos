<script setup lang="ts">
import { ElButton, ElColorPicker, ElDialog, ElForm, ElFormItem, ElInput, ElMessage, ElMessageBox, ElOption, ElRadio, ElRadioGroup, ElSelect, ElSlider, ElSwitch } from 'element-plus'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'

const appStore = useAppStore()
const todoStore = useTodoStore()

function closeSettings() {
  appStore.closeSettings()
}

// 待办事项设置相关方法
async function updateArchiveDays(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  await todoStore.updateSettings({ archiveDays: val })
}

async function updateTodoColor(colorKey: string, color: string | null) {
  if (color) {
    const colors = { ...todoStore.settings.colors, [colorKey]: color }
    await todoStore.updateSettings({ colors })
  }
}

async function clearArchivedTodos() {
  try {
    await ElMessageBox.confirm('确认清空所有归档历史？此操作不可撤销。', '确认清空', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    await todoStore.clearArchivedTodos()
    ElMessage.success('归档历史已清空')
  }
  catch {
    // 用户取消操作
  }
}

async function resetColorsToDefault() {
  try {
    await ElMessageBox.confirm('确认将所有颜色设置恢复为默认值？', '确认重置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    await todoStore.resetColorsToDefault()
    ElMessage.success('颜色设置已恢复为默认值')
  }
  catch {
    // 用户取消操作
  }
}

function updateOpacity(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ opacity: val })
}

function updateBorderRadius(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ borderRadius: val })
}

function updateBorderWidth(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ borderWidth: val })
}

function updateBorderColor(color: string | null) {
  if (color) {
    appStore.updateWindowConfig({ borderColor: color })
  }
}

// Git同步相关方法
async function updateGitSyncEnabled(enabled: boolean) {
  await todoStore.updateSettings({
    gitSync: {
      ...todoStore.settings.gitSync,
      enabled,
    },
  })
}

async function updateGitRepositoryUrl() {
  const url = todoStore.settings.gitSync.repositoryUrl.trim()

  // 根据认证方式校验URL格式
  if (url && !isValidGitUrl(url, todoStore.settings.gitSync.authMethod)) {
    const example = todoStore.settings.gitSync.authMethod === 'https'
      ? 'https://github.com/username/repo.git'
      : 'git@github.com:username/repo.git'
    ElMessage.error(`请输入正确的${todoStore.settings.gitSync.authMethod.toUpperCase()}格式Git仓库地址，例如：${example}`)
    return
  }

  await todoStore.updateSettings({
    gitSync: {
      ...todoStore.settings.gitSync,
      repositoryUrl: url,
    },
  })
}

async function handleAuthMethodChange() {
  try {
    // 切换认证方式时清空相关字段
    if (todoStore.settings.gitSync.authMethod === 'https') {
      todoStore.settings.gitSync.sshKeyPath = ''
    }
    else {
      todoStore.settings.gitSync.accessToken = ''
    }
    await todoStore.saveSettings()
    ElMessage.success('认证方式已更新')
  }
  catch (error) {
    console.error('更新认证方式失败:', error)
    ElMessage.error('更新认证方式失败')
  }
}

async function updateAccessToken() {
  try {
    await todoStore.saveSettings()
    ElMessage.success('Access Token已更新')
  }
  catch (error) {
    console.error('更新Access Token失败:', error)
    ElMessage.error('更新Access Token失败')
  }
}

async function updateSSHKeyPath() {
  await todoStore.updateSettings({
    gitSync: {
      ...todoStore.settings.gitSync,
      sshKeyPath: todoStore.settings.gitSync.sshKeyPath,
    },
  })
}

async function selectSSHKeyFile() {
  try {
    const result = await todoStore.selectSSHKeyFile()
    if (result) {
      await todoStore.updateSettings({
        gitSync: {
          ...todoStore.settings.gitSync,
          sshKeyPath: result,
        },
      })
      ElMessage.success('SSH密钥文件路径已设置')
    }
  }
  catch (error) {
    console.error('选择SSH密钥文件失败:', error)
    ElMessage.error('选择SSH密钥文件失败')
  }
}

function isValidGitUrl(url: string, authMethod: 'https' | 'ssh'): boolean {
  if (authMethod === 'https') {
    // 检查HTTPS格式：https://hostname/path.git
    const httpsPattern = /^https:\/\/[a-zA-Z0-9.-]+\/[\w./-]+\.git$/
    return httpsPattern.test(url)
  }
  else {
    // 检查SSH格式：git@hostname:path.git
    const sshPattern = /^git@[a-zA-Z0-9.-]+:[\w./-]+\.git$/
    return sshPattern.test(url)
  }
}

async function updateGitAutoSync(autoSync: boolean) {
  await todoStore.updateSettings({
    gitSync: {
      ...todoStore.settings.gitSync,
      autoSync,
    },
  })
}

// 包装函数处理ElSwitch的类型问题
function handleGitSyncEnabledChange(val: string | number | boolean) {
  updateGitSyncEnabled(Boolean(val))
}

function handleGitAutoSyncChange(val: string | number | boolean) {
  updateGitAutoSync(Boolean(val))
}

async function initializeGitSync() {
  try {
    if (!todoStore.settings.gitSync.repositoryUrl) {
      ElMessage.warning('请先设置Git仓库地址')
      return
    }

    // 根据认证方式校验URL格式
    if (!isValidGitUrl(todoStore.settings.gitSync.repositoryUrl, todoStore.settings.gitSync.authMethod)) {
      const example = todoStore.settings.gitSync.authMethod === 'https'
        ? 'https://github.com/username/repo.git'
        : 'git@github.com:username/repo.git'
      ElMessage.error(`请输入正确的${todoStore.settings.gitSync.authMethod.toUpperCase()}格式Git仓库地址，例如：${example}`)
      return
    }

    // 检查认证信息
    if (todoStore.settings.gitSync.authMethod === 'ssh') {
      if (!todoStore.settings.gitSync.sshKeyPath || todoStore.settings.gitSync.sshKeyPath.trim() === '') {
        ElMessage.error('SSH认证需要设置SSH私钥文件路径')
        return
      }
    }
    else {
      if (!todoStore.settings.gitSync.accessToken || todoStore.settings.gitSync.accessToken.trim() === '') {
        ElMessage.error('HTTPS认证需要设置Personal Access Token')
        return
      }
    }

    ElMessage.info('正在初始化Git同步...')
    await todoStore.initializeGitSync()
    ElMessage.success('Git同步初始化成功')
  }
  catch (error) {
    console.error('Git同步初始化失败:', error)
    ElMessage.error(`Git同步初始化失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

async function manualSync() {
  try {
    if (!todoStore.settings.gitSync.repositoryUrl) {
      ElMessage.warning('请先设置Git仓库地址')
      return
    }

    // 根据认证方式校验URL格式
    if (!isValidGitUrl(todoStore.settings.gitSync.repositoryUrl, todoStore.settings.gitSync.authMethod)) {
      const example = todoStore.settings.gitSync.authMethod === 'https'
        ? 'https://github.com/username/repo.git'
        : 'git@github.com:username/repo.git'
      ElMessage.error(`请输入正确的${todoStore.settings.gitSync.authMethod.toUpperCase()}格式Git仓库地址，例如：${example}`)
      return
    }

    ElMessage.info('正在同步数据...')
    await todoStore.syncWithGit()
    ElMessage.success('数据同步成功')
  }
  catch (error) {
    console.error('数据同步失败:', error)
    ElMessage.error(`数据同步失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

function formatSyncTime(syncTime: string) {
  const date = new Date(syncTime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

// 调试方法
async function refreshSettings() {
  try {
    await todoStore.loadSettings()
    ElMessage.success('设置已刷新')
  }
  catch (error) {
    console.error('刷新设置失败:', error)
    ElMessage.error('刷新设置失败')
  }
}

async function checkGitRemoteUrl() {
  try {
    ElMessage.info('正在检查Git远程URL...')
    const result = await todoStore.checkGitRemoteUrl()
    ElMessage.success(`当前Git远程URL: ${result}`)
  }
  catch (error) {
    console.error('检查Git远程URL失败:', error)
    ElMessage.error(`检查Git远程URL失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

async function updateGitRemoteUrl() {
  try {
    if (!todoStore.settings.gitSync.repositoryUrl) {
      ElMessage.warning('请先设置Git仓库地址')
      return
    }

    ElMessage.info('正在更新Git远程URL...')
    const result = await todoStore.updateGitRemoteUrl()
    ElMessage.success(result)
  }
  catch (error) {
    console.error('更新Git远程URL失败:', error)
    ElMessage.error(`更新Git远程URL失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

async function resetGitSyncSettings() {
  try {
    await todoStore.updateSettings({
      gitSync: {
        enabled: false,
        repositoryUrl: '',
        autoSync: true,
      },
    })
    ElMessage.success('Git设置已重置')
  }
  catch (error) {
    console.error('重置Git设置失败:', error)
    ElMessage.error('重置Git设置失败')
  }
}

async function testGitPushAuth() {
  try {
    if (!todoStore.settings.gitSync.repositoryUrl) {
      ElMessage.warning('请先设置Git仓库地址')
      return
    }

    // 根据认证方式校验URL格式
    if (!isValidGitUrl(todoStore.settings.gitSync.repositoryUrl, todoStore.settings.gitSync.authMethod)) {
      const example = todoStore.settings.gitSync.authMethod === 'https'
        ? 'https://github.com/username/repo.git'
        : 'git@github.com:username/repo.git'
      ElMessage.error(`请输入正确的${todoStore.settings.gitSync.authMethod.toUpperCase()}格式Git仓库地址，例如：${example}`)
      return
    }

    ElMessage.info('正在测试Git推送权限...')
    const result = await todoStore.testGitPushAuth()
    ElMessage.success(result)
  }
  catch (error) {
    console.error('Git推送权限测试失败:', error)
    ElMessage.error(`Git推送权限测试失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

async function checkLocalFiles() {
  try {
    ElMessage.info('正在检查本地同步文件...')
    const result = await todoStore.checkLocalSyncFiles()
    console.log('本地同步文件:', result)

    if (result.exists) {
      ElMessage.success(`找到 ${result.count} 个同步文件`)
    }
    else {
      ElMessage.warning('未找到同步文件')
    }
  }
  catch (error) {
    console.error('检查本地文件失败:', error)
    ElMessage.error(`检查本地文件失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}
</script>

<template>
  <ElDialog v-model="appStore.isSettingsOpen" title="Ton 设置" width="500px" :before-close="closeSettings">
    <div
      class="max-h-96 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-gray-100 hover:scrollbar-thumb-gray-400"
    >
      <ElForm label-width="120px" label-position="left">
        <!-- 外观设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            外观设置
          </h3>

          <ElFormItem label="窗口不透明度">
            <div class="flex items-center gap-3 w-full">
              <ElSlider
                v-model="appStore.windowConfig.opacity" :min="0.1" :max="1" :step="0.1"
                :format-tooltip="(val: number) => `${Math.round(val * 100)}%`" style="flex: 1"
                @change="updateOpacity"
              />
              <span class="text-sm text-gray-500 min-w-12">{{ Math.round(appStore.windowConfig.opacity * 100) }}%</span>
            </div>
          </ElFormItem>

          <ElFormItem label="圆角半径">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.windowConfig.borderRadius"
                type="number"
                :min="0"
                :max="20"
                style="width: 120px"
                @change="(value: string) => updateBorderRadius(Number(value))"
              />
              <span class="text-sm text-gray-500">px</span>
            </div>
          </ElFormItem>

          <ElFormItem label="边框宽度">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.windowConfig.borderWidth"
                type="number"
                :min="0"
                :max="5"
                style="width: 120px"
                @change="(value: string) => updateBorderWidth(Number(value))"
              />
              <span class="text-sm text-gray-500">px</span>
            </div>
          </ElFormItem>
        </div>

        <!-- 待办事项设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            待办事项设置
          </h3>

          <ElFormItem label="归档时间（天）">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="todoStore.settings.archiveDays"
                type="number"
                :min="1"
                :max="365"
                style="width: 120px"
                @change="(value: string) => updateArchiveDays(Number(value))"
              />
              <span class="text-sm text-gray-500">天</span>
            </div>
          </ElFormItem>

          <ElFormItem label="清除归档历史">
            <ElButton type="danger" size="small" @click="clearArchivedTodos">
              清空历史
            </ElButton>
          </ElFormItem>
        </div>

        <!-- 颜色主题设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            颜色主题
          </h3>

          <ElFormItem label="正常状态">
            <ElColorPicker
              :model-value="todoStore.settings.colors.normal"
              @change="(color: string | null) => updateTodoColor('normal', color)"
            />
          </ElFormItem>

          <ElFormItem label="警告状态（24小时内）">
            <ElColorPicker
              :model-value="todoStore.settings.colors.warning"
              @change="(color: string | null) => updateTodoColor('warning', color)"
            />
          </ElFormItem>

          <ElFormItem label="紧急状态（1小时内）">
            <ElColorPicker
              :model-value="todoStore.settings.colors.urgent"
              @change="(color: string | null) => updateTodoColor('urgent', color)"
            />
          </ElFormItem>

          <ElFormItem label="已完成">
            <ElColorPicker
              :model-value="todoStore.settings.colors.completed"
              @change="(color: string | null) => updateTodoColor('completed', color)"
            />
          </ElFormItem>

          <ElFormItem label="背景颜色">
            <ElColorPicker
              :model-value="todoStore.settings.colors.background"
              @change="(color: string | null) => updateTodoColor('background', color)"
            />
          </ElFormItem>

          <ElFormItem label="边框颜色">
            <ElColorPicker
              :model-value="todoStore.settings.colors.border"
              @change="(color: string | null) => updateTodoColor('border', color)"
            />
          </ElFormItem>

          <ElFormItem label="窗口边框颜色">
            <ElColorPicker v-model="appStore.windowConfig.borderColor" @change="updateBorderColor" />
          </ElFormItem>

          <ElFormItem label="重置颜色">
            <ElButton type="warning" size="small" @click="resetColorsToDefault">
              恢复默认颜色
            </ElButton>
          </ElFormItem>
        </div>

        <!-- Git同步设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            Git同步设置
          </h3>
          <!-- 调试信息 -->
          <div v-if="true" class="mb-4 p-2 bg-gray-100 rounded text-xs">
            <p>调试信息:</p>
            <p>gitSync存在: {{ !!todoStore.settings.gitSync }}</p>
            <p>enabled: {{ todoStore.settings.gitSync?.enabled }}</p>
            <p>repositoryUrl: {{ todoStore.settings.gitSync?.repositoryUrl }}</p>
            <p>autoSync: {{ todoStore.settings.gitSync?.autoSync }}</p>
          </div>

          <ElFormItem label="启用Git同步">
            <div class="flex items-center gap-2">
              <ElSwitch v-model="todoStore.settings.gitSync.enabled" @change="handleGitSyncEnabledChange" />
              <span class="text-sm text-gray-600">
                {{ todoStore.settings.gitSync.enabled ? '已启用' : '已关闭' }}
              </span>
            </div>
          </ElFormItem>

          <ElFormItem v-if="todoStore.settings.gitSync.enabled" label="认证方式">
            <ElRadioGroup v-model="todoStore.settings.gitSync.authMethod" @change="handleAuthMethodChange">
              <ElRadio value="https">
                HTTPS + Personal Access Token
              </ElRadio>
              <ElRadio value="ssh">
                SSH + 私钥文件
              </ElRadio>
            </ElRadioGroup>
          </ElFormItem>

          <ElFormItem v-if="todoStore.settings.gitSync.enabled" label="Git仓库地址">
            <ElInput
              v-model="todoStore.settings.gitSync.repositoryUrl"
              :placeholder="todoStore.settings.gitSync.authMethod === 'https'
                ? 'https://github.com/username/repo.git'
                : 'git@github.com:username/repo.git'"
              @change="updateGitRepositoryUrl"
            />
            <div class="text-xs text-gray-500 mt-1">
              {{ todoStore.settings.gitSync.authMethod === 'https'
                ? '请输入HTTPS格式的Git仓库地址'
                : '请输入SSH格式的Git仓库地址' }}
            </div>
          </ElFormItem>

          <!-- HTTPS认证配置 -->
          <ElFormItem
            v-if="todoStore.settings.gitSync.enabled && todoStore.settings.gitSync.authMethod === 'https'"
            label="Personal Access Token"
          >
            <ElInput
              v-model="todoStore.settings.gitSync.accessToken"
              type="password"
              placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
              show-password
              @change="updateAccessToken"
            />
            <div class="text-xs text-gray-500 mt-1">
              GitHub Personal Access Token，需要repo权限
            </div>
          </ElFormItem>

          <!-- SSH认证配置 -->
          <ElFormItem
            v-if="todoStore.settings.gitSync.enabled && todoStore.settings.gitSync.authMethod === 'ssh'"
            label="SSH私钥文件路径"
          >
            <div class="flex gap-2">
              <ElInput
                v-model="todoStore.settings.gitSync.sshKeyPath"
                placeholder="C:\Users\用户名\.ssh\id_rsa"
                @change="updateSSHKeyPath"
              />
              <ElButton type="primary" size="small" @click="selectSSHKeyFile">
                选择文件
              </ElButton>
            </div>
            <div class="text-xs text-gray-500 mt-1">
              指定SSH私钥文件的完整路径，用于Git认证
            </div>
          </ElFormItem>

          <ElFormItem v-if="todoStore.settings.gitSync.enabled" label="自动同步">
            <div class="flex items-center gap-2">
              <ElSwitch v-model="todoStore.settings.gitSync.autoSync" @change="handleGitAutoSyncChange" />
              <span class="text-sm text-gray-600">
                {{ todoStore.settings.gitSync.autoSync ? '每天自动同步' : '手动同步' }}
              </span>
            </div>
          </ElFormItem>

          <ElFormItem v-if="todoStore.settings.gitSync.enabled" label="同步操作">
            <div class="flex gap-2">
              <ElButton type="primary" size="small" @click="initializeGitSync">
                初始化同步
              </ElButton>
              <ElButton type="success" size="small" @click="manualSync">
                手动同步
              </ElButton>
              <ElButton type="warning" size="small" @click="testGitPushAuth">
                测试推送权限
              </ElButton>
            </div>
          </ElFormItem>

          <ElFormItem v-if="todoStore.settings.gitSync.lastSyncTime" label="最后同步时间">
            <span class="text-sm text-gray-500">
              {{ formatSyncTime(todoStore.settings.gitSync.lastSyncTime) }}
            </span>
          </ElFormItem>

          <ElFormItem label="调试操作">
            <div class="flex gap-2">
              <ElButton type="info" size="small" @click="checkGitRemoteUrl">
                检查Git URL
              </ElButton>
              <ElButton type="success" size="small" @click="updateGitRemoteUrl">
                更新Git URL
              </ElButton>
              <ElButton type="warning" size="small" @click="resetGitSyncSettings">
                重置Git设置
              </ElButton>
            </div>
          </ElFormItem>
        </div>

        <!-- 行为设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            行为设置
          </h3>

          <ElFormItem label="背景模式">
            <div class="flex items-center gap-2">
              <ElSwitch v-model="appStore.isTransparent" />
              <span class="text-sm text-gray-600">
                {{ appStore.isTransparent ? '透明背景' : '纯色背景' }}
              </span>
            </div>
          </ElFormItem>

          <ElFormItem label="界面语言">
            <ElSelect v-model="appStore.locale" style="width: 200px" @change="appStore.updateLocale">
              <ElOption label="中文" value="zh-cn" />
              <ElOption label="English" value="en" />
            </ElSelect>
          </ElFormItem>
        </div>
      </ElForm>
    </div>

    <template #footer>
      <div class="flex justify-end gap-3">
        <ElButton @click="closeSettings">
          取消
        </ElButton>
        <ElButton type="primary" @click="closeSettings">
          确定
        </ElButton>
      </div>
    </template>
  </ElDialog>
</template>
