<template>
    <v-card class="m-event-card mb-2" variant="tonal" :class="{ 'completed-card': localData.finished }"
        @click="handleCardClick" ripple>
        <div class="d-flex align-center pa-2">
            <!-- 任务状态复选框 - 增大触摸区域 -->
            <div class="checkbox-container mr-2" @click.stop="handleComplete(localData)">
                <v-checkbox v-model="localData.finished" density="compact" hide-details class="ma-0 pa-0"
                    color="primary" @click.prevent></v-checkbox>
            </div>

            <!-- 任务内容区域 -->
            <div class="task-content flex-grow-1">
                <!-- 任务标题 -->
                <div class="text-subtitle-1 font-weight-medium mb-1"
                    :class="{ 'text-decoration-line-through': localData.finished }">
                    {{ localData.title }}
                </div>

                <!-- 截止时间和标签 -->
                <div class="d-flex flex-wrap align-center">
                    <!-- 截止时间 -->
                    <span v-if="ddlTime" class="time-chip mr-2"
                        :style="{ color: localData.color || 'var(--md-sys-color-on-surface-variant)' }">
                        <v-icon size="16" class="mr-1">mdi-clock-outline</v-icon>
                        {{ ddlTime }}
                    </span>

                    <!-- 标签展示 - 移动端优化：只显示最多2个 -->
                    <div v-if="localData.tag?.length" class="d-flex">
                        <span v-for="(tag, i) in displayTags" :key="tag || i" class="tag-chip mr-1">{{ tag }}</span>
                        <span v-if="localData.tag.length > 2" class="more-tags">
                            +{{ localData.tag.length - 2 }}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    </v-card>

    <!-- 优化模态框条件渲染 -->
    <CardContentModal v-if="showModal" v-model="showModal" :card-data="localData" @confirm="handleConfirm" />
</template>

<script lang="ts">
import { defineComponent, defineAsyncComponent, computed } from 'vue';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { convertTimestampToTime } from '@/services/DateTimeService';

// 动态导入模态框组件，减少初始加载时间
const CardContentModal = defineAsyncComponent(() =>
    import('@/components/Modals/CardContentModal.vue')
);

// 时间戳缓存对象
const timeCache = new Map<string, string | null>();

export default defineComponent({
    name: 'MEventCard',
    components: {
        CardContentModal
    },
    props: {
        data: {
            type: Object as () => FEvent,
            required: true,
            validator: (value: FEvent) => !!(value?.id && value?.title),
        }
    },
    emits: ['update', 'delete', 'toggleStatus'],
    setup(props) {
        // 计算属性，用于缓存时间显示
        const ddlTime = computed(() => {
            if (!props.data.ddl) return null;

            // 从缓存中获取时间
            if (timeCache.has(props.data.ddl)) {
                return timeCache.get(props.data.ddl);
            }

            // 计算并缓存结果
            const result = convertTimestampToTime(props.data.ddl);
            timeCache.set(props.data.ddl, result);
            return result;
        });

        // 计算属性，限制显示的标签数量
        const displayTags = computed(() => {
            if (!props.data.tag || props.data.tag.length === 0) return [];
            // 移动端只显示前2个标签
            return props.data.tag.slice(0, 2);
        });

        return {
            ddlTime,
            displayTags
        };
    },
    data() {
        return {
            showModal: false,
            // 避免使用昂贵的JSON解析，使用展开运算符进行浅拷贝
            localData: {
                ...this.data,
                finished: this.data.finished || false
            } as FEvent,
            // 添加防抖定时器引用
            _clickTimeout: null as NodeJS.Timeout | null
        }
    },
    methods: {
        handleCardClick() {
            this.showModal = true;
        },
        handleConfirm(updatedData: FEvent) {
            this.localData = { ...updatedData };
            this.$emit('update', updatedData);
        },
        handleComplete(updatedData: FEvent) {
            if (this._clickTimeout) return;

            this._clickTimeout = setTimeout(() => {
                this.localData.finished = !this.localData.finished;
                this.localData = { ...updatedData };
                this.$emit('update', updatedData);
                this._clickTimeout = null;
            }, 100);
        },
    },
    // 添加beforeUnmount钩子清理可能的定时器
    beforeUnmount() {
        if (this._clickTimeout) {
            clearTimeout(this._clickTimeout);
        }
    }
});
</script>

<style scoped>
/* 移动卡片基本样式 */
.m-event-card {
    border-radius: 12px;
    transition: all 0.2s ease;
    contain: content;
    /* 提升渲染性能 */
    will-change: transform;
    overflow: hidden;
    border-radius: 18px;
    background: var(--md-sys-color-surface-container);
    border: 2px solid var(--md-sys-color-outline);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
}

.m-event-card:active {
    transform: scale(0.98);
    /* 触摸反馈效果 */
}

/* 已完成状态样式 */
.completed-card {
    opacity: 0.75;
    background-color: var(--v-theme-surface) !important;
}

/* 优化复选框区域 - 增大触摸区域 */
.checkbox-container {
    min-width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* 任务内容样式 */
.task-content {
    min-width: 0;
    /* 允许内容区域缩小 */
    overflow: hidden;
}

/* 截止时间样式 */
.time-chip {
    display: inline-flex;
    align-items: center;
    font-size: 0.75rem;
    height: 20px;
    border-radius: 10px;
    padding: 0 8px;
    white-space: nowrap;
}

/* 标签样式 */
.tag-chip {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 10px;
    background-color: rgba(var(--v-theme-primary), 0.1);
    color: rgb(var(--v-theme-primary));
    white-space: nowrap;
}

/* 更多标签提示 */
.more-tags {
    font-size: 0.75rem;
    opacity: 0.7;
    margin-left: 4px;
}

/* 确保文本不溢出 */
.text-subtitle-1 {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* 优化性能 */
:deep(.v-checkbox) {
    margin: 0;
    padding: 0;
}

/* 适应较小的屏幕 */
@media (max-width: 320px) {
    .task-content {
        max-width: 200px;
        /* 限制在极小屏幕上的宽度 */
    }
}
</style>