import { createApp } from 'vue'
import { setupCalendar, Calendar } from 'v-calendar';
import 'v-calendar/style.css';
import App from './App.vue'

// Pinia
import { createPinia } from 'pinia'

// Vuetify
import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import '@mdi/font/css/materialdesignicons.css'
import '@/styles/vuetify-override.css'

const vuetify = createVuetify({
  components,
  directives,
})

// Create pinia
const pinia = createPinia()

// Create app
const app = createApp(App)

app.use(setupCalendar, {})
app.component('VCalendar', Calendar)
// app.component('VDatePicker', DatePicker)
app.use(vuetify)
app.use(pinia) // 使用Pinia
app.mount('#app')