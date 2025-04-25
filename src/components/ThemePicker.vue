<template>
    <v-menu location="bottom" :close-on-content-click="false" offset="10">
        <template v-slot:activator="{ props }">
            <v-btn v-bind="props" density="comfortable" variant="text" icon="mdi-palette"
                color="var(--md-sys-color-primary)">
            </v-btn>
        </template>

        <v-card min-width="200" class="pa-2" color="var(--md-sys-color-surface-container)">
            <v-list density="compact" bg-color="var(--md-sys-color-surface-container)">
                <v-list-item @click="toggleDarkMode"
                    :prepend-icon="isDarkMode ? 'mdi-white-balance-sunny' : 'mdi-moon-waning-crescent'" :title="''"
                    :tooltip="isDarkMode ? '切换到亮色模式' : '切换到暗色模式'">
                </v-list-item>
                <v-divider class="my-2"></v-divider>
                <v-list-item v-for="theme in availableThemes" :key="theme.name" @click="changeTheme(theme.name)"
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
import { ref, onMounted } from 'vue';
import {
    getThemes,
    loadThemePreference,
    saveThemePreference,
    getThemeClassName,
    getSystemThemePreference
} from '@/services/ThemeService';

// 定义类型接口
interface Theme {
    name: string;
    light: string;
    dark: string;
    color: string;
}

interface ThemePreference {
    themeName: string;
    isDarkMode: boolean;
}

// 使用TypeScript的类型注解
const isDarkMode = ref<boolean>(false);
const currentTheme = ref<string>('');
const availableThemes = getThemes() as Theme[];

/**
 * 切换深色/浅色模式
 */
function toggleDarkMode(): void {
    isDarkMode.value = !isDarkMode.value;
    applyTheme();
    // 保存偏好设置
    saveThemePreference(currentTheme.value, isDarkMode.value);
}

/**
 * 更改主题
 * @param themeName - 主题名称
 */
function changeTheme(themeName: string): void {
    currentTheme.value = themeName;
    applyTheme();
    // 保存偏好设置
    saveThemePreference(currentTheme.value, isDarkMode.value);
}

/**
 * 应用当前选择的主题
 */
function applyTheme(): void {
    // 先移除所有主题相关的类
    document.body.classList.forEach((className: string) => {
        if (className === 'dark' || className.endsWith('-light') || className.endsWith('-dark')) {
            document.body.classList.remove(className);
        }
    });

    // 应用当前主题
    const themeClassName: string = getThemeClassName(currentTheme.value, isDarkMode.value);
    document.body.classList.add(themeClassName);

    // 确保图标颜色更新
    document.documentElement.style.setProperty(
        '--icon-base-color',
        isDarkMode.value ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.6)'
    );
}

// 在组件挂载时初始化主题
onMounted(() => {
    const preference: ThemePreference = loadThemePreference();
    currentTheme.value = preference.themeName;
    isDarkMode.value = preference.isDarkMode;
    applyTheme();
});

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