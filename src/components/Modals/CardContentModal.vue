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
                                <input id="tag" v-model="tagInput" type="text" placeholder="如: #重要 #今日">
                            </div>
                            <div class="form-group">
                                <label for="date">日期</label>
                                <VDatePicker v-model="dateValue" mode="dateTime"
                                    @update:model-value="handleDateSelected"
                                    :popover="{ visibility: 'click' }" :attributes="attributes"
                                    hide-time-header is-expanded
                                    :min-date="new Date(2000, 0, 1)"
                                    :max-date="new Date(2100, 11, 31)">
                                    <template #default="slotProps">
                                        <!-- @ts-ignore -->
                                        <input id="date" type="text" :value="formattedDate" v-on="slotProps.inputEvents"
                                            placeholder="选择日期和时间" class="date-input">
                                    </template>
                                </VDatePicker>
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
import { defineComponent, ref, watch, onBeforeUnmount, nextTick, computed } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { getEventContent, putEventContent } from '@/services/EventService';
import { DatePicker } from 'v-calendar';

// 为DatePicker的slot上下文添加类型声明
interface DatePickerContext {
  inputValue: string | { start: string; end: string };
  inputEvents: object;
  updateValue: (value: any) => void;
}

export default defineComponent({
    name: 'CardContentModal',
    components: {
        VDatePicker: DatePicker
    },
    props: {
        modelValue: {
            type: Boolean,
            default: false
        },
        cardData: {
            type: Object as () => FEvent,
            required: true
        }
    },
    emits: ['update:modelValue', 'confirm'],
    setup(props, { emit }) {

        const vditor = ref<Vditor | null>(null);
        const content = ref<string>('');
        const isLoading = ref(false);
        const isInitialized = ref(false); // 添加初始化状态标记

        const formData = ref<FEvent>({
            id: props.cardData.id || '',
            title: props.cardData.title || '',
            ddl: props.cardData.ddl || '',
            listid: props.cardData.listid || '',
            tag: props.cardData.tag || [],
            create: props.cardData.create || '',
            finished: props.cardData.finished || false,
            priority: props.cardData.priority || 'Low',
            icon: props.cardData.icon || '',
            color: props.cardData.color || ''
        });

        const tagInput = ref(
            processTags(props.cardData.tag || [])
        );

        const dateDisplay = computed({
            get: () => {
                // 如果ddl是时间戳字符串，转换为YYYY-MM-DD格式
                if (formData.value.ddl) {
                    try {
                        const date = new Date(Number(formData.value.ddl));
                        if (!isNaN(date.getTime())) {
                            // 格式化为YYYY-MM-DD
                            return date.toISOString().split('T')[0];
                        }
                    } catch (e) {
                        console.error('日期转换错误:', e);
                    }
                }
                return ''; // 如果没有日期或转换失败则返回空
            },
            set: (dateString) => {
                if (dateString) {
                    // 将YYYY-MM-DD转换为时间戳字符串
                    const date = new Date(dateString);
                    formData.value.ddl = String(date.getTime());
                    console.log('设置时间戳:', formData.value.ddl);
                } else {
                    formData.value.ddl = '';
                }
            }
        });

        // 日期值的响应式引用
        const dateValue = computed({
            get: () => {
                if (formData.value.ddl) {
                    try {
                        const timestamp = Number(formData.value.ddl);
                        if (!isNaN(timestamp)) {
                            const date = new Date(timestamp);
                            // 额外检查确保返回有效日期
                            return !isNaN(date.getTime()) ? date : null;
                        }
                    } catch (e) {
                        console.error('日期转换错误:', e);
                    }
                }
                return null;
            },
            set: (date: Date | null) => {
                if (date instanceof Date && !isNaN(date.getTime())) {
                    formData.value.ddl = String(date.getTime());
                } else {
                    formData.value.ddl = '';
                }
            }
        });

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark') || document.body.classList.toString().includes('-dark');
        };

        // 新版初始化编辑器函数
        const initEditor = async () => {
            await nextTick();

            // 销毁旧实例
            if (vditor.value) {
                vditor.value.destroy();
                vditor.value = null;
                isInitialized.value = false;
            }

            console.log('正在初始化 Vditor 编辑器...');

            // 创建新实例
            vditor.value = new Vditor('vditor', {
                height: 500,
                width: '100%',
                theme: isDarkMode() ? 'dark' : 'classic',
                toolbarConfig: { pin: true },
                cache: { enable: false },
                placeholder: '请输入内容...',
                mode: 'wysiwyg',
                after: () => {
                    console.log('Vditor 初始化完成');
                    isInitialized.value = true;
                    // 设置初始内容
                    if (content.value) {
                        console.log('设置编辑器内容');
                        vditor.value?.setValue(content.value);
                    }
                },
                input: (value: string) => {
                    content.value = value;
                }
            });
        };

        // 改进内容加载函数
        const loadContent = async () => {
            if (!props.cardData.id) return;

            isLoading.value = true;
            try {
                console.log('加载内容, ID:', props.cardData.id);
                const newContent = await getEventContent(props.cardData.id);
                content.value = newContent || '';
                console.log('内容已加载:', content.value ? '有内容' : '无内容');

                // 如果编辑器已初始化，立即更新内容
                if (isInitialized.value && vditor.value) {
                    console.log('更新已初始化的编辑器内容');
                    vditor.value.setValue(content.value);
                }
            } catch (error) {
                console.error('加载内容失败:', error);
                content.value = '';
            } finally {
                isLoading.value = false;
            }
        };

        // 新增处理模态框打开的函数
        const handleOpen = async () => {
            console.log('模态框打开');

            // 重置表单数据
            formData.value = { ...props.cardData };

            // 更新标签
            tagInput.value = processTags(props.cardData.tag || []);

            // 先加载内容
            await loadContent();

            // 然后初始化编辑器
            await initEditor();
        };

        const handleConfirm = async () => {
            try {
                // 获取编辑器内容
                if (vditor.value) {
                    content.value = vditor.value.getValue();

                    // 保存编辑器内容到后端
                    if (formData.value.id && content.value !== null) {
                        await putEventContent(formData.value.id, content.value);
                    }
                }

                // 解析标签字符串为数组
                const tagArray = tagInput.value
                    .split(/\s+/)
                    .filter(tag => tag.trim() !== '')
                    .map(tag => tag.startsWith('#') ? tag.substring(1) : tag);

                // 更新 formData 中的 tag 数组
                formData.value.tag = tagArray;

                emit('confirm', formData.value);
                handleClose();
            } catch (error) {
                console.error('保存内容失败:', error);
            }
        };

        const handleClose = () => {
            // 先发出事件，再销毁编辑器
            emit('update:modelValue', false);

            // 确保编辑器被销毁
            setTimeout(() => {
                if (vditor.value) {
                    console.log('销毁编辑器');
                    vditor.value.destroy();
                    vditor.value = null;
                    isInitialized.value = false;
                }
            }, 10);
        };

        // 简化 watch 逻辑
        watch(
            () => props.modelValue,
            async (val) => {
                if (val) {
                    await handleOpen();
                }
            },
            { immediate: true }
        );

        watch(
            () => props.cardData,
            (newVal) => {
                if (props.modelValue) {
                    formData.value = { ...newVal };
                    tagInput.value = processTags(newVal.tag || []);
                }
            },
            { deep: true }
        );

        // 移除不必要的 onMounted 逻辑，依赖 watch
        onBeforeUnmount(() => {
            if (vditor.value) {
                vditor.value.destroy();
                vditor.value = null;
            }
        });

        // 格式化日期显示
        const formattedDate = computed(() => {
            if (!formData.value.ddl) return '';
            try {
                const date = new Date(Number(formData.value.ddl));
                if (isNaN(date.getTime())) return '';

                // 格式化为 YYYY-MM-DD HH:MM
                const year = date.getFullYear();
                const month = String(date.getMonth() + 1).padStart(2, '0');
                const day = String(date.getDate()).padStart(2, '0');
                const hours = String(date.getHours()).padStart(2, '0');
                const minutes = String(date.getMinutes()).padStart(2, '0');

                return `${year}-${month}-${day} ${hours}:${minutes}`;
            } catch (e) {
                console.error('日期格式化错误:', e);
                return '';
            }
        });

        // 处理日期选择完成 - 保留函数但简化逻辑
        const handleDateSelected = (date: Date | null) => {
            // 日期选择完成后的处理逻辑
        };

        // 定义日期选择器属性
        const attributes = [
            {
                key: 'today',
                highlight: {
                    color: 'var(--md-sys-color-primary)',
                    fillMode: 'light',
                },
                dates: new Date()
            },
        ];

        return {
            formData,
            tagInput,
            isLoading,
            handleConfirm,
            handleClose,
            vditorRef: ref(null),
            dateValue, // 返回新的日期值计算属性
            dateDisplay, // 添加到返回值
            formattedDate,
            handleDateSelected,
            attributes
        };
    }
});

// 辅助函数
function processTags(tags: string[]): string {
    return tags
        .filter(tag => tag && tag.trim() !== '')
        .map(tag => tag.startsWith('#') ? tag : `#${tag}`)
        .join(' ');
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
@import '@/styles/vditor.css';
</style>