<template>
  <v-expansion-panel>
    <v-expansion-panel-title>
      <h3 class="text-h6 font-weight-medium">模型设置</h3>
    </v-expansion-panel-title>
    <v-expansion-panel-text>
      <v-switch
        v-model="localAigcEnabled"
        label="智能生成标签"
        color="primary"
        hide-details
        class="mb-4"
      ></v-switch>
      <v-expand-transition>
        <div v-if="localAigcEnabled">
          <v-text-field
            v-model="localToken"
            label="API Token"
            variant="outlined"
            density="compact"
            class="mt-2"
            hide-details
          ></v-text-field>
          <v-text-field
            v-model="localApiUrl"
            label="API URL"
            variant="outlined"
            density="compact"
            class="mt-2"
            hide-details
          ></v-text-field>
          <v-text-field
            v-model="localModel"
            label="模型名称 (如 gpt-3.5-turbo)"
            variant="outlined"
            density="compact"
            class="mt-2"
            hide-details
          ></v-text-field>
        </div>
      </v-expand-transition>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue';
import { SettingService } from '@/services/SettingService';
import type { Model } from 'src-tauri/bindings/Model';
import { debounce } from '@/utils/debounce';

// 本地状态
const localAigcEnabled = ref(false);
const localToken = ref('');
const localModel = ref('');
const localApiUrl = ref('https://api.openai.com/v1/chat/completions'); // 默认API URL
const isSaving = ref(false);
const isInitializing = ref(true);  // 初始化标志，用于防止初始化时触发自动保存
const lastModified = ref(Date.now()); // 上次修改时间

// 初始化
onMounted(() => {
  try {
    // 从设置服务获取AI模型设置
    const settings = SettingService.getAigcSettings();
    if (settings) {
      localAigcEnabled.value = settings.switch ?? false;
      localToken.value = settings.tokens ?? '';
      localModel.value = settings.name ?? '';
      localApiUrl.value = settings.api ?? 'https://api.openai.com/v1/chat/completions'; // 确保API URL有默认值
    }
  } catch (error) {
    console.error('加载AI模型设置失败', error);
  } finally {
    // 初始化完成后设置标志
    setTimeout(() => {
      isInitializing.value = false;
    }, 500); // 给予足够的时间让组件完全渲染
  }
});

// 更新设置 - 不防抖的版本，用于外部调用
const updateSettings = async () => {
  if (isSaving.value) return; // 防止重复保存
  
  isSaving.value = true;
  try {
    const modelSetting: Model = {
      switch: localAigcEnabled.value,
      tokens: localToken.value,
      name: localModel.value,
      api: localApiUrl.value,
    };
    await SettingService.saveSettings({Model: modelSetting});
    console.log('AI模型设置保存成功');
  } catch (error) {
    console.error('保存AI模型设置失败', error);
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
  [localAigcEnabled, localToken, localModel],
  () => {
    // 初始化或保存中时不自动保存
    if (isInitializing.value || isSaving.value) {
      return;
    }
    
    // 计算距上次修改的时间
    const now = Date.now();
    const timeSinceLastModification = now - lastModified.value;
    
    // 更新最后修改时间
    lastModified.value = now;
    
    console.log('AI模型设置已变更，准备自动保存');
    
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
  console.log('AI模型设置组件已卸载');
  // 如果有未完成的保存，可以在这里强制执行
  if (isSaving.value) {
    console.log('组件卸载时仍有未完成的保存操作');
  }
});

defineExpose({
  updateSettings,
})
</script>
