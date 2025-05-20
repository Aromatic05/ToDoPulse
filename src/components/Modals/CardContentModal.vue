<template>
    <Teleport to="body">
        <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
            <!-- 标签添加弹窗 -->
            <AddTagModal v-model="showAddTagModal" @created="handleTagCreated" />
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
                                    placeholder="输入标题" variant="outlined" density="compact"></v-text-field>
                            </div>
                            <div class="form-group">
                                <v-autocomplete
                                    v-model="selectedTags"
                                    :items="allTags"
                                    item-title="name"
                                    item-value="name"
                                    label="标签"
                                    placeholder="选择或输入标签"
                                    variant="outlined"
                                    density="compact"
                                    chips
                                    multiple
                                    closable-chips
                                    :menu-props="{ maxHeight: 400 }"
                                    :append-inner-icon="'mdi-plus'"
                                    @click:append-inner="openAddTagModal"
                                >
                                    <template v-slot:chip="{ props, item }">
                                        <v-chip
                                            v-bind="props"
                                            :color="item.raw && item.raw.color ? getTagColor(item.raw) : 'default'"
                                        >
                                            {{ item.title }}
                                        </v-chip>
                                    </template>
                                    <template v-slot:item="{ props, item }">
                                        <v-list-item
                                            v-bind="props"
                                            :prepend-icon="'mdi-tag'"
                                            :subtitle="'点击选择'"
                                            :color="item.raw && item.raw.color ? getTagColor(item.raw) : 'default'"
                                        >
                                            <v-list-item-title
                                                :style="item.raw && item.raw.color ?
                                                    { color: getTagColor(item.raw) } : {}"
                                            >
                                                {{ item.raw && item.raw.name ? item.raw.name : item.title }}
                                            </v-list-item-title>
                                        </v-list-item>
                                    </template>
                                </v-autocomplete>
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
                                            variant="outlined" density="compact"></v-text-field>
                                    </template>
                                </VDatePicker>
                            </div>

                            <div class="form-group">
                                <v-select v-model="formData.priority" :items="['Low', 'Medium', 'High', 'Undefined']"
                                    label="优先级" variant="outlined" density="compact" class="mb-4"></v-select>
                            </div>

                            <!-- 将图标输入改为图标选择 -->
                            <div class="form-group">
                                <p class="text-subtitle-1 mb-2">图标选择</p>
                                <v-chip-group v-model="selectedIconIndex" column>
                                    <v-chip v-for="(icon, index) in availableIcons" :key="index" filter :value="index"
                                        :selected="formData.icon === icon">
                                        <v-icon :icon="icon"></v-icon>
                                    </v-chip>
                                </v-chip-group>
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
import { defineComponent, ref, watch, onBeforeUnmount, nextTick, computed, onErrorCaptured, onMounted } from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { useEventStore, useTagStore } from '@/stores';
import { DatePicker } from 'v-calendar';
import 'v-calendar/dist/style.css';
import type { Tag } from '@/services/TagService';
import AddTagModal from './AddTagModal.vue';

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
        VDatePicker: DatePicker,
        AddTagModal
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
        // 使用store
        const eventStore = useEventStore();
        const tagStore = useTagStore();

        const vditor = ref<Vditor | null>(null);
        const content = ref<string>('');
        const isLoading = computed(() => eventStore.isLoading);
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

        const allTags = ref<Tag[]>([]);
        const selectedTags = ref<string[]>(props.cardData.tag || []);
        const showAddTagModal = ref(false);

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

        // 改进内容加载函数 - 使用eventStore
        const loadContent = async () => {
            if (!props.cardData.id) return;

            try {
                const newContent = await eventStore.getEventContent(props.cardData.id);
                content.value = newContent || '';

                // 如果编辑器已初始化，立即更新内容
                if (isInitialized.value && vditor.value) {
                    vditor.value.setValue(content.value);
                }
            } catch (error) {
                console.error('加载内容失败:', error);
                content.value = '';
            }
        };

        // 加载所有标签
        const loadTags = async () => {
            try {
                allTags.value = await tagStore.fetchTags();
            } catch (error) {
                console.error('加载标签失败:', error);
            }
        };

        // 获取标签颜色
        const getTagColor = (tag: Tag | undefined): string => {
            if (!tag || !tag.color) return 'default';
            
            // 处理枚举值或字符串
            if (typeof tag.color === 'string') {
                return tag.color.toLowerCase();
            } else {
                // 处理枚举类型 (TagColor)
                return String(tag.color).toLowerCase();
            }
        };

        // 打开添加标签弹窗
        const openAddTagModal = () => {
            showAddTagModal.value = true;
        };

        // 处理标签创建成功
        const handleTagCreated = async (tagName: string) => {
            await loadTags(); // 重新加载标签列表
            // 将新标签添加到已选择的标签中
            if (!selectedTags.value.includes(tagName)) {
                selectedTags.value = [...selectedTags.value, tagName];
            }
        };

        const handleOpen = async () => {
            formData.value = { ...props.cardData };
            selectedTags.value = props.cardData.tag || [];
            await loadTags();
            await loadContent();
            await initEditor();
        };

        const handleConfirm = async () => {
            try {
                if (vditor.value) {
                    content.value = vditor.value.getValue();
                    if (formData.value.id && content.value !== null) {
                        // 使用eventStore保存内容
                        await eventStore.saveEventContent(formData.value.id, content.value);
                    }
                }

                formData.value.tag = selectedTags.value;

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
                    selectedTags.value = newVal.tag || [];
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
            if (date && !Number.isNaN(date.getTime())) {
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

        const availableIcons = [
            'mdi-home',
            'mdi-account',
            'mdi-briefcase',
            'mdi-shopping',
            'mdi-star',
        ];

        // 同步图标选择和formData.icon
        const selectedIconIndex = computed({
            get: () => {
                const index = availableIcons.findIndex(icon => icon === formData.value.icon);
                return index !== -1 ? index : -1;
            },
            set: (index: number) => {
                if (index >= 0 && index < availableIcons.length) {
                    formData.value.icon = availableIcons[index];
                }
            }
        });

        // 初始加载标签
        onMounted(loadTags);

        return {
            formData,
            selectedTags,
            allTags,
            isLoading,
            handleConfirm,
            handleClose,
            openAddTagModal,
            getTagColor,
            vditorRef: ref(null),
            dateValue,
            handleDateSelected,
            minDate,
            maxDate,
            isDarkMode,
            availableIcons,
            selectedIconIndex,
            showAddTagModal,
            handleTagCreated,
        };
    }
});

</script>


<style scoped>
@import '@/styles/Modals/contentmodal.css';
</style>

<style>
@import '@/styles/vditor.css';

.v-overlay--active {
    z-index: 9999999 !important; /* 非常高的z-index值 */
}

.priority-select-menu {
    z-index: 9999999 !important; /* 非常高的z-index值 */
    position: fixed !important;
}
</style>