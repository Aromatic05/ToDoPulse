<template>
  <div class="timeline-view">
    <v-timeline side="end" align="start" class="timeline-force-left" line-color=var(--md-sys-color-outline)>
      <!-- 当没有时间线组时显示的内容 -->
      <template v-if="showedTimelineGroups.length === 0">
        <v-timeline-item dot-color="success" size="large" fill-dot>
          <template v-slot:icon>
            <v-avatar color="success">
              <v-icon color="white">mdi-party-popper</v-icon>
            </v-avatar>
          </template>
          <div class="timeline-empty-container">
            <div class="timeline-empty-title">
              恭喜你完成了所有任务!
            </div>
            <div class="timeline-empty-content">
              <p>计划已全部完成，休息一下，享受成就感吧!</p>
            </div>
          </div>
        </v-timeline-item>
      </template>

      <template v-else v-for="group in showedTimelineGroups" :key="group.id">
        <!-- 时间线组标题 - 从服务获取 -->
        <v-timeline-item :dot-color="group.color" size="large" fill-dot>
          <template v-slot:icon>
            <v-avatar :color="group.color">
              <v-icon color="white">{{ group.iconName }}</v-icon>
            </v-avatar>
          </template>
          <div class="timeline-group-title">{{ group.title }}</div>
        </v-timeline-item>

        <!-- 该组的所有项目 -->
        <v-timeline-item v-for="item in groupItems[group.dateGroup] || []" :key="item.id" :dot-color="item.color"
          :icon="item.icon" size="small" density="compact">
          <EventCard :data="timelineStore.formatCardData(item, group.dateGroup)"
            @update="(data: FEvent) => handleUpdateItem(data, group.dateGroup)" />
        </v-timeline-item>
      </template>
    </v-timeline>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import EventCard from '@/components/Cards/EventCard.vue'
import { FEvent } from 'src-tauri/bindings/FEvent';
import { useTimelineStore } from '@/stores'

// 使用Pinia store管理时间线数据
const timelineStore = useTimelineStore();
const showedTimelineGroups = computed(() => timelineStore.showedTimelineGroups);
// const isLoading = computed(() => timelineStore.isLoading);
const groupItems = computed(() => {
  const result: Record<string, FEvent[]> = {};
  for (const group of showedTimelineGroups.value) {
    result[group.dateGroup] = timelineStore.getGroupItems(group.dateGroup);
  }
  return result;
});

// 组件挂载时初始化数据
onMounted(async () => {
  // 从store加载时间线数据
  if (!timelineStore.dataInitialized) {
    await timelineStore.fetchEvents();
  }
});

// 更新项目处理函数
async function handleUpdateItem(data: FEvent, dateGroup: string) {
  await timelineStore.updateEvent(data, dateGroup);
}
</script>

<style scoped>
.timeline-view {
  width: 100% !important;
  max-width: 900px !important;
  margin: 0 auto !important;
  padding: 16px 16px 0 16px !important;
}

/* 强制时间线左对齐并占满宽度 */
:deep(.timeline-force-left) {
  width: 100% !important;
  margin-left: 0 !important;
  padding-left: 0 !important;
  justify-content: flex-start !important;
}

/* 强制整个时间线组件占满容器宽度 */
:deep(.v-timeline) {
  width: 100% !important;
  max-width: 100% !important;
  padding: 0 !important;
}

/* 时间线内的其他样式 */
.timeline-group-title {
  font-size: 18px;
  font-weight: 600;
  margin: 4px 0;
  color: var(--md-sys-color-on-surface);
}

/* 强制时间线项目占满可用空间 */
:deep(.v-timeline-item) {
  width: 100% !important;
  margin-bottom: -12px;
}

/* 让时间线项目的布局更紧凑 */
:deep(.v-timeline-item) {
  margin-bottom: -12px;
}

/* 让带有组标题的项目有适当的间距 */
:deep(.v-timeline-item--fill-dot) {
  margin-top: 24px;
  margin-bottom: 8px;
}

/* 这是最关键的部分 - 时间线项的内容区域 */
:deep(.v-timeline-item__body) {
  width: calc(100% - 36px) !important;
  /* 减去图标和间距的宽度 */
  max-width: none !important;
  padding-right: 0 !important;
}

/* 给时间线项内的内容增加水平空间 */
:deep(.v-timeline-item__opposite),
:deep(.v-timeline-item__content) {
  width: 100% !important;
  max-width: 100% !important;
  flex: 1 1 auto !important;
}

/* 空时间线样式 */
.timeline-empty-container {
  padding: 20px;
  border-radius: 12px;
  margin: 8px 0;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  border-left: 4px solid var(--md-sys-color-success);
}

.timeline-empty-title {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 16px;
  color: var(--md-sys-color-on-surface);
  display: flex;
  align-items: center;
}

.timeline-empty-content {
  color: var(--md-sys-color-on-surface-variant);
  font-size: 16px;
  text-align: center;
  padding: 16px 0;
}
</style>