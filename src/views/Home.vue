<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

const modules = [
  { id: 'question-bank', name: '题库管理', icon: 'i-carbon-document-multiple', description: '管理试题，创建和编辑不同类型的题目' },
  { id: 'exam-management', name: '考试管理', icon: 'i-carbon-exam-mode', description: '创建和管理考试，设置考试规则和时间' },
  { id: 'student-management', name: '学生管理', icon: 'i-carbon-user-multiple', description: '管理学生信息和考试权限' },
  { id: 'statistics', name: '统计分析', icon: 'i-carbon-chart-line', description: '查看考试统计数据和学生表现' },
]

function navigateTo(path: string) {
  router.push(path)
}
</script>

<template>
  <div class="home-container">
    <h1 class="title">欢迎使用 ExamX 考试系统</h1>
    <p class="subtitle">选择一个模块开始使用</p>
    
    <div class="modules-grid">
      <div 
        v-for="module in modules" 
        :key="module.id"
        class="module-card"
        @click="navigateTo(`/${module.id}`)"
      >
        <div class="icon" :class="module.icon"></div>
        <h2>{{ module.name }}</h2>
        <p>{{ module.description }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.title {
  font-size: 28px;
  font-weight: 600;
  color: #1e293b;
  margin-bottom: 8px;
  text-align: center;
}

.subtitle {
  font-size: 16px;
  color: #64748b;
  margin-bottom: 32px;
  text-align: center;
}

.modules-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 24px;
}

.module-card {
  background-color: #fff;
  border-radius: 8px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: transform 0.3s, box-shadow 0.3s;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.module-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.icon {
  font-size: 36px;
  margin-bottom: 16px;
  color: #3b82f6;
}

.module-card h2 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #1e293b;
}

.module-card p {
  font-size: 14px;
  color: #64748b;
  line-height: 1.5;
}
</style>
