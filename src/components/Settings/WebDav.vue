<template>
  <v-expansion-panel>
    <v-expansion-panel-title>
      <h3 class="text-h6 font-weight-medium">WebDAV 同步</h3>
    </v-expansion-panel-title>
    <v-expansion-panel-text>
      <v-form @submit.prevent="testConnection">
        <v-row>
          <v-col cols="12">
            <v-text-field v-model="host" label="WebDAV 服务器地址" placeholder="https://example.com/dav/" variant="outlined"
              density="compact" class="mb-3" required></v-text-field>
          </v-col>
          <v-col cols="12" md="6">
            <v-text-field v-model="username" label="用户名" variant="outlined" density="compact" class="mb-3"
              required></v-text-field>
          </v-col>
          <v-col cols="12" md="6">
            <v-text-field v-model="password" label="密码" type="password" variant="outlined" density="compact"
              class="mb-3" required></v-text-field>
          </v-col>
          <v-col cols="12" md="6">
            <v-text-field v-model="remoteDir" label="远程目录" placeholder="/remote/path/" variant="outlined"
              density="compact" class="mb-3" required></v-text-field>
          </v-col>

          <v-col cols="12" md="6">
            <v-btn block color="primary" @click="testConnection" :loading="isTesting" class="mb-3">
              测试连接
            </v-btn>
          </v-col>
          <v-col cols="12" md="6">
            <v-btn block color="success" @click="sync" :loading="isSyncing" :disabled="!connectionTested" class="mb-3">
              开始同步
            </v-btn>
          </v-col>
        </v-row>
      </v-form>

      <v-alert v-if="result" :color="result.success ? 'success' : 'error'" :title="result.success ? '操作成功' : '操作失败'"
        :text="result.message" class="mt-3" variant="tonal" closable @click:close="clearResult"></v-alert>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from "vue";
import type { WebDav } from "src-tauri/bindings/WebDav";
import { SettingService } from "@/services/SettingService";
import { debounce } from "@/utils/debounce";

// 状态
const host = ref("");
const username = ref("");
const password = ref("");
const localDir = ref("/");
const remoteDir = ref("/");
const isTesting = ref(false);
const isSyncing = ref(false);
const connectionTested = ref(false);
const result = ref<{ success: boolean; message: string } | null>(null);
const isSaving = ref(false);
const isInitializing = ref(true);  // 初始化标志，用于防止初始化时触发自动保存
const lastModified = ref(Date.now()); // 上次修改时间

// 事件
const emit = defineEmits(["test-connection", "sync", "update:webdavResult"]);

// 方法
const testConnection = () => {
  if (!host.value || !username.value || !password.value) {
    result.value = {
      success: false,
      message: "请填写完整的 WebDAV 服务器信息",
    };
    emit("update:webdavResult", result.value);
    return;
  }

  isTesting.value = true;
  emit("test-connection", {
    host: host.value,
    username: username.value,
    password: password.value,
  });
};

onMounted(() => {
  try {
    // 从设置服务获取 WebDAV 设置
    const settings = SettingService.getWebDavSettings();
    if (settings) {
      host.value = settings.host || "";
      username.value = settings.username || "";
      password.value = settings.password || "";
      remoteDir.value = settings.remote_dir || "/";
    }
  } catch (error) {
    console.error("加载 WebDAV 设置失败", error);
  } finally {
    // 初始化完成后设置标志
    setTimeout(() => {
      isInitializing.value = false;
    }, 500); // 给予足够的时间让组件完全渲染
  }
});

const sync = () => {
  if (!connectionTested.value) {
    result.value = {
      success: false,
      message: "请先测试连接",
    };
    emit("update:webdavResult", result.value);
    return;
  }

  if (!remoteDir.value) {
    result.value = {
      success: false,
      message: "请填写远程目录",
    };
    emit("update:webdavResult", result.value);
    return;
  }

  isSyncing.value = true;
  emit("sync", {
    host: host.value,
    username: username.value,
    password: password.value,
    localDir: localDir.value,
    remoteDir: remoteDir.value,
  });
};

const clearResult = () => {
  result.value = null;
  emit("update:webdavResult", null);
};

const updateSettings = async () => {
  if (isSaving.value) return; // 防止重复保存

  isSaving.value = true;
  try {
    const webDavSetting: WebDav = {
      enabled: true,
      host: host.value,
      username: username.value,
      password: password.value,
      remote_dir: remoteDir.value,
    };
    await SettingService.saveSettings({ WebDav: webDavSetting });
    console.log("WebDAV 设置保存成功");
  } catch (error) {
    console.error("保存 WebDAV 设置失败", error);
  } finally {
    isSaving.value = false;
  }
};

// 使用防抖的自动保存
const debouncedSave = debounce(async () => {
  await updateSettings();
}, 1000); // 1秒防抖延迟

// 监听设置变化，自动保存
watch(
  [host, username, password, remoteDir],
  () => {
    // 初始化或保存中时不自动保存
    if (isInitializing.value || isSaving.value) {
      return;
    }

    // 只有关键字段有值时才自动保存
    if (!host.value || !username.value) {
      return;
    }

    // 计算距上次修改的时间
    const now = Date.now();
    const timeSinceLastModification = now - lastModified.value;

    // 更新最后修改时间
    lastModified.value = now;

    console.log('WebDAV 设置已变更，准备自动保存');

    // 时间间隔太短可能是批量操作，使用更长的延迟
    const delay = timeSinceLastModification < 300 ? 1500 : 800;

    // 使用延迟保存以避免与UI渲染冲突
    setTimeout(() => {
      // 再次检查，确保没有新的修改正在进行中
      if (!isSaving.value) {
        debouncedSave();
      }
    }, delay);
  },
  { deep: true }
);

// 在组件卸载时清理资源
onUnmounted(() => {
  console.log('WebDAV 设置组件已卸载');
  // 如果有未完成的保存，可以在这里记录
  if (isSaving.value) {
    console.log('组件卸载时仍有未完成的保存操作');
  }
});

// 提供方法给父组件调用
defineExpose({
  setTestingStatus: (value: boolean) => {
    isTesting.value = value;
  },
  setSyncingStatus: (value: boolean) => {
    isSyncing.value = value;
  },
  setConnectionTested: (value: boolean) => {
    connectionTested.value = value;
  },
  setResult: (value: { success: boolean; message: string } | null) => {
    result.value = value;
  },
  updateSettings,
});
</script>
