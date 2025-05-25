<template>
  <v-card class="pa-4">
    <h3 class="text-h6 mb-5">WebDAV 同步</h3>
    <v-form @submit.prevent="testConnection">
      <v-row>
        <v-col cols="12">
          <v-text-field
            v-model="host"
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
            v-model="username"
            label="用户名"
            variant="outlined"
            density="compact"
            class="mb-3"
            required
          ></v-text-field>
        </v-col>
        <v-col cols="12" md="6">
          <v-text-field
            v-model="password"
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
            v-model="localDir"
            label="本地目录"
            variant="outlined"
            density="compact"
            class="mb-3"
            required
          ></v-text-field>
        </v-col>
        <v-col cols="12" md="6">
          <v-text-field
            v-model="remoteDir"
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
            @click="testConnection"
            :loading="isTesting"
            class="mb-3"
          >
            测试连接
          </v-btn>
        </v-col>
        <v-col cols="12" md="6">
          <v-btn
            block
            color="success"
            @click="sync"
            :loading="isSyncing"
            :disabled="!connectionTested"
            class="mb-3"
          >
            开始同步
          </v-btn>
        </v-col>
      </v-row>
    </v-form>

    <v-alert
      v-if="result"
      :color="result.success ? 'success' : 'error'"
      :title="result.success ? '操作成功' : '操作失败'"
      :text="result.message"
      class="mt-3"
      variant="tonal"
      closable
      @click:close="clearResult"
    ></v-alert>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';

// 状态
const host = ref('');
const username = ref('');
const password = ref('');
const localDir = ref('');
const remoteDir = ref('/');
const isTesting = ref(false);
const isSyncing = ref(false);
const connectionTested = ref(false);
const result = ref<{ success: boolean; message: string } | null>(null);

// 事件
const emit = defineEmits([
  'test-connection',
  'sync',
  'update:webdavResult'
]);

// 方法
const testConnection = () => {
  if (!host.value || !username.value || !password.value) {
    result.value = {
      success: false,
      message: '请填写完整的 WebDAV 服务器信息'
    };
    emit('update:webdavResult', result.value);
    return;
  }

  isTesting.value = true;
  emit('test-connection', {
    host: host.value,
    username: username.value,
    password: password.value
  });
};

const sync = () => {
  if (!connectionTested.value) {
    result.value = {
      success: false,
      message: '请先测试连接'
    };
    emit('update:webdavResult', result.value);
    return;
  }

  if (!localDir.value || !remoteDir.value) {
    result.value = {
      success: false,
      message: '请填写本地和远程目录'
    };
    emit('update:webdavResult', result.value);
    return;
  }

  isSyncing.value = true;
  emit('sync', {
    host: host.value,
    username: username.value,
    password: password.value,
    localDir: localDir.value,
    remoteDir: remoteDir.value
  });
};

const clearResult = () => {
  result.value = null;
  emit('update:webdavResult', null);
};

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
  }
});
</script>