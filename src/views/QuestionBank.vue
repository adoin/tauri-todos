<script setup lang="ts">
import { ref, onMounted } from 'vue'

// Mock data for questions
const questions = ref([
  { 
    id: 1, 
    type: 'single-choice', 
    content: '以下哪个是JavaScript的基本数据类型？', 
    options: ['Object', 'Array', 'String', 'Function'],
    answer: 'String',
    difficulty: 'easy',
    category: '编程基础'
  },
  { 
    id: 2, 
    type: 'multiple-choice', 
    content: '以下哪些是HTTP请求方法？', 
    options: ['GET', 'POST', 'DELETE', 'FETCH'],
    answer: ['GET', 'POST', 'DELETE'],
    difficulty: 'medium',
    category: '网络'
  },
  { 
    id: 3, 
    type: 'true-false', 
    content: 'HTML是一种编程语言。', 
    options: ['True', 'False'],
    answer: 'False',
    difficulty: 'easy',
    category: '编程基础'
  },
])

const categories = ref(['编程基础', '网络', '数据结构', '算法', '操作系统'])
const questionTypes = ref(['single-choice', 'multiple-choice', 'true-false', 'fill-blank', 'short-answer'])
const difficulties = ref(['easy', 'medium', 'hard'])

const showAddForm = ref(false)
const newQuestion = ref({
  type: 'single-choice',
  content: '',
  options: ['', '', '', ''],
  answer: '',
  difficulty: 'medium',
  category: '编程基础'
})

const searchQuery = ref('')
const filterCategory = ref('')
const filterType = ref('')
const filterDifficulty = ref('')

function addQuestion() {
  const id = questions.value.length > 0 
    ? Math.max(...questions.value.map(q => q.id)) + 1 
    : 1
  
  const questionToAdd = {
    ...newQuestion.value,
    id
  }
  
  questions.value.push(questionToAdd)
  resetForm()
  showAddForm.value = false
}

function resetForm() {
  newQuestion.value = {
    type: 'single-choice',
    content: '',
    options: ['', '', '', ''],
    answer: '',
    difficulty: 'medium',
    category: '编程基础'
  }
}

function deleteQuestion(id: number) {
  questions.value = questions.value.filter(q => q.id !== id)
}

function addOption() {
  if (newQuestion.value.options.length < 6) {
    newQuestion.value.options.push('')
  }
}

function removeOption(index: number) {
  if (newQuestion.value.options.length > 2) {
    newQuestion.value.options.splice(index, 1)
  }
}

function getQuestionTypeText(type: string) {
  const typeMap: Record<string, string> = {
    'single-choice': '单选题',
    'multiple-choice': '多选题',
    'true-false': '判断题',
    'fill-blank': '填空题',
    'short-answer': '简答题'
  }
  return typeMap[type] || type
}

function getDifficultyText(difficulty: string) {
  const difficultyMap: Record<string, string> = {
    'easy': '简单',
    'medium': '中等',
    'hard': '困难'
  }
  return difficultyMap[difficulty] || difficulty
}

const filteredQuestions = computed(() => {
  return questions.value.filter(q => {
    const matchesSearch = searchQuery.value === '' || 
      q.content.toLowerCase().includes(searchQuery.value.toLowerCase())
    
    const matchesCategory = filterCategory.value === '' || 
      q.category === filterCategory.value
    
    const matchesType = filterType.value === '' || 
      q.type === filterType.value
    
    const matchesDifficulty = filterDifficulty.value === '' || 
      q.difficulty === filterDifficulty.value
    
    return matchesSearch && matchesCategory && matchesType && matchesDifficulty
  })
})
</script>

