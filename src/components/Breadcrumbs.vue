<template>
  <div class="header_title pl-4">
    <v-breadcrumbs :items="items" class="pa-0">
      <template v-slot:prepend>
        <v-icon icon="mdi-home-outline" size="small"></v-icon>
      </template>
      <template v-slot:divider>
        <v-icon icon="mdi-chevron-right" size="small"></v-icon>
      </template>
      <template v-slot:title="{ item }">
        <span :class="{ 'font-weight-medium': item.disabled }">{{ item.title }}</span>
      </template>
    </v-breadcrumbs>
    <div class="page_title">
      {{ currentPageTitle }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

// 当前面包屑路径
const breadcrumbPath = ref([
  { title: '首页', disabled: false },
  { title: '任务管理', disabled: false },
  { title: '今日任务', disabled: true }
]);

// 面包屑项目（用于展示）
const items = computed(() => {
  return breadcrumbPath.value.map(item => ({
    title: item.title,
    disabled: item.disabled,
  }));
});

// 当前页面标题（最后一项）
const currentPageTitle = computed(() => {
  const lastItem = breadcrumbPath.value[breadcrumbPath.value.length - 1];
  return lastItem ? lastItem.title : '';
});

// 在实际应用中，你可以添加方法来更新面包屑路径
// 例如：
// function updateBreadcrumbs(path: Array<{title: string, disabled: boolean}>) {
//   breadcrumbPath.value = path;
// }
</script>

<style scoped>
.header_title {
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.page_title {
  margin-top: 4px;
  font-size: 16px;
  font-weight: 700;
}

/* 避免图标与文字间距过大 */
:deep(.v-breadcrumbs-item) {
  padding: 0 8px;
}

:deep(.v-breadcrumbs-divider) {
  padding: 0 4px;
}

:deep(.v-icon) {
  opacity: 0.7;
}

:deep(.v-breadcrumbs-item--disabled) {
  opacity: 0.6;
}
</style>