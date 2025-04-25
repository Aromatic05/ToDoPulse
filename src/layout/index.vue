<template>
    <v-layout :class="{ 'mini_nav': rail }">
        <Sidebar @update:rail="(val: boolean) => rail = val" :rail="rail" />
        <main class="app_main">
            <Topbar @toggle-settings="toggleSettings" />
            
            <!-- 内联设置面板 -->
            <div v-if="showSettings" class="settings-container">
                <SettingsPanel @close="showSettings = false" />
            </div>
            
            <!-- 其他内容会在这里 -->
            <div v-else class="content-area">
                <!-- 应用主要内容 -->
            </div>
        </main>
    </v-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from '@/layout/Sidebar.vue'
import Topbar from '@/layout/Topbar.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'

const rail = ref(false)
const showSettings = ref(false)

function toggleSettings() {
    showSettings.value = !showSettings.value
}
</script>

<style>
@import "@/styles/layout.css";

/* 设置面板容器样式 */
.settings-container {
    background: rgba(var(--header-bg), 0.8);
    backdrop-filter: blur(10px);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 16px;
    box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
    animation: fadeIn 0.3s ease-in-out;
}

.content-area {
    min-height: 200px;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(-10px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>