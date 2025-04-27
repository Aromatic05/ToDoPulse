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
                                <label for="tags">标签</label>
                                <input id="tags" v-model="formData.tags" type="text" placeholder="用逗号分隔多个标签">
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

<script>
import { Teleport } from 'vue'
import Vditor from 'vditor'
import 'vditor/dist/index.css'

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
            },
            vditor: null,
            vditorInitialized: false
        }
    },
    watch: {
        modelValue(val) {
            if (val) {
                // 先初始化表单数据
                this.initFormData();

                // 在下一个 tick 更新编辑器内容
                this.$nextTick(() => {
                    // 如果编辑器尚未初始化，则进行初始化
                    if (!this.vditorInitialized) {
                        this.initVditor();
                    } else if (this.vditor) {
                        // 编辑器已存在，只需更新内容
                        this.vditor.setValue(this.formData.content || '');
                    }
                });
            }
        }
    },
    mounted() {
        // 组件挂载时初始化编辑器，但仅在模态窗口显示时
        if (this.modelValue) {
            this.initVditor();
        }

        // 使用全局事件替代 MutationObserver
        document.addEventListener('theme-change', this.handleThemeChangeEvent);
    },
    beforeUnmount() {
        // 移除事件监听
        document.removeEventListener('theme-change', this.handleThemeChangeEvent);

        // 组件卸载前销毁编辑器
        this.destroyVditor();
    },
    methods: {
        // 添加一个方法检测当前是否为暗模式
        isDarkMode() {
            return document.body.classList.contains('dark') ||
                document.body.classList.toString().includes('-dark');
        },

        // 处理主题变化事件 - 新增方法
        handleThemeChangeEvent(event) {
            const { isDarkMode } = event.detail;
            if (this.vditor) {
                this.vditor.setTheme(isDarkMode ? 'dark' : 'classic');
            }
        },

        initFormData() {
            this.formData = {
                title: this.cardData.title || '',
                content: this.cardData.content || '',
                tags: Array.isArray(this.cardData.tags) ? this.cardData.tags.join(',') : '',
                date: this.cardData.date || ''
            };
        },

        initVditor() {
            // 如果已经存在编辑器实例，直接返回
            if (this.vditorInitialized && this.vditor) {
                this.vditor.setValue(this.formData.content || '');
                return;
            }

            // 创建新的 Vditor 实例
            this.vditor = new Vditor('vditor', {
                height: 500,
                width: '100%',
                theme: this.isDarkMode() ? 'dark' : 'classic', // 根据当前主题设置
                toolbarConfig: {
                    pin: true,
                },
                cache: {
                    enable: false,
                },
                placeholder: '请输入内容...',
                mode: 'wysiwyg',
                after: () => {
                    console.log('Vditor initialized');
                    this.vditorInitialized = true;
                    if (this.formData.content) {
                        this.vditor.setValue(this.formData.content);
                    }
                },
                input: (value) => {
                    this.formData.content = value;
                }
            });

            // 创建自定义样式以匹配应用主题
            this.createCustomVditorStyles();
        },

        // 创建与应用主题匹配的自定义样式
        createCustomVditorStyles() {
            const styleId = 'vditor-custom-style';
            let styleEl = document.getElementById(styleId);

            if (!styleEl) {
                styleEl = document.createElement('style');
                styleEl.id = styleId;
                document.head.appendChild(styleEl);
            }

            styleEl.textContent = `
                .dark .vditor,
                [class*="-dark"] .vditor {
                    --vditor-theme-primary-color: var(--md-sys-color-primary);
                    --vditor-bg-color: var(--md-sys-color-surface);
                    --vditor-code-bg-color: var(--md-sys-color-surface-variant);
                    --vditor-border-color: var(--md-sys-color-outline);
                }
                
                .dark .vditor-toolbar,
                [class*="-dark"] .vditor-toolbar {
                    background-color: var(--md-sys-color-surface-variant);
                    border-bottom: 1px solid var(--md-sys-color-outline);
                }
            `;
        },

        destroyVditor() {
            if (this.vditor) {
                this.vditor.destroy();
                this.vditor = null;
                this.vditorInitialized = false;
            }
        },

        handleConfirm() {
            // 确保从编辑器获取最新内容
            if (this.vditor) {
                this.formData.content = this.vditor.getValue();
            }

            const updatedData = {
                ...this.cardData,
                title: this.formData.title.trim(),
                content: this.formData.content.trim(),
                tags: this.formData.tags.split(',').map(tag => tag.trim()).filter(Boolean),
                date: this.formData.date
            };

            this.$emit('confirm', updatedData);
            this.handleClose();
        },

        handleClose() {
            this.$emit('update:modelValue', false);

            // 在模态窗口关闭时清理编辑器状态，确保下次重新初始化
            if (this.vditor) {
                // 保存编辑器内容到 formData 以避免内容丢失
                this.formData.content = this.vditor.getValue();

                // 销毁编辑器实例并重置标志
                this.vditor.destroy();
                this.vditor = null;
                this.vditorInitialized = false;
            }
        },
    }
}
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