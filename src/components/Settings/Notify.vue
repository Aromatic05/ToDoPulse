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
            <div class="position-relative flex-grow-1 mt-2">
              <v-btn
                block
                variant="outlined"
                density="compact"
                :prepend-icon="'mdi-clock-outline'"
                class="time-input"
                @click="openTimePicker(index)"
              >
                <div class="d-flex justify-space-between align-center w-100">
                  <span>{{ localNotificationTime[index] || '设置时间' }}</span>
                  <span class="text-caption">
                    {{ index === 0 ? '提醒时间' : '额外提醒时间' }}
                  </span>
                </div>
              </v-btn>
            </div>

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

          <!-- Cron时间选择器 -->
          <CronTimePicker
            v-model="showCronPicker"
            :initial-cron="getCronFromTime(selectedCronIndex)"
            :title="'设置提醒时间'"
            @confirm="handleCronConfirm"
          />

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
import { ref, onMounted, watch } from "vue";
import { SettingService } from "@/services/SettingService";
import type { Info } from "src-tauri/bindings/Info";
import { debounce } from "@/utils/debounce";
import CronTimePicker from "@/utils/TimePicker.vue";

// 扩展Info接口以支持cron表达式
interface ExtendedInfo extends Info {
  cronExpressions?: {[key: number]: string};
}

// 本地状态
const localDesktopNotifications = ref(true);
const localNotificationTime = ref<string[]>(["09:00"]);
const notificationCronExpressions = ref<{[key: number]: string}>({});
const isSaving = ref(false);
const isInitializing = ref(true);  // 初始化标志，用于防止初始化时触发自动保存
const lastModified = ref(Date.now()); // 上次修改时间

// Cron选择器相关
const showCronPicker = ref(false);
const selectedCronIndex = ref(0);

