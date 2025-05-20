// 主题管理服务

// 定义类型接口
export interface Theme {
    name: string;
    light: string;
    dark: string;
    color: string;
}

export interface ThemePreference {
    themeName: string;
    isDarkMode: boolean;
}

interface ThemeColorMap {
    [themeName: string]: string;
}

// 主题列表
const themes: Theme[] = [
    { name: "Default", light: "default-light", dark: "dark", color: "#FFFFFF" },
    { name: "Red", light: "red-light", dark: "red-dark", color: "#F44336" },
    { name: "Yellow", light: "yellow-light", dark: "yellow-dark", color: "#FFC107" },
    { name: "Ivory", light: "ivory-light", dark: "ivory-dark", color: "#AA9B6E" },
    { name: "Green", light: "green-light", dark: "green-dark", color: "#4CAF50" },
    { name: "SkyBlue", light: "skyblue-light", dark: "skyblue-dark", color: "#1478DC" },
    { name: "Blue", light: "blue-light", dark: "blue-dark", color: "#2196F3" },
    { name: "Purple", light: "purple-light", dark: "purple-dark", color: "#9C27B0" },
];

// 主题颜色映射，便于快速访问
const themeColors: ThemeColorMap = themes.reduce<ThemeColorMap>((acc, theme) => {
    acc[theme.name] = theme.color;
    return acc;
}, {});

/**
 * 获取系统主题偏好（深色/浅色）
 * @returns 是否为深色主题
 */
export function getSystemThemePreference(): boolean {
    return window.matchMedia?.('(prefers-color-scheme: dark)')?.matches ?? false;
}

/**
 * 从本地存储加载主题偏好
 * @returns 包含主题名称和是否为深色模式的对象
 */
export function loadThemePreference(): ThemePreference {
    const storedTheme = localStorage.getItem('theme-name') || themes[0].name;
    const storedPreference = localStorage.getItem('dark-mode');
    const isDarkMode = storedPreference !== null ? storedPreference === 'true' : getSystemThemePreference();

    return {
        themeName: storedTheme,
        isDarkMode
    };
}

/**
 * 保存主题偏好到本地存储
 * @param themeName - 主题名称
 * @param isDarkMode - 是否使用深色主题
 */
export function saveThemePreference(themeName: string, isDarkMode: boolean): void {
    localStorage.setItem('theme-name', themeName);
    localStorage.setItem('dark-mode', String(isDarkMode));
}

/**
 * 获取当前主题的类名
 * @param themeName - 主题名称
 * @param isDarkMode - 是否使用深色主题
 * @returns 主题类名
 */
export function getThemeClassName(themeName: string, isDarkMode: boolean): string {
    const theme = themes.find(t => t.name === themeName);
    if (!theme) return themes[0].light;
    return isDarkMode ? theme.dark : theme.light;
}

/**
 * 获取主题对应的颜色值
 * @param themeName - 主题名称
 * @returns 主题颜色值
 */
export function getThemeColor(themeName: string): string {
    return themeColors[themeName] || themeColors.Green;
}

/**
 * 获取所有可用主题
 * @returns 主题列表
 */
export function getThemes(): Theme[] {
    return themes;
}

export default {
    themes,
    themeColors,
    getSystemThemePreference,
    loadThemePreference,
    saveThemePreference,
    getThemeClassName,
    getThemeColor,
    getThemes
};