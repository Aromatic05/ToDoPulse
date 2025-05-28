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
          <div
            v-for="(_time, index) in localNotificationTime"
            :key="index"
            class="d-flex align-center"
          >
            <v-text-field
              v-model="localNotificationTime[index]"
              :label="
                index === 0
                  ? '提醒时间 (HH:MM) (Reminder Time)'
                  : '额外提醒时间'
              "
              type="time"
              variant="outlined"
              density="compact"
              class="mt-2 flex-grow-1"
              @update:model-value="updateSettings"
              placeholder="例如: 09:00 (e.g., 09:00)"
            ></v-text-field>
            <v-btn
              v-if="index > 0"
              icon
              variant="text"
              size="small"
              class="mt-2 ml-2"
              @click="removeTimeField(index)"
            >
              <v-icon>mdi-delete</v-icon>
            </v-btn>
          </div>

          <v-btn
            prepend-icon="mdi-plus"
            variant="text"
            size="small"
            class="mt-3"
            @click="addTimeField"
          >
            添加提醒时间
          </v-btn>
        </div>
      </v-expand-transition>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { SettingService } from "@/services/SettingService";
import type { Info } from "src-tauri/bindings/Info";

// 本地状态
const localDesktopNotifications = ref(true);
const localNotificationTime = ref<string[]>(["09:00"]);

// 初始化
onMounted(async () => {
  try {
    // 从设置服务获取通知设置
    const settings = SettingService.getNotificationSettings();
    if (settings) {
      localDesktopNotifications.value = settings.switch ?? true;
      localNotificationTime.value =
        settings.time && settings.time.length > 0 ? settings.time : ["09:00"];
    }
  } catch (error) {
    console.error("加载通知设置失败", error);
  }
});

// 添加新的时间字段
const addTimeField = () => {
  localNotificationTime.value.push("");
  updateSettings();
};

// 移除时间字段
const removeTimeField = (index: number) => {
  localNotificationTime.value.splice(index, 1);
  updateSettings();
};

// 更新设置
const updateSettings = async () => {
  try {
    // 过滤掉空值
    const filteredTimes = localNotificationTime.value.filter(
      (time) => time.trim() !== ""
    );

    const infoSettings: Info = {
      switch: localDesktopNotifications.value,
      time: filteredTimes.length > 0 ? filteredTimes : ["09:00"],
    };

    await SettingService.saveSettings({
      Info: infoSettings,
    });
  } catch (error) {
    console.error("保存通知设置失败", error);
  }
};

defineExpose({
  updateSettings,
})

</script>
