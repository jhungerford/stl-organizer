import { createApp } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import App from './App.vue'

invoke('sample_command')
createApp(App).mount('#app')
