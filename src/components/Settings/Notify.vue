<template>
  <v-card class="pa-4">
    <h3 class="text-h6 mb-5">通知设置</h3>

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
  </v-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

// 定义组件的属性
const props = defineProps({
  desktopNotifications: {
    type: Boolean,
    default: false
  },
  notificationTime: {
    type: String,
    default: '09:00' // Default time if not provided by parent
  }
});

// 定义组件的事件
const emit = defineEmits(['update:desktopNotifications', 'update:notificationTime']); // Added 'update:notificationTime'

// 本地状态
const localDesktopNotifications = ref(props.desktopNotifications);
const localNotificationTime = ref(props.notificationTime); // Use prop for initial value and rename for clarity

// 监听属性变化
watch(() => props.desktopNotifications, (newVal) => {
  localDesktopNotifications.value = newVal;
});

watch(() => props.notificationTime, (newVal) => { // Watch for parent changes to notificationTime
  localNotificationTime.value = newVal;
});

// 更新设置
const updateSettings = () => {
  emit('update:desktopNotifications', localDesktopNotifications.value);
  emit('update:notificationTime', localNotificationTime.value); // Emit the current time
};
</script>