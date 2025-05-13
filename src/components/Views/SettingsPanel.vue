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
                                <v-col cols="12" md="6">
                                    <v-select v-model="selectedExportFormat" :items="exportFormats" label="导出格式"
                                        variant="outlined" density="compact" class="mb-4"></v-select>
                                </v-col>
                                <v-col cols="12" md="6">
                                    <v-select v-model="exportFilter" :items="exportFilterOptions" label="导出筛选"
                                        variant="outlined" density="compact" class="mb-4"></v-select>
                                </v-col>
                                
                                <!-- 导出路径选择已移除 -->

                                <v-col cols="12" md="4">
                                    <v-btn block color="primary" @click="exportAllEvents" :loading="exporting"
                                        class="mb-3">
                                        导出全部事件
                                    </v-btn>
                                </v-col>
                                <v-col cols="12" md="4">
                                    <v-btn block color="info" @click="exportFilteredEvents" :loading="exporting"
                                        class="mb-3">
                                        按筛选导出
                                    </v-btn>
                                </v-col>

                                <v-col cols="12" md="4">
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

                    <!-- 添加 WebDAV 同步设置卡片 -->
                    <v-col cols="12" md="6">
                        <v-card class="pa-4">
                            <h3 class="text-h6 mb-5">WebDAV 同步</h3>
                            <v-form @submit.prevent="testWebDAVConnection">
                                <v-row>
                                    <v-col cols="12">
                                        <v-text-field
                                            v-model="webdavHost"
                                            label="WebDAV 服务器地址"
                                            placeholder="https://example.com/dav/"
                                            variant="outlined"
                                            density="compact"
                                            class="mb-3"
                                            required
                                        ></v-text-field>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-text-field
                                            v-model="webdavUsername"
                                            label="用户名"
                                            variant="outlined"
                                            density="compact"
                                            class="mb-3"
                                            required
                                        ></v-text-field>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-text-field
                                            v-model="webdavPassword"
                                            label="密码"
                                            type="password"
                                            variant="outlined"
                                            density="compact"
                                            class="mb-3"
                                            required
                                        ></v-text-field>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-text-field
                                            v-model="webdavLocalDir"
                                            label="本地目录"
                                            variant="outlined"
                                            density="compact"
                                            class="mb-3"
                                            required
                                        ></v-text-field>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-text-field
                                            v-model="webdavRemoteDir"
                                            label="远程目录"
                                            placeholder="/remote/path/"
                                            variant="outlined"
                                            density="compact"
                                            class="mb-3"
                                            required
                                        ></v-text-field>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-btn
                                            block
                                            color="primary"
                                            @click="testWebDAVConnection"
                                            :loading="testingWebDAV"
                                            class="mb-3"
                                        >
                                            测试连接
                                        </v-btn>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-btn
                                            block
                                            color="success"
                                            @click="syncWithWebDAV"
                                            :loading="syncingWebDAV"
                                            :disabled="!webdavConnectionTested"
                                            class="mb-3"
                                        >
                                            开始同步
                                        </v-btn>
                                    </v-col>
                                </v-row>
                            </v-form>

                            <v-alert
                                v-if="webdavResult"
                                :color="webdavResult.success ? 'success' : 'error'"
                                :title="webdavResult.success ? '操作成功' : '操作失败'"
                                :text="webdavResult.message"
                                class="mt-3"
                                variant="tonal"
                                closable
                                @click:close="webdavResult = null"
                            ></v-alert>
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
                    <v-data-table v-model="selectedLists" :headers="eventHeaders" :items="Lists" show-select
                        item-value="id">
                    </v-data-table>
                </v-card-text>
                <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn color="grey-darken-1" variant="text" @click="showExportDialog = false">
                        取消
                    </v-btn>
                    <v-btn color="primary" variant="text" @click="exportSelectedLists"
                        :disabled="selectedLists.length === 0">
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
    { title: 'Markdown (.md)', value: 'md' }
];
const exportFilter = ref('all');
const exportFilterOptions = [
    { title: '全部事件', value: 'all' },
    { title: '已完成事件', value: 'completed' },
    { title: '未完成事件', value: 'pending' }
];
// 导出路径变量已移除
const exporting = ref(false);
const exportResult = ref<{success: boolean; message: string} | null>(null);
const showExportDialog = ref(false);
const selectedLists = ref<any[]>([]);
const Lists = ref<any[]>([]);
const eventHeaders = [
    { title: '标题', key: 'title' },
];

// WebDAV 同步相关变量
const webdavHost = ref('');
const webdavUsername = ref('');
const webdavPassword = ref('');
const webdavLocalDir = ref('');
const webdavRemoteDir = ref('/');
const testingWebDAV = ref(false);
const syncingWebDAV = ref(false);
const webdavConnectionTested = ref(false);
const webdavResult = ref<{success: boolean; message: string} | null>(null);

// 选项
const themes = ['default', 'blue', 'green', 'purple', 'orange'];
const languages = ['zh-CN', 'en-US', 'ja-JP', 'ko-KR'];

