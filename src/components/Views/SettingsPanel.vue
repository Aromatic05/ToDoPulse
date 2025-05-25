<template>
  <div class="settings-view">
    <div class="settings-panel">
      <div class="settings-content">
        <v-row>
          <v-col cols="12" md="4">
            <Notify
            v-model:notification-time="notificationTime"
            v-model:desktopNotifications="desktopNotifications"
            />
          </v-col>
          <v-col cols="12" md="4">
            <Aigc
            v-model:aigc-enabled="aigcEnabled"
            v-model:token="aigcToken"
            v-model:model="aigcModel"
            />
          </v-col>
          <v-col cols="12" md="6">
            <Export
              ref="exportCardRef"
              @export-all="exportAllEvents"
              @export-filtered="exportFilteredEvents"
              @show-selection-dialog="showExportDialog = true"
              @update:exporting="exporting = $event"
              @update:exportResult="exportResult = $event"
              @update:exportFormat="selectedExportFormat = $event"
              @update:exportFilter="exportFilter = $event"
            />
          </v-col>

          <v-col cols="12" md="6">
            <WebDav
              ref="webdavCardRef"
              @test-connection="handleTestWebDAVConnection"
              @sync="handleSyncWithWebDAV"
              @update:webdavResult="webdavResult = $event"
            />
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
    <ExportChat
      v-model="showExportDialog"
      :items="Lists"
      @export-selected="exportSelectedLists"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { SettingService } from '@/services/SettingService';
import Notify from '@/components/Settings/Notify.vue';
// biome-ignore lint/style/useImportType: bug
import Export from '@/components/Settings/Export.vue';
// biome-ignore lint/style/useImportType: bug
import WebDav from '@/components/Settings/WebDav.vue';
import ExportChat from '@/components/Settings/ExportChat.vue';
import Aigc from '../Settings/Aigc.vue';
// 组件引用
const exportCardRef = ref<InstanceType<typeof Export> | null>(null);
const webdavCardRef = ref<InstanceType<typeof WebDav> | null>(null);

// 设置状态
const desktopNotifications = ref(true);
const notificationTime = ref('');
const aigcEnabled = ref(false);
const aigcToken = ref('');
const aigcModel = ref('');

// 导出设置
const selectedExportFormat = ref('ics');
const exportFilter = ref('all');
const exporting = ref(false);
const exportResult = ref<{success: boolean; message: string} | null>(null);
const showExportDialog = ref(false);
const Lists = ref<any[]>([]);

// WebDAV 同步相关变量
const webdavConnectionTested = ref(false);
const webdavResult = ref<{success: boolean; message: string} | null>(null);

// 初始化
onMounted(async () => {
  // 加载可导出的事件列表
  try {
    Lists.value = await SettingService.getExportableLists();
  } catch (error) {
    console.error('初始化失败', error);
  }
});

const saveSettings = async () => {
  try {
    
    alert('设置已保存');
  } catch (error) {
    console.error('保存设置失败', error);
  }
};

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
    console.error('选择保存路径失败', error);
    return null;
  }
};

// 导出所有事件
const exportAllEvents = async () => {
  try {
    // 先让用户选择保存位置
    const filename = 'all_todopulse_events';
    const format = selectedExportFormat.value;
    const customPath = await selectSavePathForExport(filename, format);
    
    if (customPath) {
      // 用户选择了保存位置
      const result = await SettingService.exportAllEvents(
        format,
        customPath
      );
      exportResult.value = {
        success: true,
        message: `成功导出到文件: ${result}`
      };
    } else {
      // 用户取消选择
      exportResult.value = {
        success: false,
        message: '未选择保存位置'
      };
    }
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    exportResult.value = {
      success: false,
      message: `导出失败: ${errorMessage || '未知错误'}`
    };
  } finally {
    exporting.value = false;
    if (exportCardRef.value) {
      exportCardRef.value.setExporting(false);
      exportCardRef.value.setResult(exportResult.value);
    }
  }
};

// 导出选定事件
const exportSelectedLists = async (selectedLists: any[]) => {
  if (selectedLists.length === 0) return;
  exporting.value = true;
  if (exportCardRef.value) {
    exportCardRef.value.setExporting(true);
  }

  try {
    // 先让用户选择保存位置
    const filename = 'selected_events';
    const format = selectedExportFormat.value;
    const customPath = await selectSavePathForExport(filename, format);
    
    if (customPath) {
      const selectedListIds = selectedLists.map((list: any) => list);
      const result = await SettingService.exportLists(
        selectedListIds,
        selectedExportFormat.value,
        customPath
      );
      exportResult.value = {
        success: true,
        message: `成功导出到文件: ${result}`
      };
    } else {
      // 用户取消选择
      exportResult.value = {
        success: false,
        message: '未选择保存位置'
      };
    }
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    exportResult.value = {
      success: false,
      message: `导出失败: ${errorMessage || '未知错误'}`
    };
  } finally {
    exporting.value = false;
    if (exportCardRef.value) {
      exportCardRef.value.setExporting(false);
      exportCardRef.value.setResult(exportResult.value);
    }
  }
};

// 导出按筛选条件过滤的事件
const exportFilteredEvents = async () => {
  try {
    // 确定文件名和格式
    let filename: string;
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
      let result: string;
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
      // 用户取消选择
      exportResult.value = {
        success: false,
        message: '未选择保存位置'
      };
    }
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    exportResult.value = {
      success: false,
      message: `导出失败: ${errorMessage || '未知错误'}`
    };
  } finally {
    exporting.value = false;
    if (exportCardRef.value) {
      exportCardRef.value.setExporting(false);
      exportCardRef.value.setResult(exportResult.value);
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
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    webdavResult.value = {
      success: false,
      message: `连接错误: ${errorMessage || '未知错误'}`
    };
    webdavConnectionTested.value = false;
  } finally {
    if (webdavCardRef.value) {
      webdavCardRef.value.setTestingStatus(false);
      webdavCardRef.value.setConnectionTested(webdavConnectionTested.value);
      webdavCardRef.value.setResult(webdavResult.value);
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
    const result = await SettingService.syncDirectoryWithWebDAV(
      data.host,
      data.username,
      data.password,
      data.localDir,
      data.remoteDir
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
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    webdavResult.value = {
      success: false,
      message: `同步错误: ${errorMessage || '未知错误'}`
    };
  } finally {
    if (webdavCardRef.value) {
      webdavCardRef.value.setSyncingStatus(false);
      webdavCardRef.value.setResult(webdavResult.value);
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
</style>