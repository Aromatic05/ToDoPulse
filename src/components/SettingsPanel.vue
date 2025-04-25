<template>
    <div class="settings-view">
        <div class="settings-panel">
            <div class="settings-content">
                <v-row>
                    <v-col cols="12" md="4">
                        <v-card class="pa-4">
                            <h3 class="text-h6 mb-5">界面设置</h3>

                            <v-switch v-model="darkMode" label="深色模式" color="primary" hide-details
                                class="mb-4"></v-switch>

                            <v-switch v-model="compactMode" label="紧凑模式" color="primary" hide-details
                                class="mb-4"></v-switch>
                        </v-card>
                    </v-col>

                    <v-col cols="12" md="4">
                        <v-card class="pa-4">
                            <h3 class="text-h6 mb-5">个性化</h3>

                            <v-select v-model="selectedTheme" :items="themes" label="主题色" variant="outlined"
                                density="compact" class="mb-4"></v-select>

                            <v-select v-model="selectedLanguage" :items="languages" label="语言" variant="outlined"
                                density="compact" class="mb-4"></v-select>
                        </v-card>
                    </v-col>

                    <v-col cols="12" md="4">
                        <v-card class="pa-4">
                            <h3 class="text-h6 mb-5">通知设置</h3>

                            <v-switch v-model="emailNotifications" label="邮件通知" color="primary" hide-details
                                class="mb-4"></v-switch>

                            <v-switch v-model="desktopNotifications" label="桌面通知" color="primary" hide-details
                                class="mb-4"></v-switch>
                        </v-card>
                    </v-col>
                </v-row>
            </div>

            <div class="settings-footer mt-6">
                <v-btn color="primary" size="large" variant="elevated" @click="saveSettings">
                    保存设置
                </v-btn>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

// 设置状态
const darkMode = ref(false);
const compactMode = ref(false);
const selectedTheme = ref('default');
const selectedLanguage = ref('zh-CN');
const emailNotifications = ref(true);
const desktopNotifications = ref(true);

// 选项
const themes = ['default', 'blue', 'green', 'purple', 'orange'];
const languages = ['zh-CN', 'en-US', 'ja-JP', 'ko-KR'];

// 保存设置
const saveSettings = () => {
    // 在这里实现保存设置的逻辑
    console.log('保存设置:', {
        darkMode: darkMode.value,
        compactMode: compactMode.value,
        selectedTheme: selectedTheme.value,
        selectedLanguage: selectedLanguage.value,
        emailNotifications: emailNotifications.value,
        desktopNotifications: desktopNotifications.value
    });
};
</script>

<style scoped>
.settings-view {
    width: 100%;
    padding: 16px;
}

.settings-panel {
    max-width: 1200px;
    margin: 0 auto;
}

.settings-content {
    margin-bottom: 30px;
}

.settings-footer {
    display: flex;
    justify-content: center;
    margin-top: 20px;
}

h3 {
    font-weight: 500;
    color: var(--md-sys-color-on-surface-variant);
}
</style>