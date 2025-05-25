<template>
  <v-dialog v-model="dialogVisible" max-width="600">
    <v-card>
      <v-card-title class="text-h5">选择要导出的事件</v-card-title>
      <v-card-text>
        <v-data-table 
          v-model="selected" 
          :headers="headers" 
          :items="items" 
          show-select
          item-value="id"
        ></v-data-table>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="grey-darken-1" variant="text" @click="cancel">
          取消
        </v-btn>
        <v-btn 
          color="primary" 
          variant="text" 
          @click="confirm"
          :disabled="selected.length === 0"
        >
          导出所选事件
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

// 属性
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  items: {
    type: Array,
    default: () => []
  }
});

// 事件
const emit = defineEmits(['update:modelValue', 'export-selected']);

// 本地状态
const dialogVisible = ref(props.modelValue);
const selected = ref<any[]>([]);
const headers = [
  { title: '标题', key: 'title' },
];

// 监听属性变化
watch(() => props.modelValue, (newVal) => {
  dialogVisible.value = newVal;
});

watch(() => dialogVisible.value, (newVal) => {
  emit('update:modelValue', newVal);
  if (!newVal) {
    selected.value = [];
  }
});

// 方法
const cancel = () => {
  dialogVisible.value = false;
};

const confirm = () => {
  emit('export-selected', selected.value);
  dialogVisible.value = false;
};
</script>