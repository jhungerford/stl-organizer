<template>
  <h1>Directories:</h1>
  <div v-if="loading" class="loading">
    Loading...
  </div>
  <div v-if="error" class="error">
    {{ error }}
  </div>
  <div v-if="dirs" class="content">
    <ul>
      <li v-for="dir in dirs" v-bind:key="dir">
        {{ dir }}
      </li>
    </ul>
  </div>
</template>

<style>
</style>

<script>
import { invoke } from '@tauri-apps/api/tauri'

export default {
  data() {
    return {
      loading: false,
      dirs: null,
      error: null
    }
  },

  created() {
    // Fetch settings data once the view is created
    this.fetchData()
  },

  watch: {
    // Fetch data again if the route changes
    '$route': 'fetchData'
  },

  methods: {
    fetchData() {
      this.error = this.dirs = null;
      this.loading = true;

      invoke('list_dirs')
          .then((dirs) => {
            this.loading = false;
            this.dirs = dirs;
          })
          .catch((error) => {
            this.loading = false;
            this.error = error;
          })
    }
  }
}
</script>