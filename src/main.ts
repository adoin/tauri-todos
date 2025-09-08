import { createPinia } from 'pinia'
import { createApp } from 'vue'
import App from './App.vue'
import 'uno.css'
import '@unocss/reset/tailwind.css'
import 'element-plus/dist/index.css'
import './style.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
