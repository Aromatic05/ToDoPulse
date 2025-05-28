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
        @update:model-value="updateSettings"
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
            @update:model-value="updateSettings"
          ></v-text-field>
          <v-text-field
            v-model="localModel"
            label="模型名称 (如 gpt-3.5-turbo)"
            variant="outlined"
            density="compact"
            class="mt-2"
            hide-details
            @update:model-value="updateSettings"
          ></v-text-field>
        </div>
      </v-expand-transition>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { SettingService } from '@/services/SettingService';
import type { Model } from 'src-tauri/bindings/Model';

// 本地状态
const localAigcEnabled = ref(false);
const localToken = ref('');
const localModel = ref('');

// 初始化
onMounted(async () => {
  try {
    // 从设置服务获取AI模型设置
    const settings = SettingService.getAigcSettings();
    if (settings) {
      localAigcEnabled.value = settings.switch ?? false;
      localToken.value = settings.tokens ?? '';
      localModel.value = settings.name ?? '';
    }
  } catch (error) {
    console.error('加载AI模型设置失败', error);
  }
});

// 更新设置
const updateSettings = async () => {
  try {
    const modelSetting: Model = {
      switch: localAigcEnabled.value,
      tokens: localToken.value,
      name: localModel.value,
    };
    await SettingService.saveSettings({Model: modelSetting});
  } catch (error) {
    console.error('保存AI模型设置失败', error);
  }
};

defineExpose({
  updateSettings,
})
</script>
