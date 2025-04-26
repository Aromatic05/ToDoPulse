<template>
    <div class="card-base" @click.stop="handleCardClick">
        <div class="card-content-row">
            <input 
                type="checkbox" 
                :checked="localData.isCompleted" 
                @click.stop="toggleComplete" 
                class="card-checkbox"
            />
            <h3 class="card-title">{{ localData.title }}</h3>
            <span v-if="localData.time" class="card-"
                :style="{ color: localData.dateColor || 'var(--md-sys-color-on-surface-variant)' }">
                {{ localData.date }}
            </span>
        </div>
        <div v-if="localData.tags?.length" class="card-tags">
            <span v-for="(tag, i) in localData.tags" :key="i" class="card-tag">{{ tag }}</span>
        </div>
    </div>

    <CardContentModal v-model="showModal" :card-data="localData" @confirm="handleConfirm" />
</template>

<script>
import CardContentModal from '@/components/Modals/CardContentModal.vue'

export default {
    name: 'EventCard',
    components: {
        CardContentModal
    },
    emits: ['update'],
    props: {
        data: {
            type: Object,
            required: true,
            validator: (value) => value?.id && value?.title && value?.content
        }
    },
    data() {
        return {
            showModal: false,
            localData: { 
                ...this.data,
                isCompleted: this.data.isCompleted || false // 确保有完成状态属性
            }
        }
    },
    watch: {
        data: {
            handler(newVal) {
                console.log('Card data updated:', newVal)
                this.localData = { 
                    ...newVal,
                    isCompleted: newVal.isCompleted || false // 确保有完成状态属性
                }
            },
            deep: true,  // 深度监听对象变化
            immediate: true  // 组件创建时立即执行
        }
    },
    methods: {
        handleCardClick() {
            this.showModal = true
        },
        handleConfirm(updatedData) {
            console.log('EventCard updating:', updatedData)
            this.localData = { ...updatedData }  // 更新本地数据
            this.$emit('update', updatedData)
        },
        toggleComplete(event) {
            // 更新完成状态
            const updatedData = { 
                ...this.localData, 
                isCompleted: !this.localData.isCompleted 
            }
            this.localData = updatedData
            this.$emit('update', updatedData)
        }
    }
}
</script>

<style scoped>
@import '@/styles/card.css';

.card-content-row {
    display: flex;
    align-items: center;
    gap: 12px;
}

/* 圆形复选框样式 */
.card-checkbox {
    cursor: pointer;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    appearance: none; /* 移除默认样式 */
    -webkit-appearance: none;
    -moz-appearance: none;
    background-color: transparent;
    border: 2px solid var(--md-sys-color-outline);
    border-radius: 50%; /* 圆形边框 */
    position: relative;
    margin: 0;
    transition: all 0.3s;
}

/* 选中状态的复选框 */
.card-checkbox:checked {
    background-color: var(--md-sys-color-primary);
    border-color: var(--md-sys-color-primary);
}

/* 选中状态的复选框中心标记 */
.card-checkbox:checked::after {
    content: '';
    position: absolute;
    width: 10px;
    height: 10px;
    background-color: white;
    border-radius: 50%;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
}

/* 悬停状态 */
.card-checkbox:hover {
    border-color: var(--md-sys-color-primary);
}

.card-title {
    margin: 0;
    flex-grow: 1;
}

.card-date {
    white-space: nowrap;
    margin-left: auto;
}

/* 完成状态的卡片样式 */
.card-checkbox:checked + .card-title {
    text-decoration: line-through;
    opacity: 0.7;
}
</style>