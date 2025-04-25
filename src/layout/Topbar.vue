<template>
    <v-card class="header">
        <!-- 添加回Breadcrumbs组件 -->
        <div class="header_title ml-4">
            <Breadcrumbs />
        </div>
        <v-spacer></v-spacer>
        <div style="width: 320px" class="search_ip mr-2">
            <v-text-field rounded variant="outlined" density="compact" label="Search" prepend-inner-icon="mdi-magnify"
                hide-details clearable bg-color="var(--md-sys-color-surface-container)"
                color="var(--md-sys-color-primary)"></v-text-field>
        </div>
        <div class="tool_btns">
            <ThemePicker />

            <v-btn density="comfortable" variant="text" icon="mdi-cog" @click="toggleSettings"
                :color="isSettingsActive ? 'var(--md-sys-color-primary)' : 'var(--md-sys-color-on-surface-variant)'">
            </v-btn>
            
        </div>
    </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Breadcrumbs from '@/components/Breadcrumbs.vue';
import SettingsPanel from '@/components/SettingsPanel.vue';
import ThemePicker from '@/components/ThemePicker.vue';

const emit = defineEmits(['toggle-settings']);
const isSettingsActive = ref(false);

function toggleSettings() {
    isSettingsActive.value = !isSettingsActive.value;
    emit('toggle-settings');
}

// 添加暗色模式切换函数
function toggleDarkMode() {
    const themePickerInstance = document.querySelector('theme-picker')?.__vueParentComponent?.ctx;
    if (themePickerInstance && typeof themePickerInstance.toggleDarkMode === 'function') {
        themePickerInstance.toggleDarkMode();
    }
}
</script>

<style>
/* 移除了颜色点样式，因为它已经在ThemePicker组件中定义 */
</style>