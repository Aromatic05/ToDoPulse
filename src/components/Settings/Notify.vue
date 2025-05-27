<template>
  <v-expansion-panel>
    <v-expansion-panel-title>
      <h3 class="text-h6 font-weight-medium">通知设置</h3>
    </v-expansion-panel-title>
    <v-expansion-panel-text>
      <v-switch 
        v-model="localDesktopNotifications" 
        label="桌面通知" 
        color="primary" 
        hide-details
        class="mb-2"
        @update:model-value="updateSettings"
      ></v-switch>

      <v-expand-transition>
        <div v-if="localDesktopNotifications">
          <v-text-field
            v-model="localNotificationTime"
            label="提醒时间 (HH:MM) (Reminder Time)"
            type="time"
            variant="outlined"
            density="compact"
            class="mt-2"
            @update:model-value="updateSettings"
            placeholder="例如: 09:00 (e.g., 09:00)"
          ></v-text-field>
        </div>
      </v-expand-transition>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { SettingService } from '@/services/SettingService';

// 本地状态
const localDesktopNotifications = ref(true);
const localNotificationTime = ref('09:00');

// 初始化
onMounted(async () => {
  try {
    // 从设置服务获取通知设置
    const settings = await SettingService.getNotificationSettings();
    if (settings) {
      localDesktopNotifications.value = settings.enabled ?? true;
      localNotificationTime.value = settings.time ?? '09:00';
    }
  } catch (error) {
    console.error('加载通知设置失败', error);
  }
});

// 更新设置
const updateSettings = async () => {
  try {
    // 保存设置到设置服务
    await SettingService.saveNotificationSettings({
      enabled: localDesktopNotifications.value,
      time: localNotificationTime.value
    });
  } catch (error) {
    console.error('保存通知设置失败', error);
  }
};
</script>