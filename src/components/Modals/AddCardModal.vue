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
                    <label for="priority">优先级</label>
                    <select id="priority" v-model="formData.priority">
                        <option value="Low">低</option>
                        <option value="Medium">中</option>
                        <option value="High">高</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="tags">标签</label>
                    <input id="tags" v-model="formData.tags" type="text" placeholder="用逗号分隔多个标签" />
                </div>
                <div class="form-group">
                    <label for="deadline">截止日期</label>
                    <input id="deadline" v-model="formData.deadline" type="datetime-local" />
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn-cancel" @click="handleClose">取消</button>
                    <button type="submit" class="btn-confirm" :disabled="eventStore.isLoading">确认</button>
                </div>
            </form>
        </div>
    </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue';
import { useEventStore } from '@/stores';
import { Priority } from 'src-tauri/bindings/Priority';

// Props
const props = defineProps({
    modelValue: {
        type: Boolean,
        required: true
    },
    listId: {
        type: String,
        required: true
    }
});

// Emits
const emit = defineEmits(['update:modelValue', 'confirm']);

// 使用eventStore
const eventStore = useEventStore();

// 表单数据
const formData = reactive({
    title: '',
    content: '',
    tags: '',
    priority: 'Medium' as Priority,
    deadline: ''
});

// 关闭模态框
const handleClose = () => {
    emit('update:modelValue', false);
};

// 提交表单
const handleSubmit = async () => {
    // 转换日期格式为时间戳
    const timestamp = formData.deadline
        ? new Date(formData.deadline).getTime().toString()
        : Date.now().toString();
    
    // 处理标签
    // const tagArray = formData.tags
    //     .split(',')
    //     .map(tag => tag.trim())
    //     .filter(Boolean);
    
    try {
        // 使用eventStore添加事件
        await eventStore.addEvent(
            props.listId,
            formData.title,
            formData.priority as Priority,
            timestamp
        );
        
        // 如果需要保存事件内容
        if (formData.content && eventStore.getEventById(props.listId)) {
            // 获取最新添加的事件，这里需要实现更详细的逻辑来确认是哪个事件
            const events = eventStore.getEventsByListId(props.listId);
            if (events.length > 0) {
                const latestEvent = events[events.length - 1];
                // 保存内容
                await eventStore.saveEventContent(latestEvent.id, formData.content);
            }
        }
        
        // 发出确认事件
        emit('confirm', { 
            success: true,
            message: '卡片创建成功' 
        });
        
        // 重置表单并关闭
        resetForm();
        handleClose();
    } catch (error) {
        console.error('创建卡片失败:', error);
        emit('confirm', { 
            success: false,
            message: '卡片创建失败',
            error 
        });
    }
};

// 重置表单
const resetForm = () => {
    formData.title = '';
    formData.content = '';
    formData.tags = '';
    formData.priority = 'Medium' as Priority;
    formData.deadline = '';
};
</script>

<style scoped>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.3); /* 从0.5降低到0.3，使遮罩更透明 */
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
textarea,
select {
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

button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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