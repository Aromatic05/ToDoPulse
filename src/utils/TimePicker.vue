<template>
  <div class="cron-picker">
    <v-dialog v-model="dialog" max-width="520px">
      <v-card>
        <v-card-title class="text-h5 pa-4">
          {{ title || "设置定时任务" }}
          <v-spacer></v-spacer>
          <v-btn icon @click="close">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>

        <v-card-text class="pa-4">
          <v-tabs v-model="activeTab">
            <v-tab value="simple">简单模式</v-tab>
            <v-tab value="advanced">高级模式</v-tab>
          </v-tabs>

          <v-window v-model="activeTab" class="mt-4">
            <!-- 简单模式 -->
            <v-window-item value="simple">
              <v-row>
                <v-col cols="12" md="6">
                  <div class="field-label-wrapper">频率</div>
                  <v-select
                    v-model="frequency"
                    :items="frequencies"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="mb-2"
                  ></v-select>
                </v-col>

                <v-col cols="12" md="6" v-if="showTimeInput">
                  <div class="field-label-wrapper">时间</div>
                  <v-text-field
                    v-model="timeInput"
                    variant="outlined"
                    density="compact"
                    hide-details
                    readonly
                    class="mb-2 time-input-field-0"
                    @click="openTimePicker"
                  >
                    <template v-slot:append>
                      <v-icon @click.stop="openTimePicker"
                        >mdi-clock-outline</v-icon
                      >
                    </template>
                  </v-text-field>
                </v-col>
              </v-row>

              <v-row v-if="frequency === 'weekly'">
                <v-col cols="12">
                  <div class="d-flex flex-wrap">
                    <v-checkbox
                      v-for="(day, index) in weekdays"
                      :key="index"
                      v-model="selectedDays"
                      :label="day.text"
                      :value="day.value"
                      density="compact"
                      class="mr-4"
                    ></v-checkbox>
                  </div>
                </v-col>
              </v-row>

              <v-row v-if="frequency === 'monthly'">
                <v-col cols="12" md="6">
                  <v-select
                    v-model="dayOfMonth"
                    :items="daysOfMonth"
                    label="日期"
                    variant="outlined"
                    density="compact"
                  ></v-select>
                </v-col>
              </v-row>
            </v-window-item>

            <!-- 高级模式 -->
            <v-window-item value="advanced">
              <v-row>
                <v-col cols="12">
                  <div class="field-label-wrapper">cron表达式</div>

                  <v-text-field
                    v-model="cronExpression"
                    persistent-hint
                    variant="outlined"
                    class="mb-2"
                    density="compact"
                    :error-messages="cronError"
                  ></v-text-field>
                </v-col>
              </v-row>
            </v-window-item>
          </v-window>

          <v-row class="mt-4">
            <v-col cols="12">
              <div class="text-caption">预览：</div>
              <div class="text-body-1">{{ cronDescription }}</div>
              <div class="text-caption mt-2">下次执行时间：</div>
              <div class="text-body-2">{{ nextExecutionTime }}</div>
            </v-col>
          </v-row>

          <!-- 时间选择器菜单 - 时分组合选择 -->
          <v-menu
            v-model="timePickerMenu"
            :close-on-content-click="false"
            :close-on-back="true"
            transition="scale-transition"
            offset-y
            min-width="240px"
            :activator="`.time-input-field-${selectedCronIndex}`"
          >
            <v-card>
              <v-card-title class="text-subtitle-1 pb-0">选择时间</v-card-title>
              <v-card-text>
                <div class="time-selector-container">
                  <div class="time-selector-wrapper">
                    <div class="time-selector-header">
                      <div class="time-selector-label">时</div>
                      <div class="time-selector-label">分</div>
                    </div>
                    <div class="time-selector-body">
                      <div class="time-selector-column hours-column">
                        <div 
                          v-for="hour in 24" 
                          :key="`hour-${hour-1}`"
                          class="time-selector-option"
                          :class="{ 'selected': selectedHour === hour - 1 }"
                          @click="selectHour(hour - 1)"
                        >
                          {{ (hour - 1).toString().padStart(2, '0') }}
                        </div>
                      </div>
                      <div class="time-selector-column minutes-column">
                        <div 
                          v-for="minute in minuteOptions" 
                          :key="`minute-${minute}`"
                          class="time-selector-option"
                          :class="{ 'selected': selectedMinute === minute }"
                          @click="selectMinute(minute)"
                        >
                          {{ minute.toString().padStart(2, '0') }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </v-card-text>
              <v-divider></v-divider>
              <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn variant="text" @click="cancelTimeSelection">取消</v-btn>
                <v-btn color="primary" variant="text" @click="confirmHourMinuteSelection">确定</v-btn>
              </v-card-actions>
            </v-card>
          </v-menu>
        </v-card-text>

        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="primary" @click="confirm">确认</v-btn>
          <v-btn @click="close">取消</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";

// 定义组件属性
const props = defineProps({
  title: {
    type: String,
    default: "设置定时任务",
  },
  initialCron: {
    type: String,
    default: "0 9 * * *", // 默认每天上午9点
  },
  modelValue: {
    type: Boolean,
    default: false,
  },
});

// 定义事件
const emit = defineEmits(["update:modelValue", "confirm", "close"]);

// 内部状态
const dialog = ref(props.modelValue);
const activeTab = ref("simple");
const cronExpression = ref(props.initialCron);
const cronError = ref("");

// 简单模式的状态
const frequency = ref("daily"); // daily, weekly, monthly
const frequencies = [
  { title: "每天", value: "daily" },
  { title: "每周", value: "weekly" },
  { title: "每月", value: "monthly" },
];
const timeInput = ref("09:00");
const selectedDays = ref([1]); // 默认周一
const dayOfMonth = ref(1);
const daysOfMonth = Array.from({ length: 31 }, (_, i) => ({
  title: `${i + 1}日`,
  value: i + 1,
}));

// 时间选择器
const timePickerMenu = ref(false);
const selectedTime = ref("09:00");
const selectedCronIndex = ref(0);

// 时分选择器状态
const selectedHour = ref(9);
const selectedMinute = ref(0);
const minuteOptions = Array.from({ length: 60 }, (_, i) => i);
// 星期几的选项
const weekdays = [
  { text: "周一", value: 1 },
  { text: "周二", value: 2 },
  { text: "周三", value: 3 },
  { text: "周四", value: 4 },
  { text: "周五", value: 5 },
  { text: "周六", value: 6 },
  { text: "周日", value: 0 },
];

// 监听对话框值的变化
watch(
  () => props.modelValue,
  (newVal) => {
    dialog.value = newVal;
  }
);

watch(dialog, (newVal) => {
  emit("update:modelValue", newVal);
});

// 根据简单模式的选择生成cron表达式
const generateCronFromSimple = computed(() => {
  const [hour, minute] = timeInput.value.split(":").map(Number);

  switch (frequency.value) {
    case "daily":
      return `${minute} ${hour} * * *`;
    case "weekly": {
      // 确保至少选择了一天，否则默认周一
      const days = selectedDays.value.length > 0 ? selectedDays.value : [1];
      return `${minute} ${hour} * * ${days.join(",")}`;
    }
    case "monthly":
      return `${minute} ${hour} ${dayOfMonth.value} * *`;
    default:
      return "0 9 * * *";
  }
});

// 根据cron表达式更新简单模式的值
const updateSimpleFromCron = (cronStr: string) => {
  try {
    const parts = cronStr.split(" ");
    if (parts.length !== 5) {
      throw new Error("无效的cron表达式");
    }

    const minute = Number.parseInt(parts[0], 10);
    const hour = Number.parseInt(parts[1], 10);
    const dayOfMonthStr = parts[2];
    const dayOfWeekStr = parts[4];

    // 设置时间
    timeInput.value = `${hour.toString().padStart(2, "0")}:${minute
      .toString()
      .padStart(2, "0")}`;
    selectedTime.value = timeInput.value;

    // 确定频率和其他设置
    if (dayOfMonthStr === "*" && dayOfWeekStr === "*") {
      // 每天
      frequency.value = "daily";
    } else if (dayOfMonthStr === "*" && dayOfWeekStr !== "*") {
      // 每周
      frequency.value = "weekly";
      selectedDays.value = dayOfWeekStr
        .split(",")
        .map((d) => Number.parseInt(d, 10));
    } else if (dayOfMonthStr !== "*" && dayOfWeekStr === "*") {
      // 每月
      frequency.value = "monthly";
      dayOfMonth.value = Number.parseInt(dayOfMonthStr, 10);
    }
  } catch (error) {
    console.error("解析cron表达式失败", error);
    // 设置默认值
    frequency.value = "daily";
    timeInput.value = "09:00";
    selectedTime.value = "09:00";
  }
};

// 在初始化和cron表达式变化时更新简单模式
watch(
  () => props.initialCron,
  (newVal) => {
    cronExpression.value = newVal;
    updateSimpleFromCron(newVal);
  },
  { immediate: true }
);

// 当时间输入变化时更新小时和分钟
watch(timeInput, (newTimeStr) => {
  const [hour, minute] = newTimeStr.split(':').map(Number);
  selectedHour.value = hour;
  
  // 找到最接近的分钟选项
  const closestMinute = minuteOptions.reduce((prev, curr) => {
    return (Math.abs(curr - minute) < Math.abs(prev - minute)) ? curr : prev;
  });
  selectedMinute.value = closestMinute;
});

// 当简单模式变化时，更新cron表达式
watch(
  [frequency, timeInput, selectedDays, dayOfMonth],
  () => {
    if (activeTab.value === "simple") {
      cronExpression.value = generateCronFromSimple.value;
    }
  },
  { deep: true }
);

// cron表达式有效性检查
const validateCron = (cron: string) => {
  const cronRegex =
    /^(\*|([0-9]|[1-5][0-9]))(\/\d+)?(\s+(\*|([0-9]|1[0-9]|2[0-3]))(\/\d+)?){1}(\s+(\*|([1-9]|[12][0-9]|3[01]))(\/\d+)?){1}(\s+(\*|([1-9]|1[0-2]))(\/\d+)?){1}(\s+(\*|([0-7]))(\/\d+)?){1}$/;
  return cronRegex.test(cron);
};

// 当高级模式的cron表达式变化时，验证并可能更新简单模式
watch(cronExpression, (newVal) => {
  if (!validateCron(newVal)) {
    cronError.value = "无效的cron表达式";
  } else {
    cronError.value = "";
    if (activeTab.value === "advanced") {
      updateSimpleFromCron(newVal);
    }
  }
});

// 只在每天、每周、每月的模式下显示时间输入
const showTimeInput = computed(() => {
  return ["daily", "weekly", "monthly"].includes(frequency.value);
});

// 打开时间选择器
const openTimePicker = () => {
  // 从当前时间输入解析小时和分钟
  const [hour, minute] = timeInput.value.split(':').map(Number);
  selectedHour.value = hour;
  // 找到最接近的分钟选项
  const closestMinute = minuteOptions.reduce((prev, curr) => {
    return (Math.abs(curr - minute) < Math.abs(prev - minute)) ? curr : prev;
  });
  selectedMinute.value = closestMinute;
  
  timePickerMenu.value = true;
  
  // 打开菜单后，等待DOM更新，然后滚动到选中的时间
  nextTick(() => {
    scrollToSelectedOption('hours-column', hour);
    scrollToSelectedOption('minutes-column', closestMinute);
  });
};

// 选择小时
const selectHour = (hour: number) => {
  selectedHour.value = hour;
  // 滚动到视图中心
  scrollToSelectedOption('hours-column', hour);
};

// 选择分钟
const selectMinute = (minute: number) => {
  selectedMinute.value = minute;
  // 滚动到视图中心
  scrollToSelectedOption('minutes-column', minuteOptions.indexOf(minute));
};

// 滚动到选中的选项
const scrollToSelectedOption = (columnClass: string, value: number) => {
  setTimeout(() => {
    const column = document.querySelector(`.${columnClass}`);
    if (!column) return;
    
    // 每个选项的高度大约为40px
    const optionHeight = 40;
    // 将滚动位置设置为选中值的位置，使其在视图中央
    const scrollTop = value * optionHeight - (column.clientHeight / 2) + (optionHeight / 2);
    column.scrollTop = Math.max(0, scrollTop);
  }, 50);
};

// 确认小时分钟选择
const confirmHourMinuteSelection = () => {
  const formattedHour = selectedHour.value.toString().padStart(2, '0');
  const formattedMinute = selectedMinute.value.toString().padStart(2, '0');
  timeInput.value = `${formattedHour}:${formattedMinute}`;
  selectedTime.value = timeInput.value;
  timePickerMenu.value = false;
};

// 取消时间选择
const cancelTimeSelection = () => {
  timePickerMenu.value = false;
};

// 描述当前cron表达式
const cronDescription = computed(() => {
  try {
    const parts = cronExpression.value.split(" ");
    const minute = parts[0];
    const hour = parts[1];
    const dom = parts[2];
    const month = parts[3];
    const dow = parts[4];

    if (hour === "*" && minute === "*") {
      return "每分钟执行一次";
    }
    if (
      hour !== "*" &&
      minute !== "*" &&
      dom === "*" &&
      month === "*" &&
      dow === "*"
    ) {
      return `每天 ${hour}:${minute} 执行一次`;
    }
    if (
      hour !== "*" &&
      minute !== "*" &&
      dom === "*" &&
      month === "*" &&
      dow !== "*"
    ) {
      const days = dow.split(",").map((d) => {
        const dayIndex = Number.parseInt(d, 10) % 7;
        return (
          weekdays.find((w) => w.value === dayIndex)?.text || `周${dayIndex}`
        );
      });
      return `每周 ${days.join(",")} 的 ${hour}:${minute} 执行一次`;
    }
    if (
      hour !== "*" &&
      minute !== "*" &&
      dom !== "*" &&
      month === "*" &&
      dow === "*"
    ) {
      return `每月 ${dom} 日的 ${hour}:${minute} 执行一次`;
    }
    return `Cron表达式: ${cronExpression.value}`;
  } catch (error) {
    return "无法解析cron表达式";
  }
});

// 计算下一次执行时间
const nextExecutionTime = computed(() => {
  try {
    // 简单的下一次执行时间计算
    const now = new Date();
    const parts = cronExpression.value.split(" ");
    const minute =
      parts[0] === "*" ? now.getMinutes() : Number.parseInt(parts[0], 10);
    const hour =
      parts[1] === "*" ? now.getHours() : Number.parseInt(parts[1], 10);
    const dayOfMonth =
      parts[2] === "*" ? now.getDate() : Number.parseInt(parts[2], 10);
    const dayOfWeek =
      parts[4] === "*"
        ? null
        : parts[4].split(",").map((d) => Number.parseInt(d, 10));

    const next = new Date();
    next.setSeconds(0);
    next.setMilliseconds(0);

    // 设置分钟和小时
    next.setMinutes(minute);
    next.setHours(hour);

    // 如果是按周执行
    if (dayOfWeek && dayOfWeek.length > 0) {
      // 找到下一个匹配的星期几
      let daysToAdd = 0;
      const currentDayOfWeek = next.getDay();
      const sortedDays = [...dayOfWeek].sort((a, b) => a - b);

      // 寻找今天之后的下一个执行日
      for (const day of sortedDays) {
        // JS中周日是0，我们的组件中周日是0，周一是1
        const normalizedDay = day;
        if (
          normalizedDay > currentDayOfWeek ||
          (normalizedDay === currentDayOfWeek && next <= now)
        ) {
          daysToAdd = normalizedDay - currentDayOfWeek;
          break;
        }
      }

      // 如果没有找到今天之后的日期，则取下一周的第一个日期
      if (daysToAdd === 0 && next <= now) {
        daysToAdd = 7 - currentDayOfWeek + sortedDays[0];
      }

      next.setDate(next.getDate() + daysToAdd);
    }
    // 如果是按月执行
    else if (parts[2] !== "*") {
      next.setDate(dayOfMonth);

      // 如果设置的日期已经过了这个月的这一天，移到下个月
      if (next <= now) {
        next.setMonth(next.getMonth() + 1);
      }
    }
    // 如果是每天执行
    else {
      // 如果今天的执行时间已过，移到明天
      if (next <= now) {
        next.setDate(next.getDate() + 1);
      }
    }

    return next.toLocaleString("zh-CN", {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch (error) {
    console.error("计算下次执行时间失败", error);
    return "无法计算下次执行时间";
  }
});

// 确认选择
const confirm = () => {
  if (cronError.value) {
    return; // 如果有错误，不执行确认
  }
  emit("confirm", cronExpression.value);
  close();
};

// 关闭对话框
const close = () => {
  dialog.value = false;
  emit("close");
};

// 对外暴露方法
defineExpose({
  open: () => {
    dialog.value = true;
  },
  getCurrentCron: () => cronExpression.value,
});
</script>

<style scoped>
.cron-picker {
  width: 100%;
}

.field-label-wrapper {
  font-size: 0.875rem;
  color: rgba(0, 0, 0, 0.6);
  margin-bottom: 4px;
  font-weight: normal;
  line-height: 1;
  letter-spacing: 0.0071428571em;
  position: relative;
  z-index: 1;
}

/* 暗色模式适配 */
:deep(.v-theme--dark) .field-label-wrapper {
  color: rgba(255, 255, 255, 0.7);
}

/* 时间选择器样式 */
.time-selector-container {
  width: 100%;
}

.time-selector-wrapper {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.time-selector-header {
  display: flex;
  width: 100%;
  justify-content: space-around;
  padding: 8px 0;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

.time-selector-label {
  flex: 1;
  text-align: center;
  font-weight: 500;
}

.time-selector-body {
  display: flex;
  width: 100%;
  height: 200px;
}

.time-selector-column {
  flex: 1;
  overflow-y: auto;
  scroll-behavior: smooth;
  border-right: 1px solid rgba(0, 0, 0, 0.05);
}

.time-selector-column:last-child {
  border-right: none;
}

.time-selector-option {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.time-selector-option:hover {
  background-color: rgba(var(--v-theme-primary), 0.05);
}

.time-selector-option.selected {
  background-color: rgba(var(--v-theme-primary), 0.12);
  color: rgb(var(--v-theme-primary));
  font-weight: 500;
}

/* 滚动条样式 */
.time-selector-column::-webkit-scrollbar {
  width: 4px;
}

.time-selector-column::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
}

.time-selector-column::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 2px;
}

:deep(.v-theme--dark) .time-selector-header {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

:deep(.v-theme--dark) .time-selector-column {
  border-right: 1px solid rgba(255, 255, 255, 0.05);
}

:deep(.v-theme--dark) .time-selector-column::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}

:deep(.v-theme--dark) .time-selector-column::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
}
</style>
