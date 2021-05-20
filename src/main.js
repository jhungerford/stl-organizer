import { createApp } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import App from './App.vue'
import { Quasar } from 'quasar'
import quasarUserOptions from './quasar-user-options'

invoke('sample_command')
createApp(App).use(Quasar, quasarUserOptions).mount('#app')
