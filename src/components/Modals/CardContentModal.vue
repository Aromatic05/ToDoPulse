<template>
    <Teleport to="body">
        <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
            <div class="modal-container">
                <div class="modal-header">
                    <!-- <button @click="handleClose" aria-label="关闭弹窗">&times;</button> -->
                </div>
                <div class="modal-body">
                    <!-- 改为左右两栏布局 -->
                    <div class="modal-layout">
                        <!-- 左侧表单字段 -->
                        <div class="form-section">
                            <h3>卡片详情</h3>
                            <div class="form-group">
                                <label for="title">标题</label>
                                <input id="title" v-model="formData.title" type="text" placeholder="输入标题">
                            </div>
                            <div class="form-group">
                                <label for="tag">标签</label>
                                <input id="tag" v-model="formData.tag" type="text" placeholder="用逗号分隔多个标签">
                            </div>
                            <div class="form-group">
                                <label for="date">日期</label>
                                <input id="date" v-model="formData.date" type="date">
                            </div>
                        </div>

                        <!-- 右侧编辑器 -->
                        <div class="editor-section">
                            <label for="content">内容</label>
                            <div id="vditor" ref="vditorRef"></div>
                        </div>
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

<script lang="ts">
import { defineComponent, ref, watch, onMounted, onBeforeUnmount } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { getEventContent } from '@/services/ListDataService';

export default defineComponent({
    name: 'CardContentModal',
    props: {
        modelValue: {
            type: Boolean,
            default: false
        },
        cardData: {
            type: Object as () => FEvent, // 使用 FEvent 类型
            required: true
        }
    },
    emits: ['update:modelValue', 'confirm'],
    setup(props, { emit }) {
        const vditor = ref<Vditor | null>(null);
        const vditorInitialized = ref(false);

        // 将 formData 定义为 ref<FEvent>
        const formData = ref<FEvent>({
            id: props.cardData.id || '',
            title: props.cardData.title || '',
            date: props.cardData.date || '',
            time: props.cardData.time || '',
            listid: props.cardData.listid || '',
            tag: props.cardData.tag || [],
            create: props.cardData.create || '',
            finished: props.cardData.finished || false,
            priority: props.cardData.priority || 'Low', // 假设默认优先级为 'Low'
            icon: props.cardData.icon || '',
            color: props.cardData.color || ''
        });

        const content = ref<string | null>(null);
        (async () => {
            content.value = await getEventContent(formData.value.id); // 获取内容
        })();

        const initVditor = () => {
            if (vditorInitialized.value && vditor.value) {
                vditor.value.setValue(content.value || ''); // 设置内容
                return;
            }

            vditor.value = new Vditor('vditor', {
                height: 500,
                width: '100%',
                theme: isDarkMode() ? 'dark' : 'classic',
                toolbarConfig: {
                    pin: true
                },
                cache: {
                    enable: false
                },
                placeholder: '请输入内容...',
                mode: 'wysiwyg',
                after: () => {
                    vditorInitialized.value = true;
                    if (content.value) {
                        vditor.value?.setValue(content.value);
                    }
                },
                input: (value: string) => {
                    content.value = value;
                }
            });
        };

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark') || document.body.classList.toString().includes('-dark');
        };

        const handleConfirm = () => {
            if (vditor.value) {
                content.value = vditor.value.getValue();
            }

            emit('confirm', formData.value); // 直接传递 FEvent 类型的 formData
            handleClose();
        };

        const handleClose = () => {
            emit('update:modelValue', false);

            if (vditor.value) {
                vditor.value.destroy();
                vditor.value = null;
                vditorInitialized.value = false;
            }
        };

        watch(
            () => props.modelValue,
            (val) => {
                if (val) {
                    formData.value = {
                        ...props.cardData // 使用 cardData 更新 formData
                    };

                    if (!vditorInitialized.value) {
                        initVditor();
                    } else if (vditor.value) {
                        vditor.value.setValue(content.value || '');
                    }
                }
            }
        );

        onMounted(() => {
            if (props.modelValue) {
                initVditor();
            }

            document.addEventListener('theme-change', handleThemeChangeEvent);
        });

        onBeforeUnmount(() => {
            document.removeEventListener('theme-change', handleThemeChangeEvent);

            if (vditor.value) {
                vditor.value.destroy();
                vditor.value = null;
            }
        });

        const handleThemeChangeEvent = (event: Event) => {
            const isDarkMode = (event as CustomEvent).detail.isDarkMode;
            if (vditor.value) {
                vditor.value.setTheme(isDarkMode ? 'dark' : 'classic');
            }
        };

        return {
            formData,
            handleConfirm,
            handleClose,
            initVditor,
            vditor
        };
    }
});
</script>