<template>
  <div class="question-bank-container">
    <div class="header-actions">
      <h1>题库管理</h1>
      <button class="btn-primary" @click="showAddForm = true">
        <div class="i-carbon-add mr-1"></div>
        添加题目
      </button>
    </div>

    <div class="filters">
      <div class="search-box">
        <input 
          v-model="searchQuery"
          type="text" 
          placeholder="搜索题目..."
          class="search-input"
        >
        <div class="i-carbon-search search-icon"></div>
      </div>
      
      <div class="filter-selects">
        <select v-model="filterCategory" class="filter-select">
          <option value="">所有分类</option>
          <option v-for="category in categories" :key="category" :value="category">
            {{ category }}
          </option>
        </select>
        
        <select v-model="filterType" class="filter-select">
          <option value="">所有题型</option>
          <option v-for="type in questionTypes" :key="type" :value="type">
            {{ getQuestionTypeText(type) }}
          </option>
        </select>
        
        <select v-model="filterDifficulty" class="filter-select">
          <option value="">所有难度</option>
          <option v-for="difficulty in difficulties" :key="difficulty" :value="difficulty">
            {{ getDifficultyText(difficulty) }}
          </option>
        </select>
      </div>
    </div>

    <div class="questions-list">
      <div v-if="filteredQuestions.length === 0" class="no-questions">
        没有找到符合条件的题目
      </div>
      
      <div v-for="question in filteredQuestions" :key="question.id" class="question-card">
        <div class="question-header">
          <div class="question-meta">
            <span class="question-type">{{ getQuestionTypeText(question.type) }}</span>
            <span class="question-category">{{ question.category }}</span>
            <span :class="['question-difficulty', `difficulty-${question.difficulty}`]">
              {{ getDifficultyText(question.difficulty) }}
            </span>
          </div>
          <div class="question-actions">
            <button class="btn-icon" title="编辑">
              <div class="i-carbon-edit"></div>
            </button>
            <button class="btn-icon" title="删除" @click="deleteQuestion(question.id)">
              <div class="i-carbon-trash-can"></div>
            </button>
          </div>
        </div>
        
        <div class="question-content">
          <p>{{ question.content }}</p>
        </div>
        
        <div v-if="question.options && question.options.length > 0" class="question-options">
          <div v-for="(option, index) in question.options" :key="index" class="option">
            <span class="option-label">{{ String.fromCharCode(65 + index) }}.</span>
            <span class="option-text">{{ option }}</span>
          </div>
        </div>
        
        <div class="question-answer">
          <strong>答案：</strong>
          <span v-if="Array.isArray(question.answer)">
            {{ question.answer.join(', ') }}
          </span>
          <span v-else>
            {{ question.answer }}
          </span>
        </div>
      </div>
    </div>

    <!-- Add Question Modal -->
    <div v-if="showAddForm" class="modal-overlay">
      <div class="modal">
        <div class="modal-header">
          <h2>添加新题目</h2>
          <button class="btn-icon" @click="showAddForm = false">
            <div class="i-carbon-close"></div>
          </button>
        </div>
        
        <div class="modal-body">
          <div class="form-group">
            <label>题目类型</label>
            <select v-model="newQuestion.type" class="form-control">
              <option v-for="type in questionTypes" :key="type" :value="type">
                {{ getQuestionTypeText(type) }}
              </option>
            </select>
          </div>
          
          <div class="form-group">
            <label>分类</label>
            <select v-model="newQuestion.category" class="form-control">
              <option v-for="category in categories" :key="category" :value="category">
                {{ category }}
              </option>
            </select>
          </div>
          
          <div class="form-group">
            <label>难度</label>
            <select v-model="newQuestion.difficulty" class="form-control">
              <option v-for="difficulty in difficulties" :key="difficulty" :value="difficulty">
                {{ getDifficultyText(difficulty) }}
              </option>
            </select>
          </div>
          
          <div class="form-group">
            <label>题目内容</label>
            <textarea 
              v-model="newQuestion.content" 
              class="form-control" 
              rows="3" 
              placeholder="输入题目内容..."
            ></textarea>
          </div>
          
          <div v-if="['single-choice', 'multiple-choice'].includes(newQuestion.type)" class="form-group">
            <label>选项</label>
            <div 
              v-for="(option, index) in newQuestion.options" 
              :key="index" 
              class="option-input"
            >
              <span class="option-label">{{ String.fromCharCode(65 + index) }}.</span>
              <input 
                v-model="newQuestion.options[index]" 
                type="text" 
                class="form-control" 
                placeholder="输入选项内容..."
              >
              <button 
                v-if="newQuestion.options.length > 2" 
                class="btn-icon" 
                @click="removeOption(index)"
              >
                <div class="i-carbon-subtract"></div>
              </button>
            </div>
            
            <button 
              v-if="newQuestion.options.length < 6" 
              class="btn-text" 
              @click="addOption"
            >
              <div class="i-carbon-add mr-1"></div>
              添加选项
            </button>
          </div>
          
          <div class="form-group">
            <label>答案</label>
            <div v-if="newQuestion.type === 'single-choice'">
              <select v-model="newQuestion.answer" class="form-control">
                <option v-for="(option, index) in newQuestion.options" :key="index" :value="option">
                  {{ String.fromCharCode(65 + index) }}. {{ option }}
                </option>
              </select>
            </div>
            
            <div v-else-if="newQuestion.type === 'multiple-choice'">
              <div v-for="(option, index) in newQuestion.options" :key="index" class="checkbox-group">
                <input 
                  :id="`option-${index}`" 
                  type="checkbox" 
                  :value="option"
                  v-model="newQuestion.answer"
                >
                <label :for="`option-${index}`">
                  {{ String.fromCharCode(65 + index) }}. {{ option }}
                </label>
              </div>
            </div>
            
            <div v-else-if="newQuestion.type === 'true-false'">
              <div class="radio-group">
                <input id="true" type="radio" value="True" v-model="newQuestion.answer">
                <label for="true">正确</label>
              </div>
              <div class="radio-group">
                <input id="false" type="radio" value="False" v-model="newQuestion.answer">
                <label for="false">错误</label>
              </div>
            </div>
            
            <div v-else>
              <input 
                v-model="newQuestion.answer" 
                type="text" 
                class="form-control" 
                placeholder="输入答案..."
              >
            </div>
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="btn-secondary" @click="showAddForm = false">取消</button>
          <button class="btn-primary" @click="addQuestion">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.question-bank-container {
  width: 100%;
}

