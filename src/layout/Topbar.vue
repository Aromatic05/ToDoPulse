<template>
    <v-card class="header">
        <!-- 标题区域 -->
        <div class="header_title ml-4">
            <!-- 只在 ListView 中显示 Breadcrumbs -->
            <Breadcrumbs v-if="currentView && currentView.startsWith('list/')" :currentPageTitle="viewTitle" />

            <!-- 在其他视图中显示英文标题 -->
            <h2 v-else class="text-h6">{{ viewTitle }}</h2>
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
import { ref, computed } from 'vue';
import Breadcrumbs from '@/components/Breadcrumbs.vue';
import ThemePicker from '@/components/ThemePicker.vue';

// 定义视图名称的类型
type ViewName = 'timeline' | 'calendar' | 'tags' | 'settings' | 'default';

const props = defineProps({
    currentView: {
        type: String,
        default: 'timeline' as ViewName
    }
});

const emit = defineEmits(['toggle-settings']);
const isSettingsActive = ref(false);

// 视图名称映射到英文标题
const viewTitles: Record<ViewName, string> = {
    'timeline': 'Timeline',
    'calendar': 'Calendar',
    'tags': 'Tags',
    'settings': 'Settings',
    'default': 'Dashboard'
};

// 计算当前应显示的标题
const viewTitle = computed(() => {
    if (props.currentView.startsWith('list/')) {
        return 'List Details';
    }
    // 使用类型断言来确保 TypeScript 理解这是一个有效的键
    return viewTitles[(props.currentView as ViewName)] || viewTitles.default;
});

function toggleSettings() {
    isSettingsActive.value = !isSettingsActive.value;
    emit('toggle-settings');
}

// 添加暗色模式切换函数
// function toggleDarkMode() {
//     const themePickerInstance = document.querySelector('theme-picker')?.__vueParentComponent?.ctx;
//     if (themePickerInstance && typeof themePickerInstance.toggleDarkMode === 'function') {
//         themePickerInstance.toggleDarkMode();
//     }
// }
</script>

<style scoped>
.header_title {
    display: flex;
    align-items: center;
    min-width: 150px;
}

.header_title h2 {
    margin: 0;
    color: var(--md-sys-color-on-surface);
    font-weight: 600;
}
</style>