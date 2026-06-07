import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import DefaultLayout from './layouts/DefaultLayout.vue'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'
import './styles/global.css'
import * as AntdIcons from '@ant-design/icons-vue'
import { useTheme } from './composables/useTheme'

// ensure theme attribute is set as early as possible (avoids FOUC)
useTheme()

const app = createApp(App)
app.use(router)
app.use(Antd)
for (const [key, component] of Object.entries(AntdIcons)) {
  app.component(key, component)
}
app.component('DefaultLayout', DefaultLayout)
app.mount('#app')
