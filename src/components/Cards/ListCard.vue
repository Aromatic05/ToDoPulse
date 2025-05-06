<template>
    <div class="card-base">
        <div class="card-content-row">
            <!-- 状态列 - 添加checked绑定 -->
            <div class="card-column status-column">
                <input type="checkbox" class="card-checkbox" :checked="localData.finished" @change="handleComplete(localData)"/>
            </div>

            <!-- 任务列 -->
            <div class="card-column title-column">
                <h3 class="card-title" :class="{ 'completed-task': localData.finished }">
                    {{ localData.title }}
                </h3>
            </div>

            <!-- 优先级列 -->
            <div class="card-column priority-column">
                <div v-if="localData.tag?.length" class="card-tags">
                    <span v-for="(tag, i) in localData.tag" :key="tag || i" class="card-tag">{{ tag }}</span>
                </div>
            </div>

            <!-- 截止日期列 -->
            <div class="card-column date-column">
                <span v-if="localData.ddl" class="card-date"
                    :style="{ color: localData.color || 'var(--md-sys-color-on-surface-variant)' }">
                    {{ formattedDate }}
                </span>
            </div>

            <!-- 操作列 -->
            <div class="card-column actions-column">
                <div class="card-actions">
                    <v-icon size="small" class="action-icon" @click.stop="handleEdit" title="编辑">mdi-pencil</v-icon>
                    <v-icon size="small" class="action-icon" @click.stop="handleDelete" title="删除">mdi-delete</v-icon>
                </div>
            </div>
        </div>
    </div>
    
    <!-- 添加确认删除对话框 -->
    <v-dialog v-model="deleteDialog" max-width="400px">
        <v-card>
            <v-card-title class="text-h5">确认删除</v-card-title>
            <v-card-text>
                确定要删除"{{ localData.title }}"任务吗？此操作无法撤销。
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="grey darken-1" text @click="deleteDialog = false">取消</v-btn>
                <v-btn color="error" variant="elevated" @click="confirmDelete">删除</v-btn>
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
import { FEvent } from 'src-tauri/bindings/FEvent';
import { convertTimestampToDate, convertTimestampToTime } from '@/services/DateTimeService';

// 懒加载模态框组件
const CardContentModal = defineAsyncComponent(() =>
  import('@/components/Modals/CardContentModal.vue')
);

// 时间戳缓存对象
const dateCache = new Map<string, string | null>();
const timeCache = new Map<string, string | null>();

export default defineComponent({
    name: 'ListCard',
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
    emits: ['update', 'delete', 'toggleStatus'], // 声明自定义事件
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
            // 添加防抖定时器引用
            _clickTimeout: null as NodeJS.Timeout | null
        };
    },
    methods: {
        // 缓存时间戳转换结果
        convertTimestampToDate(timestamp: string | undefined): string | null {
            if (!timestamp) return null;
            
            // 检查缓存中是否已有此时间戳的结果
            if (dateCache.has(timestamp)) {
                return dateCache.get(timestamp) || null;
            }
            
            // 计算结果并缓存
            const result = convertTimestampToDate(timestamp);
            dateCache.set(timestamp, result);
            return result;
        },
        convertTimestampToTime(timestamp: string | undefined): string | null {
            if (!timestamp) return null;
            
            if (timeCache.has(timestamp)) {
                return timeCache.get(timestamp) || null;
            }
            
            const result = convertTimestampToTime(timestamp);
            timeCache.set(timestamp, result);
            return result;
        },
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
@import '@/styles/Cards/card.css';
@import '@/styles/Cards/listcard.css';

/* 添加性能优化相关的CSS */
.card-base {
    contain: content; /* 启用CSS包含以改进性能 */
    will-change: transform; /* 提示浏览器这个元素将会变化，优化动画性能 */
}

/* 提高列布局性能 */
.card-column {
    contain: layout style; /* 改进布局性能 */
}

/* 优化标题元素渲染性能 */
.card-title {
    text-rendering: optimizeSpeed; /* 优化文本渲染速度 */
}

/* 优化标签列表渲染 */
.card-tags {
    contain: layout style; /* 限制重绘范围 */
}

/* 优化操作图标 */
.card-actions {
    contain: layout style;
}
</style>