.header-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-actions h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #1e293b;
}

.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 24px;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 200px;
}

.search-input {
  width: 100%;
  padding: 10px 16px 10px 40px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 14px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: #64748b;
  font-size: 18px;
}

.filter-selects {
  display: flex;
  gap: 12px;
}

.filter-select {
  padding: 10px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 14px;
  background-color: white;
}

.questions-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.no-questions {
  padding: 24px;
  text-align: center;
  background-color: white;
  border-radius: 8px;
  color: #64748b;
}

.question-card {
  background-color: white;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.question-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.question-meta {
  display: flex;
  gap: 8px;
}

.question-type,
.question-category,
.question-difficulty {
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  background-color: #f1f5f9;
  color: #64748b;
}

.question-type {
  background-color: #e0f2fe;
  color: #0369a1;
}

.question-category {
  background-color: #f0fdf4;
  color: #166534;
}

.difficulty-easy {
  background-color: #ecfdf5;
  color: #047857;
}

.difficulty-medium {
  background-color: #fef3c7;
  color: #b45309;
}

.difficulty-hard {
  background-color: #fee2e2;
  color: #b91c1c;
}

.question-actions {
  display: flex;
  gap: 8px;
}

.question-content {
  margin-bottom: 12px;
  font-size: 16px;
  color: #1e293b;
}

.question-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.option {
  display: flex;
  gap: 8px;
  font-size: 14px;
  color: #334155;
}

.option-label {
  font-weight: 600;
  min-width: 20px;
}

.question-answer {
  font-size: 14px;
  color: #334155;
  padding-top: 12px;
  border-top: 1px solid #e2e8f0;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal {
  background-color: white;
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid #e2e8f0;
}

.modal-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1e293b;
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid #e2e8f0;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #334155;
}

.form-control {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 14px;
}

textarea.form-control {
  resize: vertical;
}

.option-input {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.checkbox-group,
.radio-group {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

/* Button styles */
.btn-primary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-primary:hover {
  background-color: #2563eb;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  background-color: #f1f5f9;
  color: #334155;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-secondary:hover {
  background-color: #e2e8f0;
}

.btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background-color: transparent;
  color: #64748b;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-icon:hover {
  background-color: #f1f5f9;
  color: #334155;
}

.btn-text {
  display: inline-flex;
  align-items: center;
  padding: 6px 12px;
  background-color: transparent;
  color: #3b82f6;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-text:hover {
  background-color: #eff6ff;
}

.mr-1 {
  margin-right: 4px;
}
</style>
