<template>
    <Teleport to="body">
        <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
            <div class="modal-container">
                <div class="modal-header">
                    <h3>卡片详情</h3>
                    <button @click="handleClose" aria-label="关闭弹窗">&times;</button>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label for="title">标题</label>
                        <input id="title" v-model="formData.title" type="text" placeholder="输入标题">
                    </div>
                    <div class="form-group">
                        <label for="content">内容</label>
                        <textarea id="content" v-model="formData.content" placeholder="输入内容" rows="4"></textarea>
                    </div>
                    <div class="form-group">
                        <label for="tags">标签</label>
                        <input id="tags" v-model="formData.tags" type="text" placeholder="用逗号分隔多个标签">
                    </div>
                    <div class="form-group">
                        <label for="date">日期</label>
                        <input id="date" v-model="formData.date" type="date">
                    </div>
                </div>
                <div class="modal-footer">
                    <button @click="handleClose">取消</button>
                    <button type="button" @click="handleConfirm" class="confirm-btn">保存</button>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<script>
import { Teleport } from 'vue' // 仅在Vue 3中需要

export default {
    name: 'CardContentModal',
    props: {
        modelValue: {
            type: Boolean,
            default: false
        },
        cardData: {
            type: Object,
            required: true
        }
    },
    emits: ['update:modelValue', 'confirm'],
    data() {
        return {
            formData: {
                title: '',
                content: '',
                tags: '',
                date: ''
            }
        }
    },
    watch: {
        modelValue: {
            immediate: true,
            handler(val) {
                if (val) {
                    this.initFormData()
                }
            }
        }
    },
    methods: {
        initFormData() {
            this.formData = {
                title: this.cardData.title || '',
                content: this.cardData.content || '',
                tags: Array.isArray(this.cardData.tags) ? this.cardData.tags.join(',') : '',
                date: this.cardData.date || ''
            }
        },
        handleConfirm() {
            const updatedData = {
                ...this.cardData,
                title: this.formData.title.trim(),
                content: this.formData.content.trim(),
                tags: this.formData.tags.split(',').map(tag => tag.trim()).filter(Boolean),
                date: this.formData.date
            }
            console.log('Updating card:', updatedData)
            this.$emit('confirm', updatedData)
            this.handleClose()
        },
        handleClose() {
            this.$emit('update:modelValue', false)
        }
    }
}
</script>

<style scoped>
/* 弹窗样式 */
.modal-mask {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw; /* 使用视口单位 */
    height: 100vh;
    background: rgba(0, 0, 0, 0.3); /* 改为半透明黑色 */
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 99999;
    transform: none !important; /* 避免创建新的堆叠上下文 */
    isolation: isolate; /* 创建独立的堆叠上下文 */
}

.modal-container {
    background: var(--md-sys-color-surface);
    padding: 20px;
    border-radius: 8px;
    width: min(90%, 400px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.33);
    position: relative; /* 确保容器也有相对定位 */
    z-index: 10002; /* 比mask更高一级 */
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.modal-header h3 {
    color: var(--md-sys-color-primary);
    margin: 0;
}

.modal-header button {
    background: none;
    border: none;
    font-size: 1.5em;
    cursor: pointer;
    color: var(--md-sys-color-on-surface);
}

.modal-body input {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--md-sys-color-outline);
    border-radius: 4px;
    margin-bottom: 1rem;
}

.modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
}

.modal-footer button {
    background: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
    color: var(--md-sys-color-on-surface);
}

.confirm-btn {
    background: var(--md-sys-color-primary);
    color: white;
    border: none;
}

.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 8px;
    color: var(--md-sys-color-on-surface);
    font-weight: 500;
}

.form-group input,
.form-group textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--md-sys-color-outline);
    border-radius: 4px;
    background: var(--md-sys-color-surface);
    color: var(--md-sys-color-on-surface);
}

.form-group textarea {
    resize: vertical;
    min-height: 100px;
}

.form-group input:focus,
.form-group textarea:focus {
    outline: none;
    border-color: var(--md-sys-color-primary);
}
</style>

/* 添加这个全局样式到您的App.vue或main.css中 */
#app {
    isolation: isolate; /* 创建新的堆叠上下文 */
}