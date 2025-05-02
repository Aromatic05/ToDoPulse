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

                    <!-- 添加导出设置卡片 -->
                    <v-col cols="12" md="6">
                        <v-card class="pa-4">
                            <h3 class="text-h6 mb-5">数据导出</h3>

                            <v-row>
                                <v-col cols="12">
                                    <v-select v-model="selectedExportFormat" :items="exportFormats" label="导出格式"
                                        variant="outlined" density="compact" class="mb-4"></v-select>
                                </v-col>
                                
                                <!-- 添加导出路径选择 -->
                                <v-col cols="12" class="mb-2">
                                    <v-text-field
                                        v-model="exportPath"
                                        label="导出路径"
                                        readonly
                                        density="compact"
                                        variant="outlined"
                                        :append-inner-icon="'mdi-folder-open'"
                                        @click:append-inner="selectExportPath"
                                    ></v-text-field>
                                </v-col>

                                <v-col cols="12" md="6">
                                    <v-btn block color="primary" @click="exportAllEvents" :loading="exporting"
                                        class="mb-3">
                                        导出全部事件
                                    </v-btn>
                                </v-col>

                                <v-col cols="12" md="6">
                                    <v-btn block outlined @click="showExportDialog = true" :disabled="exporting"
                                        class="mb-3">
                                        选择事件导出
                                    </v-btn>
                                </v-col>
                            </v-row>

                            <v-alert v-if="exportResult" :color="exportResult.success ? 'success' : 'error'"
                                :title="exportResult.success ? '导出成功' : '导出失败'" :text="exportResult.message"
                                class="mt-3" variant="tonal" closable @click:close="exportResult = null"></v-alert>
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

        <!-- 事件选择对话框 -->
        <v-dialog v-model="showExportDialog" max-width="600">
            <v-card>
                <v-card-title class="text-h5">选择要导出的事件</v-card-title>
                <v-card-text>
                    <v-data-table v-model="selectedEvents" :headers="eventHeaders" :items="eventList" show-select
                        item-value="id">
                    </v-data-table>
                </v-card-text>
                <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn color="grey-darken-1" variant="text" @click="showExportDialog = false">
                        取消
                    </v-btn>
                    <v-btn color="primary" variant="text" @click="exportSelectedEvents"
                        :disabled="selectedEvents.length === 0">
                        导出所选事件
                    </v-btn>
                </v-card-actions>
            </v-card>
        </v-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { SettingService } from '@/services/SettingService';
// 修改这一行，使用正确的导入语句
import { open } from '@tauri-apps/plugin-dialog';
// 或者使用这种导入方式
// import { dialog } from '@tauri-apps/api';

// 设置状态
const darkMode = ref(false);
const compactMode = ref(false);
const selectedTheme = ref('default');
const selectedLanguage = ref('zh-CN');
const emailNotifications = ref(true);
const desktopNotifications = ref(true);

// 导出设置
const selectedExportFormat = ref('ics');
const exportFormats = [
    { title: 'iCalendar (.ics)', value: 'ics' },
    { title: 'JSON (.json)', value: 'json' },
    { title: 'Markdown (.md)', value: 'markdown' }
];
const exportPath = ref(''); // 新增导出路径变量
const exporting = ref(false);
const exportResult = ref(null);
const showExportDialog = ref(false);
const selectedEvents = ref([]);
const eventList = ref([]);
const eventHeaders = [
    { title: '标题', key: 'title' },
    { title: '创建日期', key: 'create' },
    { title: '状态', key: 'status', sortable: false }
];

// 选项
const themes = ['default', 'blue', 'green', 'purple', 'orange'];
const languages = ['zh-CN', 'en-US', 'ja-JP', 'ko-KR'];

// 初始化
onMounted(async () => {
    // 加载可导出的事件列表
    try {
        eventList.value = await SettingService.getExportableEvents();
        // 获取默认导出路径
        exportPath.value = await SettingService.getDefaultExportPath();
    } catch (error) {
        console.error('初始化失败', error);
    }
});

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

// 选择导出路径
const selectExportPath = async () => {
    try {
        // 使用 Tauri 对话框打开目录选择器
        const selected = await open({
            directory: true,
            multiple: false,
            title: '选择导出目录'
        });
        
        if (selected !== null) {
            exportPath.value = selected;
        }
    } catch (error) {
        console.error('选择路径失败', error);
    }
};

// 导出所有事件
const exportAllEvents = async () => {
    exporting.value = true;
    try {
        const result = await SettingService.exportAllEvents(
            selectedExportFormat.value, 
            exportPath.value // 传递选择的路径
        );
        exportResult.value = {
            success: true,
            message: `成功导出到文件: ${result}`
        };
    } catch (error) {
        exportResult.value = {
            success: false,
            message: `导出失败: ${error.toString()}`
        };
    } finally {
        exporting.value = false;
    }
};

// 导出选定事件
const exportSelectedEvents = async () => {
    if (selectedEvents.value.length === 0) return;

    exporting.value = true;
    showExportDialog.value = false;

    try {
        const eventIds = selectedEvents.value.map(event => event.id);
        const result = await SettingService.exportEvents(
            eventIds, 
            selectedExportFormat.value,
            exportPath.value // 传递选择的路径
        );
        exportResult.value = {
            success: true,
            message: `成功导出到文件: ${result}`
        };
    } catch (error) {
        exportResult.value = {
            success: false,
            message: `导出失败: ${error.toString()}`
        };
    } finally {
        exporting.value = false;
    }
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