<template>
    <div class="timeline-view">
        <v-timeline side="end" align="start" class="timeline-force-left"
            line-color=var(--md-sys-color-outline)>
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
                <EventCard :data="formatCardData(item)" @update="updateItem" />
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
                <EventCard :data="formatCardData(item)" @update="updateItem" />
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
                <EventCard :data="formatCardData(item)" @update="updateItem" />
            </v-timeline-item>
        </v-timeline>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import EventCard from '@/components/Cards/EventCard.vue'

// 定义类型接口
interface TimelineItem {
    id: number;
    title: string;
    description: string;
    time: string;
    color: string;
    icon: string;
    dateGroup: 'today' | 'tomorrow' | 'next-week';
    isCompleted: boolean;
}

interface EventCardData {
    id: number;
    title: string;
    content: string;
    date: string;
    dateColor: string;
    isCompleted: boolean;
    tags: string[];
}

// 所有时间线项目数据
const timelineItems = ref<TimelineItem[]>([
    {
        id: 1,
        title: '完成项目规划',
        description: '讨论并确定项目范围、目标和里程碑',
        time: '今天 09:00',
        color: 'primary',
        icon: 'mdi-file-document',
        dateGroup: 'today',
        isCompleted: false
    },
    {
        id: 2,
        title: '团队会议',
        description: '周会：讨论本周工作进展和问题',
        time: '今天 14:00',
        color: 'secondary',
        icon: 'mdi-account-group',
        dateGroup: 'today',
        isCompleted: false
    },
    {
        id: 3,
        title: '提交代码审查',
        description: '提交新功能的代码审查请求',
        time: '明天 10:30',
        color: 'success',
        icon: 'mdi-code-tags',
        dateGroup: 'tomorrow',
        isCompleted: false
    },
    {
        id: 4,
        title: 'UI设计评审',
        description: '评审新界面设计和用户体验改进',
        time: '明天 15:00',
        color: 'info',
        icon: 'mdi-palette',
        dateGroup: 'tomorrow',
        isCompleted: false
    },
    {
        id: 5,
        title: '项目进度汇报',
        description: '向管理层汇报项目进展情况',
        time: '下周一 11:00',
        color: 'warning',
        icon: 'mdi-chart-timeline',
        dateGroup: 'next-week',
        isCompleted: false
    },
    {
        id: 6,
        title: '产品发布准备',
        description: '准备产品发布材料和营销内容',
        time: '下周三 09:30',
        color: 'error',
        icon: 'mdi-rocket-launch',
        dateGroup: 'next-week',
        isCompleted: false
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

// 将timeline数据格式转换为EventCard所需的格式
const formatCardData = (item: TimelineItem): EventCardData => {
    return {
        id: item.id,
        title: item.title,
        content: item.description,
        date: item.time,
        dateColor: getColorVariable(item.color),
        isCompleted: item.isCompleted || false,
        tags: [item.dateGroup] // 使用dateGroup作为标签
    }
}

// 将颜色名称转换为CSS变量
const getColorVariable = (color: string): string => {
    const colorMap: Record<string, string> = {
        'primary': 'var(--md-sys-color-primary)',
        'secondary': 'var(--md-sys-color-secondary)',
        'info': 'var(--md-sys-color-tertiary)',
        'success': 'var(--md-sys-color-success)',
        'warning': 'var(--md-sys-color-warning)',
        'error': 'var(--md-sys-color-error)'
    }
    return colorMap[color] || 'var(--md-sys-color-on-surface-variant)'
}

// 更新项目的回调函数
const updateItem = (updatedData: EventCardData): void => {
    const index = timelineItems.value.findIndex(item => item.id === updatedData.id)
    if (index !== -1) {
        // 从EventCard格式转回timeline格式
        timelineItems.value[index] = {
            ...timelineItems.value[index],
            title: updatedData.title,
            description: updatedData.content,
            isCompleted: updatedData.isCompleted
        }
    }
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

</style>