// 初始化
onMounted(async () => {
    // 加载可导出的事件列表
    try {
        Lists.value = await SettingService.getExportableLists();
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

// 选择导出路径函数已移除

// 选择导出文件保存位置
const selectSavePathForExport = async (filename: string, format: string) => {
    try {
        // 使用新增的后端API选择文件保存位置
        const selectedPath = await SettingService.selectSavePath(filename, format);
        if (selectedPath) {
            return selectedPath;
        }
        return null;
    } catch (error) {
        console.error('选择保存路径失败', error);
        return null;
    }
};

// 导出所有事件
const exportAllEvents = async () => {
    exporting.value = true;
    try {
        // 先让用户选择保存位置
        const filename = 'all_todopulse_events';
        const format = selectedExportFormat.value;
        const customPath = await selectSavePathForExport(filename, format);
        
        if (customPath) {
            // 用户选择了保存位置
            const result = await SettingService.exportAllEvents(
                format,
                customPath // 使用用户选择的保存路径
            );
            exportResult.value = {
                success: true,
                message: `成功导出到文件: ${result}`
            };
        } else {
            // 用户取消选择且没有默认路径
            exportResult.value = {
                success: false,
                message: '未选择保存位置'
            };
        }
    } catch (error: any) {
        exportResult.value = {
            success: false,
            message: `导出失败: ${error?.toString() || '未知错误'}`
        };
    } finally {
        exporting.value = false;
    }
};

// 导出选定事件
const exportSelectedLists = async () => {
    if (selectedLists.value.length === 0) return;
    console.log('导出选定事件:', selectedLists.value);
    exporting.value = true;
    showExportDialog.value = false;

    try {
        // 先让用户选择保存位置
        const filename = 'selected_events';
        const format = selectedExportFormat.value;
        const customPath = await selectSavePathForExport(filename, format);
        
        if (customPath) {
            const selectedListIds = selectedLists.value.map((list: any) => list);
            const result = await SettingService.exportLists(
                selectedListIds,
                selectedExportFormat.value,
                customPath // 使用用户选择的路径
            );
            exportResult.value = {
                success: true,
                message: `成功导出到文件: ${result}`
            };
        } else {
            // 用户取消选择且没有默认路径
            exportResult.value = {
                success: false,
                message: '未选择保存位置'
            };
        }
    } catch (error: any) {
        exportResult.value = {
            success: false,
            message: `导出失败: ${error?.toString() || '未知错误'}`
        };
    } finally {
        exporting.value = false;
    }
};

// 导出按筛选条件过滤的事件
const exportFilteredEvents = async () => {
    exporting.value = true;
    try {
        // 确定文件名和格式
        let filename;
        switch (exportFilter.value) {
            case 'completed':
                filename = 'completed_events';
                break;
            case 'pending':
                filename = 'pending_events';
                break;
            default:
                filename = 'all_events';
        }
        
        const format = selectedExportFormat.value;
        const customPath = await selectSavePathForExport(filename, format);
        
        if (customPath) {
            let result;
            // 使用用户选择的路径
            const savePath = customPath;
            
            switch (exportFilter.value) {
                case 'completed':
                    result = await SettingService.exportEventsByStatus(
                        true,
                        selectedExportFormat.value,
                        savePath
                    );
                    break;
                case 'pending':
                    result = await SettingService.exportEventsByStatus(
                        false,
                        selectedExportFormat.value,
                        savePath
                    );
                    break;
                default:
                    // 默认导出全部
                    result = await SettingService.exportAllEvents(
                        selectedExportFormat.value,
                        savePath
                    );
            }
            
            exportResult.value = {
                success: true,
                message: `成功导出到文件: ${result}`
            };
        } else {
            // 用户取消选择且没有默认路径
            exportResult.value = {
                success: false,
                message: '未选择保存位置'
            };
        }
    } catch (error: any) {
        exportResult.value = {
            success: false,
            message: `导出失败: ${error?.toString() || '未知错误'}`
        };
    } finally {
        exporting.value = false;
    }
};

// WebDAV 连接测试
const testWebDAVConnection = async () => {
    if (!webdavHost.value || !webdavUsername.value || !webdavPassword.value) {
        webdavResult.value = {
            success: false,
            message: '请填写完整的 WebDAV 服务器信息'
        };
        return;
    }

    testingWebDAV.value = true;
    try {
        const result = await SettingService.testWebDAVConnection(
            webdavHost.value,
            webdavUsername.value,
            webdavPassword.value
        );

        if (result) {
            webdavResult.value = {
                success: true,
                message: 'WebDAV 连接测试成功'
            };
            webdavConnectionTested.value = true;
        } else {
            webdavResult.value = {
                success: false,
                message: 'WebDAV 连接测试失败，请检查服务器地址和凭据'
            };
            webdavConnectionTested.value = false;
        }
    } catch (error: any) {
        webdavResult.value = {
            success: false,
            message: `连接错误: ${error?.toString() || '未知错误'}`
        };
        webdavConnectionTested.value = false;
    } finally {
        testingWebDAV.value = false;
    }
};

// WebDAV 同步
const syncWithWebDAV = async () => {
    if (!webdavConnectionTested.value) {
        webdavResult.value = {
            success: false,
            message: '请先测试连接'
        };
        return;
    }

    if (!webdavLocalDir.value || !webdavRemoteDir.value) {
        webdavResult.value = {
            success: false,
            message: '请填写本地和远程目录'
        };
        return;
    }

    syncingWebDAV.value = true;
    try {
        const result = await SettingService.syncDirectoryWithWebDAV(
            webdavHost.value,
            webdavUsername.value,
            webdavPassword.value,
            webdavLocalDir.value,
            webdavRemoteDir.value
        );

        if (result) {
            webdavResult.value = {
                success: true,
                message: '目录同步成功'
            };
        } else {
            webdavResult.value = {
                success: false,
                message: '目录同步失败'
            };
        }
    } catch (error: any) {
        webdavResult.value = {
            success: false,
            message: `同步错误: ${error?.toString() || '未知错误'}`
        };
    } finally {
        syncingWebDAV.value = false;
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