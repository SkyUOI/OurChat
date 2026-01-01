import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zh from './locales/zh.json'
import { createPinia } from 'pinia'

// Create i18n instance
const i18n = createI18n({
  legacy: false,
  locale: 'en', // Default language
  fallbackLocale: 'en', // Fallback language
  messages: {
    en,
    zh,
  },
})

const app = createApp(App)

// Globally register Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(ElementPlus)
app.use(i18n) // Use i18n plugin
app.mount('#app')
