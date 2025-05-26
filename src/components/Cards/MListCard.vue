<template>
    <v-card class="mb-2 m-list-card" variant="tonal" :class="{ 'completed-card': localData.finished }">
        <div class="d-flex align-center pa-2">
            <!-- 左侧区域：复选框和标题 -->
            <div class="d-flex align-center flex-grow-1">
                <!-- 状态复选框 - 优化触摸区域 -->
                <div class="checkbox-container mr-2" @click.stop="handleComplete(localData)">
                    <v-checkbox
                        v-model="localData.finished"
                        density="compact"
                        hide-details
                        class="ma-0 pa-0"
                        color="primary"
                        @click.stop
                    ></v-checkbox>
                </div>

                <!-- 标题和截止日期 -->
                <div class="task-info-container" @click="handleEdit">
                    <div class="d-flex justify-space-between align-center">
                        <h3 class="text-subtitle-1 font-weight-medium mb-1" 
                            :class="{ 'text-decoration-line-through': localData.finished }">
                            {{ localData.title }}
                        </h3>
                    </div>

                    <!-- 截止日期和标签 -->
                    <div class="d-flex flex-wrap align-center">
                        <!-- 截止日期 - 优先显示 -->
                        <span v-if="localData.ddl" class="text-caption mr-2"
                            :style="{ color: localData.color || 'var(--md-sys-color-on-surface-variant)' }">
                            <v-icon size="12" class="mr-1">mdi-calendar</v-icon>
                            {{ formattedDate }}
                        </span>
                    </div>
                </div>
            </div>

            <!-- 右侧操作区域 - 优化为单个按钮 -->
            <v-menu location="bottom end">
                <template v-slot:activator="{ props }">
                    <v-btn icon size="small" v-bind="props" variant="text">
                        <v-icon>mdi-dots-vertical</v-icon>
                    </v-btn>
                </template>
                <v-list density="compact">
                    <v-list-item @click="handleEdit" prepend-icon="mdi-pencil">
                        <v-list-item-title>编辑</v-list-item-title>
                    </v-list-item>
                    <v-list-item @click="handleDelete" prepend-icon="mdi-delete" color="error">
                        <v-list-item-title>删除</v-list-item-title>
                    </v-list-item>
                </v-list>
            </v-menu>
        </div>
    </v-card>
    
    <!-- 确认删除对话框 -->
    <v-dialog v-model="deleteDialog" max-width="300px">
        <v-card>
            <v-card-title class="text-subtitle-1">确认删除</v-card-title>
            <v-card-text>
                确定删除"{{ localData.title }}"吗？
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn variant="text" @click="deleteDialog = false">取消</v-btn>
                <v-btn color="error" variant="text" @click="confirmDelete">删除</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>

    <!-- 优化模态框条件渲染 -->
    <CardContentModal
        v-if="showModal"
        v-model="showModal"
        :card-data="localData"
        @confirm="handleConfirm"
    />
</template>

<script lang="ts">
import { defineComponent, defineAsyncComponent, computed } from 'vue';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { convertTimestampToDate } from '@/services/DateTimeService';

// 懒加载模态框组件
const CardContentModal = defineAsyncComponent(() =>
  import('@/components/Modals/CardContentModal.vue')
);

// 时间戳缓存对象
const dateCache = new Map<string, string | null>();

export default defineComponent({
    name: 'MListCard',
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
    emits: ['update', 'delete'],
    setup(props) {
        // 计算属性缓存日期转换结果
        const formattedDate = computed(() => {
            if (!props.data.ddl) return null;
            
            // 从缓存中获取日期
            if (dateCache.has(props.data.ddl)) {
                return dateCache.get(props.data.ddl);
            }
            
            // 计算并缓存结果
            const result = convertTimestampToDate(props.data.ddl);
            dateCache.set(props.data.ddl, result);
            return result;
        });
        
        return {
            formattedDate
        };
    },
    data() {
        return {
            showModal: false,
            deleteDialog: false,
            // 使用浅拷贝代替昂贵的JSON深拷贝
            localData: {
                ...this.data,
                finished: this.data.finished || false
            } as FEvent,
            _clickTimeout: null as NodeJS.Timeout | null
        };
    },
    methods: {
        // 添加防抖处理，避免快速点击导致的性能问题
        handleComplete(updatedData: FEvent) {
            if (this._clickTimeout) return;
            
            this._clickTimeout = setTimeout(() => {
                this.localData.finished = !this.localData.finished;
                this.localData = { ...updatedData };
                this.$emit('update', updatedData);
                this._clickTimeout = null;
            }, 100);
        },
        handleConfirm(updatedData: FEvent) {
            this.localData = { ...updatedData };
            this.$emit('update', updatedData);
        },
        handleEdit() {
            this.showModal = true;
        },
        handleDelete() {
            this.deleteDialog = true;
        },
        confirmDelete() {
            this.$emit('delete', this.localData);
            this.deleteDialog = false;
        }
    },
    // 添加组件卸载前的清理
    beforeUnmount() {
        if (this._clickTimeout) {
            clearTimeout(this._clickTimeout);
        }
    }
});
</script>

<style scoped>
/* 移动卡片基本样式 */
.m-list-card {
    border-radius: 12px;
    transition: all 0.2s ease;
}

.m-list-card:active {
    transform: scale(0.98);
}

/* 已完成状态样式 */
.completed-card {
    opacity: 0.8;
    background-color: var(--v-theme-surface) !important;
}

/* 优化复选框区域 */
.checkbox-container {
    min-width: 40px; /* 增大触摸区域 */
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* 任务信息容器 */
.task-info-container {
    flex: 1;
    min-height: 48px;
    padding: 4px 0;
    cursor: pointer;
}

/* 标签胶囊样式 */
.tag-pill {
    background-color: rgba(var(--v-theme-primary), 0.1);
    color: rgb(var(--v-theme-primary));
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    white-space: nowrap;
}

/* 优化触摸体验 */
:deep(.v-btn) {
    min-height: 36px;
    min-width: 36px;
}

/* 确保所有可点击元素有足够大的触摸区域 */
:deep(.v-checkbox) {
    margin: 0;
    padding: 0;
}

/* 优化性能 */
.m-list-card {
    contain: content;
    will-change: transform;
}
</style>