<style scoped>
/* 弹窗样式 */
.modal-mask {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    /* 使用视口单位 */
    height: 100vh;
    background: rgba(0, 0, 0, 0.3);
    /* 改为半透明黑色 */
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 99999;
    transform: none !important;
    /* 避免创建新的堆叠上下文 */
    isolation: isolate;
    /* 创建独立的堆叠上下文 */
}

.modal-container {
    background: var(--md-sys-color-surface);
    padding: 20px;
    border-radius: 8px;
    width: min(95%, 1200px);
    /* 增加窗口最大宽度到 1200px */
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.33);
    position: relative;
    /* 确保容器也有相对定位 */
    z-index: 10002;
    /* 比mask更高一级 */
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.form-section h3 {
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

.editor-section label {
    color: var(--md-sys-color-on-surface);
    font-size: 16px;
}

.vditor {
    border: 1px solid var(--md-sys-color-outline);
    border-radius: 4px;
    margin-bottom: 1rem;
    flex-grow: 1;
    /* 让编辑器填充可用空间 */
    min-height: 530px;
    /* 增加编辑器最小高度 */
}

.modal-layout {
    display: flex;
    flex-direction: row;
    gap: 20px;
    margin-bottom: 1rem;
    min-height: 500px;
    /* 增加最小高度 */
}

.form-section {
    flex: 1;
    min-width: 250px;
    max-width: 350px;
    /* 限制左侧宽度 */
}

.editor-section {
    flex: 3;
    /* 增加编辑器区域的比例 */
    display: flex;
    flex-direction: column;
    min-height: 500px;
    /* 增加编辑器区域最小高度 */
}

/* 移动端响应式布局 */
@media (max-width: 768px) {
    .modal-layout {
        flex-direction: column;
    }

    .form-section {
        max-width: 100%;
    }

    .editor-section {
        min-height: 400px;
    }
}
</style>

<style>
/* 全局样式，确保 Vditor 显示正常 */
.vditor {
    --vh: 1vh;
    /* 修复移动端高度问题 */
    border: 1px solid var(--md-sys-color-outline) !important;
    border-radius: 4px !important;
}

/* 全局样式以优化 Vditor 在不同主题下的外观 */
:root .vditor-toolbar {
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
}

:root .vditor {
    border: 1px solid var(--md-sys-color-outline-variant);
}

.dark .vditor-toolbar,
[class*="-dark"] .vditor-toolbar {
    background-color: var(--md-sys-color-surface-variant);
    border-bottom: 1px solid var(--md-sys-color-outline);
}

.dark .vditor-reset,
[class*="-dark"] .vditor-reset {
    color: var(--md-sys-color-on-surface);
}

.dark .vditor,
[class*="-dark"] .vditor {
    border-color: var(--md-sys-color-outline);
}

.dark .vditor-ir,
[class*="-dark"] .vditor-ir,
.dark .vditor-wysiwyg,
[class*="-dark"] .vditor-wysiwyg,
.dark .vditor-sv,
[class*="-dark"] .vditor-sv {
    background-color: var(--md-sys-color-surface);
}
</style>

/* 添加这个全局样式到您的App.vue或main.css中 */
#app {
isolation: isolate; /* 创建新的堆叠上下文 */
}