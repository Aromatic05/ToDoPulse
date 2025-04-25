<template>
    <div v-if="modelValue" class="modal-overlay" @click.self="handleClose">
        <div class="modal-content">
            <h2>新建卡片</h2>
            <form @submit.prevent="handleSubmit">
                <div class="form-group">
                    <label for="title">标题</label>
                    <input id="title" v-model="formData.title" type="text" required />
                </div>
                <div class="form-group">
                    <label for="content">内容</label>
                    <textarea id="content" v-model="formData.content" required></textarea>
                </div>
                <div class="form-group">
                    <label for="tags">标签</label>
                    <input id="tags" v-model="formData.tags" type="text" placeholder="用逗号分隔多个标签" />
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn-cancel" @click="handleClose">取消</button>
                    <button type="submit" class="btn-confirm">确认</button>
                </div>
            </form>
        </div>
    </div>
</template>

<script>
export default {
    name: 'AddCardModal',
    props: {
        modelValue: {
            type: Boolean,
            required: true
        }
    },
    data() {
        return {
            formData: {
                title: '',
                content: '',
                tags: ''
            }
        }
    },
    methods: {
        handleClose() {
            this.$emit('update:modelValue', false)
        },
        handleSubmit() {
            const cardData = {
                title: this.formData.title,
                content: this.formData.content,
                tags: this.formData.tags.split(',').map(tag => tag.trim()).filter(Boolean),
                date: new Date().toLocaleDateString()
            }
            this.$emit('confirm', cardData)
            this.resetForm()
            this.handleClose()
        },
        resetForm() {
            this.formData = {
                title: '',
                content: '',
                tags: ''
            }
        }
    }
}
</script>

<style scoped>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
}

.modal-content {
    background: var(--md-sys-color-surface);
    padding: 24px;
    border-radius: 16px;
    width: 90%;
    max-width: 500px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.form-group {
    margin-bottom: 16px;
}

label {
    display: block;
    margin-bottom: 8px;
    color: var(--md-sys-color-on-surface);
}

input,
textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--md-sys-color-outline);
    border-radius: 8px;
    background: var(--md-sys-color-surface);
    color: var(--md-sys-color-on-surface);
}

textarea {
    min-height: 100px;
    resize: vertical;
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 24px;
}

button {
    padding: 8px 16px;
    border-radius: 8px;
    border: none;
    cursor: pointer;
}

.btn-confirm {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
}

.btn-cancel {
    background: var(--md-sys-color-surface-variant);
    color: var(--md-sys-color-on-surface-variant);
}
</style>