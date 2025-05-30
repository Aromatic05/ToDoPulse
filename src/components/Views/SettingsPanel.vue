<template>
    <div class="settings-view">
        <div class="settings-panel">
            <div class="settings-content">
                <v-container>
                    <v-row justify="center">
                        <v-col cols="12" lg="8" md="10">
                            <v-card class="pa-4">
                                <v-expansion-panels v-model="expandedPanels" multiple variant="accordion"
                                    class="settings-panels" expand-icon="mdi-chevron-down">
                                    <Notify ref="notifyRef" />
                                    <Aigc ref="aigcRef" />
                                    <Export ref="exportCardRef" :settings="exportSettings" @export-all="exportAllEvents"
                                        @export-filtered="exportFilteredEvents"
                                        @show-selection-dialog="exportSettings.showDialog = true"
                                        @update:settings="handleExportSettingsUpdate" />
                                    <WebDav ref="webdavCardRef" @test-connection="handleTestWebDAVConnection"
                                        @sync="handleSyncWithWebDAV"
                                        @update:webdavResult="WebDavSettings.result = $event" />
                                </v-expansion-panels>
                            </v-card>
                        </v-col>
                    </v-row>
                </v-container>
            </div>
        </div>

        <!-- 事件选择对话框 -->
        <ExportChat v-model="exportSettings.showDialog" :items="Lists" @export-selected="exportSelectedLists" />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, watch, onUnmounted } from "vue";
import { SettingService } from "@/services/SettingService";
// biome-ignore lint/style/useImportType: bug
import Notify from "@/components/Settings/Notify.vue";
// biome-ignore lint/style/useImportType: bug
import Export from "@/components/Settings/Export.vue";
// biome-ignore lint/style/useImportType: bug
import WebDav from "@/components/Settings/WebDav.vue";
import ExportChat from "@/components/Settings/ExportChat.vue";
// biome-ignore lint/style/useImportType: bug
import Aigc from "../Settings/Aigc.vue";

// 组件引用
const notifyRef = ref<InstanceType<typeof Notify> | null>(null);
const aigcRef = ref<InstanceType<typeof Aigc> | null>(null);
const exportCardRef = ref<InstanceType<typeof Export> | null>(null);
const webdavCardRef = ref<InstanceType<typeof WebDav> | null>(null);

// 控制面板展开状态 - 默认全部收起
const expandedPanels = ref<number[]>([]);

// 简化的watch监听器 - 移除折叠时的保存逻辑
watch(
    expandedPanels,
    (newVal, oldVal) => {
        console.log("面板状态变化:", newVal, oldVal);
        // 初始化检查
        if (!oldVal) {
            console.log("初始化，跳过当前变化");
            return;
        }

        // 防止重复处理
        if (JSON.stringify(newVal) === JSON.stringify(oldVal)) {
            console.log("面板状态未实际变化，跳过处理");
            return;
        }

        // 只记录状态变化，不触发保存操作
        const collapsedPanels = oldVal.filter((panel) => !newVal.includes(panel));
        if (collapsedPanels.length > 0) {
            console.log("检测到折叠的面板:", collapsedPanels);
        }

        // 处理展开的面板
        const expandedPanelsNew = newVal.filter((panel) => !oldVal.includes(panel));
        if (expandedPanelsNew.length > 0) {
            console.log("检测到展开的面板:", expandedPanelsNew);
        }
    },
    { deep: false }
);

// 与队列相关的代码已移除，因为我们改为子组件内自动保存

// 导出设置
const exportSettings = reactive({
    format: "ics",
    filter: "all",
    inProgress: false,
    result: null as { success: boolean; message: string } | null,
    showDialog: false,
});

const Lists = ref<any[]>([]);

// WebDAV 同步相关变量
const WebDavSettings = reactive({
    connectionTest: false,
    result: null as { success: boolean; message: string } | null,
});

onMounted(async () => {
    try {
        // 添加超时控制
        const loadPromise = SettingService.getExportableLists();
        const timeoutPromise = new Promise((_, reject) => {
            setTimeout(() => reject(new Error('加载超时')), 10000);
        });

        Lists.value = await Promise.race([loadPromise, timeoutPromise]) as any[];
    } catch (error) {
        console.error("初始化失败:", error);
        // 设置默认值或显示错误提示
        Lists.value = [];
    }
});

onUnmounted(() => {
    // 清理相关资源
    console.log('设置面板已卸载');
    // 注意：子组件会在它们各自的 onUnmounted 钩子中清理自己
});

// display actions

const handleExportSettingsUpdate = (update: Partial<typeof exportSettings>) => {
    Object.assign(exportSettings, update);
};

//functional actions

// 选择导出文件保存位置
const selectSavePathForExport = async (filename: string, format: string) => {
    try {
        // 使用后端API选择文件保存位置
        const selectedPath = await SettingService.selectSavePath(filename, format);
        if (selectedPath) {
            return selectedPath;
        }
        return null;
    } catch (error) {
        console.error("选择保存路径失败", error);
        return null;
    }
};

