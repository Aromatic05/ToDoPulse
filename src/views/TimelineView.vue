<template>
    <div class="timeline-view">
        <v-timeline side="end" align="start" class="timeline-force-left">
            <!-- 今天的事项 -->
            <v-timeline-item dot-color="primary" size="large" fill-dot>
                <template v-slot:icon>
                    <v-avatar color="primary">
                        <v-icon color="white">mdi-calendar-today</v-icon>
                    </v-avatar>
                </template>
                <div class="timeline-group-title">今天</div>
            </v-timeline-item>

            <v-timeline-item v-for="item in todayItems" :key="item.id" :dot-color="item.color" :icon="item.icon"
                size="small" density="compact">
                <v-card class="elevation-1">
                    <v-card-title class="text-h6">
                        {{ item.title }}
                    </v-card-title>
                    <v-card-text>
                        {{ item.description }}
                        <div class="text-caption mt-2">
                            {{ item.time }}
                        </div>
                    </v-card-text>
                </v-card>
            </v-timeline-item>

            <!-- 明天的事项 -->
            <v-timeline-item dot-color="secondary" size="large" fill-dot>
                <template v-slot:icon>
                    <v-avatar color="secondary">
                        <v-icon color="white">mdi-calendar-arrow-right</v-icon>
                    </v-avatar>
                </template>
                <div class="timeline-group-title">明天</div>
            </v-timeline-item>

            <v-timeline-item v-for="item in tomorrowItems" :key="item.id" :dot-color="item.color" :icon="item.icon"
                size="small" density="compact">
                <v-card class="elevation-1">
                    <v-card-title class="text-h6">
                        {{ item.title }}
                    </v-card-title>
                    <v-card-text>
                        {{ item.description }}
                        <div class="text-caption mt-2">
                            {{ item.time }}
                        </div>
                    </v-card-text>
                </v-card>
            </v-timeline-item>

            <!-- 下周的事项 -->
            <v-timeline-item dot-color="info" size="large" fill-dot>
                <template v-slot:icon>
                    <v-avatar color="info">
                        <v-icon color="white">mdi-calendar-week</v-icon>
                    </v-avatar>
                </template>
                <div class="timeline-group-title">下周</div>
            </v-timeline-item>

            <v-timeline-item v-for="item in nextWeekItems" :key="item.id" :dot-color="item.color" :icon="item.icon"
                size="small" density="compact">
                <v-card class="elevation-1">
                    <v-card-title class="text-h6">
                        {{ item.title }}
                    </v-card-title>
                    <v-card-text>
                        {{ item.description }}
                        <div class="text-caption mt-2">
                            {{ item.time }}
                        </div>
                    </v-card-text>
                </v-card>
            </v-timeline-item>
        </v-timeline>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// 所有时间线项目数据
const timelineItems = ref([
    {
        id: 1,
        title: '完成项目规划',
        description: '讨论并确定项目范围、目标和里程碑',
        time: '今天 09:00',
        color: 'primary',
        icon: 'mdi-file-document',
        dateGroup: 'today'
    },
    {
        id: 2,
        title: '团队会议',
        description: '周会：讨论本周工作进展和问题',
        time: '今天 14:00',
        color: 'secondary',
        icon: 'mdi-account-group',
        dateGroup: 'today'
    },
    {
        id: 3,
        title: '提交代码审查',
        description: '提交新功能的代码审查请求',
        time: '明天 10:30',
        color: 'success',
        icon: 'mdi-code-tags',
        dateGroup: 'tomorrow'
    },
    {
        id: 4,
        title: 'UI设计评审',
        description: '评审新界面设计和用户体验改进',
        time: '明天 15:00',
        color: 'info',
        icon: 'mdi-palette',
        dateGroup: 'tomorrow'
    },
    {
        id: 5,
        title: '项目进度汇报',
        description: '向管理层汇报项目进展情况',
        time: '下周一 11:00',
        color: 'warning',
        icon: 'mdi-chart-timeline',
        dateGroup: 'next-week'
    },
    {
        id: 6,
        title: '产品发布准备',
        description: '准备产品发布材料和营销内容',
        time: '下周三 09:30',
        color: 'error',
        icon: 'mdi-rocket-launch',
        dateGroup: 'next-week'
    }
])

// 按日期分组的计算属性
const todayItems = computed(() => {
    return timelineItems.value.filter(item => item.dateGroup === 'today')
})

const tomorrowItems = computed(() => {
    return timelineItems.value.filter(item => item.dateGroup === 'tomorrow')
})

const nextWeekItems = computed(() => {
    return timelineItems.value.filter(item => item.dateGroup === 'next-week')
})
</script>

<style scoped>
.timeline-view {
    width: 100% !important;
    margin: 0 !important;
    padding: 16px 0 0 0 !important;
    max-width: none !important;
}

/* 强制时间线左对齐 */
:deep(.timeline-force-left) {
    width: 100% !important;
    margin-left: 0 !important;
    padding-left: 0 !important;
    justify-content: flex-start !important;
}

/* 强制容器内所有元素左对齐 */
:deep(.v-timeline-item) {
    width: 100% !important;
    margin-left: 0 !important;
    padding-left: 0 !important;
}

:deep(.v-timeline-divider) {
    justify-content: flex-start !important;
}

.timeline-group-title {
    font-size: 18px;
    font-weight: 600;
    margin: 4px 0;
    color: var(--md-sys-color-on-surface);
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

/* 确保卡片右侧有适当的间距 */
:deep(.v-card) {
    margin-right: 16px;
}
</style>