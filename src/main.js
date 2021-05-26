import { createApp } from 'vue'
import App from './App.vue'
import { Quasar } from 'quasar'
import { invoke } from '@tauri-apps/api/tauri'
import quasarUserOptions from './quasar-user-options'


invoke('add_dir', {'dir': '~/sample-js'})
createApp(App).use(Quasar, quasarUserOptions).mount('#app')
