import type { RouteRecordRaw } from 'vue-router'
import { createRouter, createWebHistory } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../views/Home.vue'),
  },
  {
    path: '/question-bank',
    name: 'QuestionBank',
    component: () => import('../views/QuestionBank.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
