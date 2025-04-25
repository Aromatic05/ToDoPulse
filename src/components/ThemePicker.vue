<template>
    <!-- 将整个组件包装在单一元素中，这样可以正确使用activator="parent" -->
    <v-menu location="bottom" :close-on-content-click="false" offset="10">
        <template v-slot:activator="{ props }">
            <v-btn v-bind="props" density="comfortable" variant="text" icon="mdi-palette"
                color="var(--md-sys-color-primary)">
            </v-btn>
        </template>

        <v-card min-width="200" class="pa-2" color="var(--md-sys-color-surface-container)">
            <v-list density="compact" bg-color="var(--md-sys-color-surface-container)">
                <v-list-subheader>选择主题</v-list-subheader>
                <v-list-item @click="toggleDarkMode" prepend-icon="mdi-theme-light-dark"
                    :title="isDarkMode ? '切换到亮色模式' : '切换到暗色模式'" />
                <v-divider class="my-2"></v-divider>
                <v-list-item v-for="theme in availableThemes" :key="theme.id" @click="changeTheme(theme.id)"
                    :title="theme.name">
                    <template v-slot:prepend>
                        <div class="color-dot" :style="{ backgroundColor: theme.color }"></div>
                    </template>
                </v-list-item>
            </v-list>
        </v-card>
    </v-menu>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const isDarkMode = ref(false);
const currentTheme = ref('');

const availableThemes = [
    { id: '', name: '默认蓝绿', color: '#006874' },
    { id: 'red', name: '红色', color: '#8F4C38' },
    { id: 'green', name: '绿色', color: '#4C662B' },
    { id: 'blue', name: '蓝色', color: '#415F91' },
    { id: 'skyblue', name: '天蓝', color: '#1478DC' },
    { id: 'purple', name: '紫色', color: '#6750A4' },
];

function toggleDarkMode() {
    isDarkMode.value = !isDarkMode.value;
    applyTheme();
}

function changeTheme(themeId) {
    currentTheme.value = themeId;
    applyTheme();
}

function applyTheme() {
    // 移除所有主题相关的类
    document.body.classList.forEach(className => {
        if (className === 'dark' ||
            availableThemes.some(theme =>
                className === `${theme.id}-light` || className === `${theme.id}-dark`)) {
            document.body.classList.remove(className);
        }
    });

    // 应用选中的主题
    if (currentTheme.value === '') {
        // 默认主题
        if (isDarkMode.value) {
            document.body.classList.add('dark');
        }
    } else {
        // 其他主题
        document.body.classList.add(isDarkMode.value
            ? `${currentTheme.value}-dark`
            : `${currentTheme.value}-light`);
    }

    // 确保图标颜色更新
    document.documentElement.style.setProperty(
        '--icon-base-color',
        isDarkMode.value ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.6)'
    );
}

// 在组件挂载时检查当前主题状态
function initTheme() {
    // 检查当前是否为暗色模式
    if (document.body.classList.contains('dark')) {
        isDarkMode.value = true;
    }

    // 检查当前是否使用特定颜色主题
    for (const theme of availableThemes) {
        if (document.body.classList.contains(`${theme.id}-light`) ||
            document.body.classList.contains(`${theme.id}-dark`)) {
            currentTheme.value = theme.id;
            break;
        }
    }
}

// 初始化主题
initTheme();

// 对外暴露方法以便父组件调用
defineExpose({
    toggleDarkMode
});
</script>

<style scoped>
.color-dot {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    margin-right: 8px;
}
</style>