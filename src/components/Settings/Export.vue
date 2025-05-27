<template>
  <v-expansion-panel>
    <v-expansion-panel-title>
      <h3 class="text-h6 font-weight-medium">数据导出</h3>
    </v-expansion-panel-title>
    <v-expansion-panel-text>
      <v-row>
        <v-col cols="12" md="6">
          <v-select
            v-model="Format"
            :items="exportFormats"
            label="导出格式"
            variant="outlined"
            density="compact"
            class="mb-4"
          ></v-select>
        </v-col>
        <v-col cols="12" md="6">
          <v-select
            v-model="Filter"
            :items="exportFilterOptions"
            label="导出筛选"
            variant="outlined"
            density="compact"
            class="mb-4"
          ></v-select>
        </v-col>

        <v-col cols="12" md="4">
          <v-btn
            block
            color="primary"
            @click="exportAll"
            :loading="inprogress"
            class="mb-3"
          >
            导出全部事件
          </v-btn>
        </v-col>
        <v-col cols="12" md="4">
          <v-btn
            block
            color="info"
            @click="exportFiltered"
            :loading="inprogress"
            class="mb-3"
          >
            按筛选导出
          </v-btn>
        </v-col>

        <v-col cols="12" md="4">
          <v-btn
            block
            outlined
            @click="showSelectionDialog"
            :disabled="inprogress"
            class="mb-3"
          >
            选择事件导出
          </v-btn>
        </v-col>
      </v-row>

      <v-alert
        v-if="result"
        :color="result.success ? 'success' : 'error'"
        :title="result.success ? '导出成功' : '导出失败'"
        :text="result.message"
        class="mt-3"
        variant="tonal"
        closable
        @click:close="clearResult"
      ></v-alert>
    </v-expansion-panel-text>
  </v-expansion-panel>
</template>

<script setup lang="ts">
import { computed } from "vue";

// 导出格式选项
const exportFormats = [
  { title: "iCalendar (.ics)", value: "ics" },
  { title: "JSON (.json)", value: "json" },
  { title: "Markdown (.md)", value: "md" },
];

// 导出筛选选项
const exportFilterOptions = [
  { title: "全部事件", value: "all" },
  { title: "已完成事件", value: "completed" },
  { title: "未完成事件", value: "pending" },
];

// 状态
interface ExportSettings {
  format: string;
  filter: string;
  inProgress: boolean;
  result: { success: boolean; message: string } | null;
  showDialog: boolean;
}

const props = defineProps<{
  settings: ExportSettings;
}>();

const createSettingComputed = <K extends keyof ExportSettings>(
  key: K,
  defaultValue?: ExportSettings[K]
) => {
  return computed({
    get: () => props.settings[key] ?? defaultValue,
    set: (value: ExportSettings[K]) => {
      updateSettings(key, value);
    },
  });
};

const Format = createSettingComputed("format", "ics");
const Filter = createSettingComputed("filter", "all");
const inprogress = createSettingComputed("inProgress", false);
const result = createSettingComputed("result", null);

// 事件
const emit = defineEmits([
  "export-all",
  "export-filtered",
  "show-selection-dialog",
  "update:settings",
]);

// 方法
const exportAll = () => {
  updateSettings("inProgress", true);
  emit("export-all");
};

const exportFiltered = () => {
  updateSettings("inProgress", true);
  emit("export-filtered");
};

const updateSettings = <K extends keyof ExportSettings>(
  key: K,
  value: ExportSettings[K]
) => {
  emit("update:settings", {
    [key]: value,
  });
};

const showSelectionDialog = () => {
  updateSettings("showDialog", true);
};

const clearResult = () => {
  updateSettings("result", null);
};

// 提供方法给父组件调用
defineExpose({
  setExporting: (value: boolean) => {
    updateSettings("inProgress", value);
  },
  setResult: (value: { success: boolean; message: string } | null) => {
    updateSettings("result", value);
  },
});
</script>
