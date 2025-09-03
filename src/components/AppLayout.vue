<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const activeModule = ref('question-bank')

const modules = [
  { id: 'question-bank', name: '题库管理', path: '/question-bank' },
  { id: 'exam-management', name: '考试管理', path: '/exam-management' },
  { id: 'student-management', name: '学生管理', path: '/student-management' },
  { id: 'statistics', name: '统计分析', path: '/statistics' },
]

function navigateTo(moduleId: string, path: string) {
  activeModule.value = moduleId
  router.push(path)
}
</script>

<template>
  <div class="app-container">
    <div class="sidebar">
      <div class="logo">
        <h2>ExamX</h2>
      </div>
      <div class="menu">
        <div 
          v-for="module in modules" 
          :key="module.id"
          :class="['menu-item', { active: activeModule === module.id }]"
          @click="navigateTo(module.id, module.path)"
        >
          {{ module.name }}
        </div>
      </div>
    </div>
    <div class="content">
      <div class="header">
        <h1>ExamX 考试系统</h1>
      </div>
      <div class="main-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100%;
}

.sidebar {
  width: 220px;
  background-color: #1e293b;
  color: #fff;
  display: flex;
  flex-direction: column;
}

.logo {
  padding: 20px;
  border-bottom: 1px solid #2c3e50;
}

.logo h2 {
  margin: 0;
  color: #fff;
  font-size: 24px;
}

.menu {
  display: flex;
  flex-direction: column;
  padding: 20px 0;
}

.menu-item {
  padding: 12px 20px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.menu-item:hover {
  background-color: #2c3e50;
}

.menu-item.active {
  background-color: #3b82f6;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header {
  padding: 16px 24px;
  background-color: #fff;
  border-bottom: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.header h1 {
  margin: 0;
  font-size: 20px;
  color: #1e293b;
}

.main-content {
  flex: 1;
  padding: 24px;
  overflow: auto;
  background-color: #f8fafc;
}
</style>
