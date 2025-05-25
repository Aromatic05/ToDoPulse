<template>
  <v-card class="pa-4">
    <h3 class="text-h6 mb-5">数据导出</h3>

    <v-row>
      <v-col cols="12" md="6">
        <v-select 
          v-model="selectedFormat" 
          :items="exportFormats" 
          label="导出格式"
          variant="outlined" 
          density="compact" 
          class="mb-4"
        ></v-select>
      </v-col>
      <v-col cols="12" md="6">
        <v-select 
          v-model="selectedFilter" 
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
          :loading="isExporting"
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
          :loading="isExporting"
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
          :disabled="isExporting"
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
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';

// 导出格式选项
const exportFormats = [
  { title: 'iCalendar (.ics)', value: 'ics' },
  { title: 'JSON (.json)', value: 'json' },
  { title: 'Markdown (.md)', value: 'md' }
];

// 导出筛选选项
const exportFilterOptions = [
  { title: '全部事件', value: 'all' },
  { title: '已完成事件', value: 'completed' },
  { title: '未完成事件', value: 'pending' }
];

// 状态
const selectedFormat = ref('ics');
const selectedFilter = ref('all');
const isExporting = ref(false);
const result = ref<{ success: boolean; message: string } | null>(null);

// 事件
const emit = defineEmits([
  'export-all', 
  'export-filtered', 
  'show-selection-dialog', 
  'update:exporting',
  'update:exportResult',
  'update:exportFormat',
  'update:exportFilter'
]);

// 方法
const exportAll = () => {
  isExporting.value = true;
  emit('update:exporting', true);
  emit('update:exportFormat', selectedFormat.value);
  emit('update:exportFilter', selectedFilter.value);
  emit('export-all');
};

const exportFiltered = () => {
  isExporting.value = true;
  emit('update:exporting', true);
  emit('update:exportFormat', selectedFormat.value);
  emit('update:exportFilter', selectedFilter.value);
  emit('export-filtered');
};

const showSelectionDialog = () => {
  emit('show-selection-dialog');
  emit('update:exportFormat', selectedFormat.value);
};

const clearResult = () => {
  result.value = null;
  emit('update:exportResult', null);
};

// 提供方法给父组件调用
defineExpose({
  setExporting: (value: boolean) => {
    isExporting.value = value;
  },
  setResult: (value: { success: boolean; message: string } | null) => {
    result.value = value;
  }
});
</script>