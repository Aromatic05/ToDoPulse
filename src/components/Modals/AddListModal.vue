<template>
    <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
        <div class="modal-container">
            <div class="modal-header">
                <h3>新建列表</h3>
                <button @click="handleClose" aria-label="关闭弹窗">
                    &times;
                </button>
            </div>
            <div class="modal-body">
                <input ref="input" v-model="inputValue" type="text" placeholder="输入列表名称" @keyup.enter="handleConfirm"
                    aria-label="列表名称输入框">
            </div>
            <div class="modal-footer">
                <button @click="handleClose">取消</button>
                <button @click="handleConfirm" class="confirm-btn">确认</button>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'AddListModal',
    props: {
        modelValue: {
            type: Boolean,
            default: false
        }
    },
    data() {
        return {
            inputValue: ''
        }
    },
    watch: {
        modelValue(newVal) {
            if (newVal) {
                this.$nextTick(() => this.$refs.input.focus())
            }
        }
    },
    methods: {
        handleClose() {
            this.inputValue = ''
            this.$emit('update:modelValue', false)
        },
        handleConfirm() {
            if (this.inputValue.trim()) {
                this.$emit('confirm', this.inputValue.trim())
                this.handleClose()
            }
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
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
}

.modal-container {
    background: var(--md-sys-color-surface);
    padding: 20px;
    border-radius: 8px;
    width: min(90%, 400px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.33);
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
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
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
}

.confirm-btn {
    background: var(--md-sys-color-primary);
    color: white;
    border: none;
}
</style>