// 初始化
onMounted(() => {
  try {
    // 从设置服务获取通知设置
    const settings = SettingService.getNotificationSettings();
    if (settings) {
      localDesktopNotifications.value = settings.switch ?? true;
      
      // settings.time 本身就是 cron 表达式列表
      if (settings.time && settings.time.length > 0) {
        // 存储原始cron表达式
        const cronExpressions: {[key: number]: string} = {};
        
        // 将cron表达式转换为显示时间格式
        const displayTimes = settings.time.map((cronExpr, index) => {
          // 解析cron表达式
          try {
            const parts = cronExpr.split(" ");
            if (parts.length === 5) {
              const minute = parts[0];
              const hour = parts[1];
              
              // 只处理具体的时间值，不处理特殊字符
              const minuteNum = Number(minute);
              const hourNum = Number(hour);
              
              if (!Number.isNaN(minuteNum) && !Number.isNaN(hourNum)) {
                // 保存原始cron表达式
                cronExpressions[index] = cronExpr;
                // 返回格式化的时间
                return `${hourNum.toString().padStart(2, "0")}:${minuteNum.toString().padStart(2, "0")}`;
              }
            }
          } catch (e) {
            console.error("解析cron表达式失败", e);
          }
          
          // 如果解析失败，保存原始cron表达式并返回默认时间
          cronExpressions[index] = cronExpr;
          return "09:00";
        });
        
        localNotificationTime.value = displayTimes;
        notificationCronExpressions.value = cronExpressions;
      } else {
        // 默认时间和cron表达式
        localNotificationTime.value = ["09:00"];
        notificationCronExpressions.value = { 0: "0 9 * * *" };
      }
      
      // 检查是否有额外的cron表达式（向后兼容）
      const extendedSettings = settings as ExtendedInfo;
      if (extendedSettings.cronExpressions) {
        notificationCronExpressions.value = {
          ...notificationCronExpressions.value,
          ...extendedSettings.cronExpressions
        };
      }
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
  
  // 同时更新对应的cron表达式
  const updatedCronExpressions = { ...notificationCronExpressions.value };
  delete updatedCronExpressions[index];
  
  // 重新映射剩余的索引
  const newCronExpressions: {[key: number]: string} = {};
  for (const key of Object.keys(updatedCronExpressions)) {
    const numKey = Number.parseInt(key, 10);
    if (numKey > index) {
      newCronExpressions[numKey - 1] = updatedCronExpressions[numKey];
    } else {
      newCronExpressions[numKey] = updatedCronExpressions[numKey];
    }
  }
  
  notificationCronExpressions.value = newCronExpressions;
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

    // 将时间字符串转换为cron表达式
    const cronTimes = filteredTimes.map((time, index) => {
      // 如果有对应的cron表达式，优先使用它
      if (notificationCronExpressions.value[index]) {
        return notificationCronExpressions.value[index];
      }
      
      // 否则，从时间字符串生成基本的cron表达式
      try {
        const [hours, minutes] = time.split(':').map(Number);
        if (!Number.isNaN(hours) && !Number.isNaN(minutes)) {
          return `${minutes} ${hours} * * *`;
        }
      } catch (e) {
        console.error("转换时间到cron表达式失败", e);
      }
      
      // 默认每天9点
      return "0 9 * * *";
    });
    
    // 创建基础Info对象，直接保存cron表达式
    const infoSettings: Info = {
      switch: localDesktopNotifications.value,
      time: cronTimes.length > 0 ? cronTimes : ["0 9 * * *"]
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
  [localDesktopNotifications, localNotificationTime, notificationCronExpressions],
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

// 打开Cron选择器 (用于设置高级定时规则)
const openCronPicker = (index: number) => {
  selectedCronIndex.value = index;
  showCronPicker.value = true;
  // 默认显示高级模式标签
  setTimeout(() => {
    const cronPicker = document.querySelector('.cron-picker');
    if (cronPicker) {
      const advancedTab = cronPicker.querySelector('[value="advanced"]');
      if (advancedTab && advancedTab instanceof HTMLElement) {
        advancedTab.click();
      }
    }
  }, 50);
};

// 打开时间选择器 (使用CronTimePicker的简单模式)
const openTimePicker = (index: number) => {
  selectedCronIndex.value = index;
  showCronPicker.value = true;
  // 确保显示简单模式标签
  setTimeout(() => {
    const cronPicker = document.querySelector('.cron-picker');
    if (cronPicker) {
      const simpleTab = cronPicker.querySelector('[value="simple"]');
      if (simpleTab && simpleTab instanceof HTMLElement) {
        simpleTab.click();
      }
    }
  }, 50);
};

// 从时间字符串获取cron表达式（如果存在）
const getCronFromTime = (index: number): string => {
  if (notificationCronExpressions.value[index]) {
    return notificationCronExpressions.value[index];
  }
  
  // 如果没有现存的cron表达式，则从当前时间生成一个
  const time = localNotificationTime.value[index] || "09:00";
  const [hours, minutes] = time.split(":").map(Number);
  return `${minutes} ${hours} * * *`; // 默认每天执行
};

// 处理cron表达式确认
const handleCronConfirm = (cronExpression: string) => {
  // 保存cron表达式
  const updatedExpressions = { ...notificationCronExpressions.value };
  updatedExpressions[selectedCronIndex.value] = cronExpression;
  notificationCronExpressions.value = updatedExpressions;
  
  // 更新显示时间（从cron表达式提取）
  try {
    const parts = cronExpression.split(" ");
    if (parts.length === 5) {
      const minute = parts[0];
      const hour = parts[1];
      
      // 只有当cron表达式的分钟和小时部分是具体的数字时才更新显示时间
      const minuteNum = Number(minute);
      const hourNum = Number(hour);
      if (!Number.isNaN(minuteNum) && !Number.isNaN(hourNum)) {
        const newTime = `${hour.padStart(2, "0")}:${minute.padStart(2, "0")}`;
        const newTimes = [...localNotificationTime.value];
        newTimes[selectedCronIndex.value] = newTime;
        localNotificationTime.value = newTimes;
      }
    }
  } catch (error) {
    console.error("从cron表达式提取时间失败", error);
  }
  
  // 保存设置
  debouncedSave();
};

// 替换原有的开启选择器方法，需要修改模板中对应的调用
defineExpose({
  updateSettings,
  openTimePicker,
  openCronPicker
});
</script>

<style scoped>
.time-input {
  cursor: pointer;
}
</style>
