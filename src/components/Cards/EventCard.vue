<template>
    <div class="card-base" @click.stop="handleCardClick">
        <div class="card-content-row">
            <input type="checkbox" :checked="localData.finished" @click.stop="handleComplete()" class="card-checkbox"/>
            <h3 class="card-title" :class="{ 'completed-task': localData.finished }">{{ localData.title }}</h3>
            <!-- 直接在模板中缓存时间结果，避免在渲染过程中重复调用方法 -->
            <span v-if="ddlTime" class="card-time"
                :style="{ color: localData.color || 'var(--md-sys-color-on-surface-variant)' }">
                {{ ddlTime }}
            </span>
        </div>
        <div v-if="localData.tag?.length" class="card-tags">
            <!-- 使用标签ID或索引作为key，提高列表渲染性能 -->
            <span v-for="(tag, i) in localData.tag" :key="tag || i" class="card-tag">{{ tag }}</span>
        </div>
    </div>

    <!-- 优化模态框的条件渲染 -->
    <CardContentModal
        v-if="showModal"
        v-model="showModal"
        :card-data="localData"
        @confirm="handleConfirm"
    />
</template>

<script lang="ts">
import { defineComponent, defineAsyncComponent, computed} from 'vue';
import { FEvent } from 'src-tauri/bindings/FEvent';
import { convertTimestampToTime } from '@/services/DateTimeService';

// 动态导入模态框组件，减少初始加载时间
const CardContentModal = defineAsyncComponent(() =>
  import('@/components/Modals/CardContentModal.vue')
);

// 时间戳缓存对象
const timeCache = new Map<string, string | null>();

export default defineComponent({
    name: 'EventCard',
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
        
        return {
            ddlTime
        };
    },
    data() {
        return {
            showModal: false,
            deleteDialog: false,
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
        // 转换时间戳为可读时间，使用缓存机制优化
        convertTimestampToTime(timestamp: string | undefined): string | null {
            if (!timestamp) return null;
            
            // 检查缓存中是否已有此时间戳的结果
            if (timeCache.has(timestamp)) {
                return timeCache.get(timestamp) || null;
            }
            
            // 计算结果并缓存
            const result = convertTimestampToTime(timestamp);
            timeCache.set(timestamp, result);
            return result;
        },
        handleCardClick() {
            this.showModal = true;
        },
        handleConfirm(updatedData: FEvent) {
            this.localData = { ...updatedData } as FEvent;
            this.$emit('update', updatedData);
        },
        // 添加简单的防抖机制避免快速点击造成的问题
        handleComplete() {
            if (this._clickTimeout) return;
            
            this._clickTimeout = setTimeout(() => {
                this.localData.finished = !this.localData.finished;
                this.$emit('toggleStatus', {
                    id: this.localData.id,
                    finished: this.localData.finished
                });
                this._clickTimeout = null;
            }, 100);
        },
        handleDelete() {
            this.deleteDialog = true;
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
@import '@/styles/Cards/card.css';

/* 添加性能优化相关的CSS */
.card-base {
    contain: content; /* 启用CSS包含以改进性能 */
    will-change: transform; /* 提示浏览器这个元素将会变化，优化动画性能 */
}

/* 提高标签渲染性能 */
.card-tags {
    contain: layout style; /* 改进布局性能 */
}

/* 优化标题元素渲染性能 */
.card-title {
    text-rendering: optimizeSpeed; /* 优化文本渲染速度 */
}
</style>