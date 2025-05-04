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
                            <br>
                            <div class="form-group">
                                <v-text-field clearable label="标题" id="title" v-model="formData.title" type="text"
                                    placeholder="输入标题" variant="outlined"></v-text-field>
                            </div>
                            <div class="form-group">
                                <v-text-field clearable label="标签" id="tag" v-model="tagInput" type="text"
                                    placeholder="如: #重要 #今日" variant="outlined"></v-text-field>
                            </div>
                            <div class="form-group">
                                <VDatePicker v-model="dateValue" mode="dateTime" is24hr :is-dark="isDarkMode()"
                                    :popover="{
                                        visibility: 'click',
                                        placement: 'bottom',
                                        isInteractive: true
                                    }" is-expanded :min-date="minDate" :max-date="maxDate">
                                    <template #default="{ inputEvents }">
                                        <v-text-field clearable label="日期" v-model="formData.ddl"
                                            :value="formData.ddl ? new Date(Number(formData.ddl)).toLocaleString() : ''"
                                            v-on="inputEvents" placeholder="选择日期和时间" class="date-input" readonly
                                            variant="outlined"></v-text-field>
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
import { defineComponent, ref, watch, onBeforeUnmount, nextTick, computed, onErrorCaptured } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { getEventContent, putEventContent } from '@/services/EventService';
import { DatePicker } from 'v-calendar';
import 'v-calendar/dist/style.css';

declare module 'vditor' {
    interface IVditor {
        destroy: () => void;
        getValue: () => string;
        setValue: (content: string) => void;
    }
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
        const isInitialized = ref(false);

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

        // 简化日期值的计算属性
        const dateValue = computed({
            get: () => formData.value.ddl ? new Date(Number(formData.value.ddl)) : null,
            set: (date: Date | null) => {
                formData.value.ddl = date ? date.getTime().toString() : ''
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
                const newContent = await getEventContent(props.cardData.id);
                content.value = newContent || '';

                // 如果编辑器已初始化，立即更新内容
                if (isInitialized.value && vditor.value) {
                    vditor.value.setValue(content.value);
                }
            } catch (error) {
                console.error('加载内容失败:', error);
                content.value = '';
            } finally {
                isLoading.value = false;
            }
        };

        const handleOpen = async () => {
            formData.value = { ...props.cardData };
            tagInput.value = processTags(props.cardData.tag || []);
            await loadContent();
            await initEditor();
        };

        const handleConfirm = async () => {
            try {
                if (vditor.value) {
                    content.value = vditor.value.getValue();
                    if (formData.value.id && content.value !== null) {
                        await putEventContent(formData.value.id, content.value);
                    }
                }

                const tagArray = Array.from(
                    new Set(
                        tagInput.value
                            .split(/\s+/)
                            .map(tag => tag.trim())
                            .filter(tag => tag.length > 0 && tag.length <= 20)
                            .map(tag => tag.startsWith('#') ? tag.slice(1) : tag)
                    )
                );

                formData.value.tag = tagArray;

                emit('confirm', formData.value);
                handleClose();
            } catch (error) {
                console.error('保存内容失败:', error);
            }
        };

        const handleClose = () => {
            emit('update:modelValue', false);
            setTimeout(() => {
                if (vditor.value) {
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

        // 定义日期范围常量
        const minDate = new Date(2000, 0, 1);
        const maxDate = new Date(2100, 11, 31);

        // 简化的日期选择完成处理
        const handleDateSelected = (date: Date | null) => {
            if (date && !isNaN(date.getTime())) {
                formData.value.ddl = date.getTime().toString();
            } else {
                formData.value.ddl = '';
            }
        };

        // 添加错误边界
        onErrorCaptured((err) => {
            if (String(err).includes('dayIndex')) {
                console.warn('日历组件数据异常，已拦截:', err);
                return false; // 阻止错误继续传播
            }
            return true;
        });

        return {
            formData,
            tagInput,
            isLoading,
            handleConfirm,
            handleClose,
            vditorRef: ref(null),
            dateValue,
            handleDateSelected,
            minDate,
            maxDate,
            isDarkMode
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
@import '@/styles/Modals/contentmodal.css';
</style>

<style>
@import '@/styles/vditor.css';
</style>