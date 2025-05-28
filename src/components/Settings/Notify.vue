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
import { ref, onMounted, watch, onUnmounted } from "vue";
import { SettingService } from "@/services/SettingService";
import type { Info } from "src-tauri/bindings/Info";
import { debounce } from "@/utils/debounce";

// 本地状态
const localDesktopNotifications = ref(true);
const localNotificationTime = ref<string[]>(["09:00"]);
const isSaving = ref(false);
const isInitializing = ref(true);  // 初始化标志，用于防止初始化时触发自动保存
const lastModified = ref(Date.now()); // 上次修改时间

// 初始化
onMounted(() => {
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
  } finally {
    // 初始化完成后设置标志
    setTimeout(() => {
      isInitializing.value = false;
    }, 500); // 给予足够的时间让组件完全渲染
  }
});

// 添加新的时间字段
const addTimeField = () => {
  localNotificationTime.value.push("");
};

// 移除时间字段
const removeTimeField = (index: number) => {
  localNotificationTime.value.splice(index, 1);
};

// 更新设置 - 不防抖的版本
const updateSettings = async () => {
  if (isSaving.value) return; // 防止重复保存
  
  isSaving.value = true;
  try {
    // 过滤掉空值
    const filteredTimes = localNotificationTime.value.filter(
      (time) => time.trim() !== ""
    );

    const infoSettings: Info = {
      switch: localDesktopNotifications.value,
      time: filteredTimes.length > 0 ? filteredTimes : ["09:00"],
    };

    // 使用 Promise.race 添加超时处理
    const timeoutPromise = new Promise((_resolve, reject) => 
      setTimeout(() => reject(new Error("保存操作超时")), 2000)
    );
    await Promise.race([
      SettingService.saveSettings({
        Info: infoSettings,
      }),
      timeoutPromise,
    ]);
    console.log('通知设置保存成功');
  } catch (error) {
    console.error("保存通知设置失败", error);
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
  [localDesktopNotifications, localNotificationTime],
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
    
    // 输出调试信息
    console.log('通知设置已变更，准备自动保存');
    
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

// 在组件卸载时检查是否有未保存的设置
onUnmounted(() => {
  console.log('通知设置组件已卸载');
  // 如果有未完成的保存，记录日志
  if (isSaving.value) {
    console.log('组件卸载时仍有未完成的保存操作');
  }
});

defineExpose({
  updateSettings,
})

</script>
