<template>
    <div class="settings-panel">
        <div class="settings-header">
            <h2 class="text-h5">设置</h2>
            <v-spacer></v-spacer>
            <v-btn icon variant="text" @click="$emit('close')">
                <v-icon>mdi-close</v-icon>
            </v-btn>
        </div>

        <div class="settings-content">
            <v-row>
                <v-col cols="12" md="4">
                    <div class="pa-4">
                        <h3 class="text-h6 mb-2">界面设置</h3>
                        
                        <v-switch
                            v-model="darkMode"
                            label="深色模式"
                            color="primary"
                            hide-details
                            class="mb-4"
                        ></v-switch>
                        
                        <v-switch
                            v-model="compactMode"
                            label="紧凑模式"
                            color="primary"
                            hide-details
                            class="mb-4"
                        ></v-switch>
                    </div>
                </v-col>
                
                <v-col cols="12" md="4">
                    <div class="pa-4">
                        <h3 class="text-h6 mb-2">个性化</h3>
                        
                        <v-select
                            v-model="selectedTheme"
                            :items="themes"
                            label="主题色"
                            variant="outlined"
                            density="compact"
                            class="mb-4"
                        ></v-select>
                        
                        <v-select
                            v-model="selectedLanguage"
                            :items="languages"
                            label="语言"
                            variant="outlined"
                            density="compact"
                            class="mb-4"
                        ></v-select>
                    </div>
                </v-col>
                
                <v-col cols="12" md="4">
                    <div class="pa-4">
                        <h3 class="text-h6 mb-2">通知设置</h3>
                        
                        <v-switch
                            v-model="emailNotifications"
                            label="邮件通知"
                            color="primary"
                            hide-details
                            class="mb-4"
                        ></v-switch>
                        
                        <v-switch
                            v-model="desktopNotifications"
                            label="桌面通知"
                            color="primary"
                            hide-details
                            class="mb-4"
                        ></v-switch>
                    </div>
                </v-col>
            </v-row>
        </div>
        
        <div class="settings-footer">
            <v-spacer></v-spacer>
            <v-btn color="primary" variant="elevated" @click="saveSettings">
                保存设置
            </v-btn>
            <v-btn variant="text" class="ml-2" @click="$emit('close')">
                取消
            </v-btn>
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
    
    // 关闭设置面板
    emit('close');
};

const emit = defineEmits(['close']);
</script>

<style scoped>
.settings-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.settings-header {
    display: flex;
    align-items: center;
    padding: 16px 24px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

.settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
}

.settings-footer {
    display: flex;
    padding: 16px 24px;
    border-top: 1px solid rgba(0, 0, 0, 0.1);
}
</style>