// 导出所有事件
const exportAllEvents = async () => {
    try {
        // 先让用户选择保存位置
        const filename = "all_todopulse_events";
        const format = exportSettings.format;
        const customPath = await selectSavePathForExport(filename, format);

        if (customPath) {
            // 用户选择了保存位置
            const result = await SettingService.exportAllEvents(format, customPath);
            exportSettings.result = {
                success: true,
                message: `成功导出到文件: ${result}`,
            };
        } else {
            // 用户取消选择
            exportSettings.result = {
                success: false,
                message: "未选择保存位置",
            };
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        exportSettings.result = {
            success: false,
            message: `导出失败: ${errorMessage || "未知错误"}`,
        };
    } finally {
        exportSettings.inProgress = false;
        if (exportCardRef.value) {
            exportCardRef.value.setExporting(false);
            exportCardRef.value.setResult(exportSettings.result);
        }
    }
};

// 导出选定事件
const exportSelectedLists = async (selectedLists: any[]) => {
    if (selectedLists.length === 0) return;
    exportSettings.inProgress = true;
    if (exportCardRef.value) {
        exportCardRef.value.setExporting(true);
    }

    try {
        // 先让用户选择保存位置
        const filename = "selected_events";
        const format = exportSettings.format;
        const customPath = await selectSavePathForExport(filename, format);

        if (customPath) {
            const selectedListIds = selectedLists.map((list: any) => list);
            const result = await SettingService.exportLists(
                selectedListIds,
                exportSettings.format,
                customPath
            );
            exportSettings.result = {
                success: true,
                message: `成功导出到文件: ${result}`,
            };
        } else {
            // 用户取消选择
            exportSettings.result = {
                success: false,
                message: "未选择保存位置",
            };
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        exportSettings.result = {
            success: false,
            message: `导出失败: ${errorMessage || "未知错误"}`,
        };
    } finally {
        exportSettings.inProgress = false;
        if (exportCardRef.value) {
            exportCardRef.value.setExporting(false);
            exportCardRef.value.setResult(exportSettings.result);
        }
    }
};

// 导出按筛选条件过滤的事件
const exportFilteredEvents = async () => {
    try {
        // 确定文件名和格式
        let filename: string;
        switch (exportSettings.filter) {
            case "completed":
                filename = "completed_events";
                break;
            case "pending":
                filename = "pending_events";
                break;
            default:
                filename = "all_events";
        }

        const format = exportSettings.format;
        const customPath = await selectSavePathForExport(filename, format);

        if (customPath) {
            let result: string;
            // 使用用户选择的路径
            const savePath = customPath;

            switch (exportSettings.filter) {
                case "completed":
                    result = await SettingService.exportEventsByStatus(
                        true,
                        exportSettings.format,
                        savePath
                    );
                    break;
                case "pending":
                    result = await SettingService.exportEventsByStatus(
                        false,
                        exportSettings.format,
                        savePath
                    );
                    break;
                default:
                    // 默认导出全部
                    result = await SettingService.exportAllEvents(
                        exportSettings.format,
                        savePath
                    );
            }

            exportSettings.result = {
                success: true,
                message: `成功导出到文件: ${result}`,
            };
        } else {
            // 用户取消选择
            exportSettings.result = {
                success: false,
                message: "未选择保存位置",
            };
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        exportSettings.result = {
            success: false,
            message: `导出失败: ${errorMessage || "未知错误"}`,
        };
    } finally {
        exportSettings.inProgress = false;
        if (exportCardRef.value) {
            exportCardRef.value.setExporting(false);
            exportCardRef.value.setResult(exportSettings.result);
        }
    }
};

// WebDAV 连接测试
const handleTestWebDAVConnection = async (data: {
    host: string;
    username: string;
    password: string;
}) => {
    try {
        const result = await SettingService.testWebDAVConnection(
            data.host,
            data.username,
            data.password
        );

        if (result) {
            WebDavSettings.result = {
                success: true,
                message: "WebDAV 连接测试成功",
            };
            WebDavSettings.connectionTest = true;
        } else {
            WebDavSettings.result = {
                success: false,
                message: "WebDAV 连接测试失败，请检查服务器地址和凭据",
            };
            WebDavSettings.connectionTest = false;
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        WebDavSettings.result = {
            success: false,
            message: `连接错误: ${errorMessage || "未知错误"}`,
        };
        WebDavSettings.connectionTest = false;
    } finally {
        if (webdavCardRef.value) {
            webdavCardRef.value.setTestingStatus(false);
            webdavCardRef.value.setConnectionTested(WebDavSettings.connectionTest);
            webdavCardRef.value.setResult(WebDavSettings.result);
        }
    }
};

// WebDAV 同步
const handleSyncWithWebDAV = async (data: {
    host: string;
    username: string;
    password: string;
    localDir: string;
    remoteDir: string;
}) => {
    try {
        const result = await SettingService.syncDirectoryWithWebDAV();

        if (result) {
            WebDavSettings.result = {
                success: true,
                message: "目录同步成功",
            };
        } else {
            WebDavSettings.result = {
                success: false,
                message: "目录同步失败",
            };
        }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        WebDavSettings.result = {
            success: false,
            message: `同步错误: ${errorMessage || "未知错误"}`,
        };
    } finally {
        if (webdavCardRef.value) {
            webdavCardRef.value.setSyncingStatus(false);
            webdavCardRef.value.setResult(WebDavSettings.result);
        }
    }
};
</script>

<style scoped>
.settings-view {
    width: 100%;
    padding: 16px;
}

.settings-panel {
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

:deep(.v-expansion-panel-title) {
    padding: 16px;
    min-height: 64px;
}

:deep(.v-expansion-panel) {
    margin-bottom: 12px;
    border-radius: 8px;
    overflow: hidden;
    transition: box-shadow 0.3s ease;
}

:deep(.v-expansion-panel:hover) {
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.08);
}

:deep(.v-card) {
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

:deep(.settings-panels) {
    background: transparent;
}
</style>
