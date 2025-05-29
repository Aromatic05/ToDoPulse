<template>
    <Teleport to="body">
        <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
            <AddTagModal v-model="showAddTagModal" @created="handleTagCreated" />
            <div class="modal-container">
                <div class="modal-header">
                </div>
                <div class="modal-body">
                    <div class="modal-layout">
                        <div class="form-section">
                            <h3>卡片详情</h3>
                            <br>
                            <div class="form-group">
                                <v-text-field clearable label="标题" id="title" v-model="formData.title" type="text"
                                    placeholder="输入标题" variant="outlined" density="compact"></v-text-field>
                            </div>
                            <div class="form-group">
                                <v-autocomplete v-model="selectedTags" :items="allTags" item-title="name"
                                    item-value="name" label="标签" placeholder="选择或输入标签" variant="outlined"
                                    density="compact" chips multiple closable-chips :menu-props="{ maxHeight: 400 }"
                                    :append-inner-icon="'mdi-plus'" @click:append-inner="openAddTagModal">
                                    <template v-slot:chip="{ props, item }">
                                        <v-chip v-bind="props"
                                            :color="item.raw && item.raw.color ? getTagColor(item.raw) : 'default'">
                                            {{ item.title }}
                                        </v-chip>
                                    </template>
                                    <template v-slot:item="{ props, item }">
                                        <v-list-item v-bind="props" :prepend-icon="'mdi-tag'" :subtitle="'点击选择'"
                                            :color="item.raw && item.raw.color ? getTagColor(item.raw) : 'default'">
                                            <v-list-item-title :style="item.raw && item.raw.color ?
                                                { color: getTagColor(item.raw) } : {}">
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
                                        <v-text-field clearable label="日期"
                                            :model-value="formData.ddl ? new Date(Number(formData.ddl)).toLocaleString() : ''"
                                            v-on="inputEvents" placeholder="选择日期和时间" class="date-input" readonly
                                            variant="outlined" density="compact"></v-text-field>
                                    </template>
                                </VDatePicker>
                            </div>

                            <div class="form-group">
                                <v-select v-model="formData.priority" :items="['Low', 'Medium', 'High', 'Undefined']"
                                    label="优先级" variant="outlined" density="compact" class="mb-4"></v-select>
                            </div>

                            <div class="form-group">
                                <label class="icon-label">图标选择</label>
                                <div class="icon-selection-container">
                                    <div v-for="(icon, index) in availableIcons" :key="index" class="icon-option"
                                        :class="{ 'icon-selected': formData.icon === icon }"
                                        @click="selectedIconIndex = index">
                                        <v-icon :icon="icon"></v-icon>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="editor-section">
                            <div class="editor-header">
                                <div class="header-left">
                                    <label for="content">内容</label>
                                    <div class="ai-actions" v-if="localAigcEnabled">
                                        <v-tooltip text="AI智能总结">
                                            <template v-slot:activator="{ props }">
                                                <v-btn
                                                    v-bind="props"
                                                    icon="mdi-lightning-bolt"
                                                    variant="text"
                                                    color="primary"
                                                    :loading="isAiLoading"
                                                    @click="handleAiSummary"
                                                    :disabled="!formData.id || !editorContent"
                                                ></v-btn>
                                            </template>
                                        </v-tooltip>
                                        <v-tooltip text="内容改进建议">
                                            <template v-slot:activator="{ props }">
                                                <v-btn
                                                    v-bind="props"
                                                    icon="mdi-lightbulb"
                                                    variant="text"
                                                    color="warning"
                                                    :loading="isAiLoading"
                                                    @click="handleAiImprovement"
                                                    :disabled="!formData.id || !editorContent"
                                                ></v-btn>
                                            </template>
                                        </v-tooltip>
                                    </div>
                                </div>
                            </div>
                            <VditorEditor v-if="formData.id" :event-id="formData.id" :card-title="formData.title"
                                v-model="editorContent" editor-id="card-content-editor"
                                @initialized="onEditorInitialized" />
                            <div v-else>
                                <p>请先保存卡片以启用内容编辑器和文件上传功能。</p>
                            </div>
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
import { defineComponent, ref, watch, nextTick, computed, onErrorCaptured, onMounted } from 'vue';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { useEventStore, useTagStore } from '@/stores';
import { DatePicker } from 'v-calendar';
import 'v-calendar/dist/style.css';
import type { Tag } from '@/services/TagService';
import { SettingService } from '@/services/SettingService';
import AigcService from '@/services/AigcService';
import AddTagModal from '@/components/Modals/AddTagModal.vue';
import VditorEditor from '@/components/VditorEditor.vue';

// 不再需要 IVditor 接口定义，因为它在 VditorEditor.vue 中

export default defineComponent({
    name: 'CardContentModal',
    components: {
        VDatePicker: DatePicker,
        AddTagModal,
        VditorEditor // 注册新的编辑器组件
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
        const eventStore = useEventStore();
        const tagStore = useTagStore();
        const isAiLoading = ref(false);
        const localAigcEnabled = ref(false);

        // 获取AI设置
        onMounted(() => {
            try {
                const settings = SettingService.getAigcSettings();
                if (settings) {
                    localAigcEnabled.value = settings.switch ?? false;
                }
            } catch (error) {
                console.error('加载AI设置失败:', error);
            }
        });

        // AI内容总结
        // AI内容总结
        const handleAiSummary = async () => {
            if (!formData.value.id || !editorContent.value) return;
            
            isAiLoading.value = true;
            try {
                const summary = await AigcService.generateSummary(editorContent.value);
                if (summary) {
                    // 在内容前面插入AI总结
                    editorContent.value = `> AI总结：${summary}\n\n${editorContent.value}`;
                }
            } catch (error) {
                console.error('AI总结生成失败:', error);
            } finally {
                isAiLoading.value = false;
            }
        };

        // AI内容改进建议
        const handleAiImprovement = async () => {
            if (!formData.value.id || !editorContent.value) return;
            
            isAiLoading.value = true;
            try {
                const improvement = await AigcService.generateImprovement(editorContent.value);
                if (improvement) {
                    // 在内容后面添加AI改进建议
                    editorContent.value = `${editorContent.value}\n\n---\n\n### AI改进建议：\n\n${improvement}`;
                }
            } catch (error) {
                console.error('AI改进建议生成失败:', error);
            } finally {
                isAiLoading.value = false;
            }
        };

        const editorContent = ref<string>(''); // 用于存储编辑器内容
        // const isLoading = computed(() => eventStore.isLoading); // isLoading 似乎未在模板中使用，如果确实不用可以移除
        // const isEditorInitialized = ref(false); // 由 VditorEditor 内部管理和通过事件通知

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

        const dateValue = computed({
            get: () => formData.value.ddl ? new Date(Number(formData.value.ddl)) : null,
            set: (date: Date | null) => {
                formData.value.ddl = date ? date.getTime().toString() : '';
            }
        });

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark') || document.body.classList.toString().includes('-dark');
        };

        // Vditor 实例现在由 VditorEditor 组件管理
        // const vditorComponentRef = ref<InstanceType<typeof VditorEditor> | null>(null); // 如果需要调用子组件方法

        const onEditorInitialized = () => {
            // console.log("Editor is ready in parent.");
            // isEditorInitialized.value = true;
            // 如果有初始加载逻辑，可以在这里触发，确保编辑器已准备好
            // 例如，如果内容加载依赖编辑器初始化，可以在这里调用 loadContent
            // 但由于 VditorEditor 内部会在 after 回调中设置初始值，可能不需要父组件再次设置
        };

        const loadContent = async () => {
            if (!formData.value.id) { // 使用 formData.value.id
                editorContent.value = ''; // 清空内容
                return;
            }
            try {
                const fetchedContent = await eventStore.getEventContent(formData.value.id);
                editorContent.value = fetchedContent || '';
                // VditorEditor 组件将通过 watch props.modelValue (即这里的 editorContent) 来更新其内部值
            } catch (error) {
                console.error('加载内容失败:', error);
                editorContent.value = '';
            }
        };

        const loadTags = async () => {
            try {
                allTags.value = await tagStore.fetchTags();
            } catch (error) {
                console.error('加载标签失败:', error);
            }
        };

        const getTagColor = (tag: Tag | undefined): string => {
            if (!tag || !tag.color) return 'default';
            if (typeof tag.color === 'string') return tag.color.toLowerCase();
            return String(tag.color).toLowerCase();
        };

        const openAddTagModal = () => {
            showAddTagModal.value = true;
        };

        const handleTagCreated = async (tagName: string) => {
            await loadTags();
            if (!selectedTags.value.includes(tagName)) {
                selectedTags.value = [...selectedTags.value, tagName];
            }
        };

        const handleOpen = async () => {
            // 确保 formData 更新后再加载内容，因为 editorContent 依赖 formData.id
            formData.value = { ...props.cardData };
            selectedTags.value = props.cardData.tag || [];

            await loadTags(); // 加载标签

            // 仅当 cardData.id 存在时才加载内容
            if (props.cardData.id) {
                await loadContent(); // 加载编辑器内容
            } else {
                editorContent.value = ''; // 新卡片，内容为空
            }
            // VditorEditor 会在其 onMounted 和 props.eventId watch 中自行初始化
        };

        const handleConfirm = async () => {
            try {
                // editorContent 已经通过 v-model 与 VditorEditor 同步
                if (formData.value.id && editorContent.value !== null) {
                    await eventStore.saveEventContent(formData.value.id, editorContent.value);
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
            // VditorEditor 的清理工作由其自身的 onBeforeUnmount 处理
            // isEditorInitialized.value = false; // 重置状态
            editorContent.value = ''; // 清空内容，以便下次打开是干净的
        };

        watch(
            () => props.modelValue,
            async (val) => {
                if (val) {
                    await nextTick(); // 确保DOM更新，特别是Teleport的内容
                    await handleOpen();
                } else {
                    // Modal 关闭时，可以做一些清理， editorContent 已经在 handleClose 中清理
                }
            },
            { immediate: true } // 初始打开时也执行
        );

        watch(
            () => props.cardData,
            async (newVal) => {
                if (props.modelValue) { // 仅当模态框可见时更新
                    const prevId = formData.value.id;
                    formData.value = { ...newVal }; // 更新表单数据
                    selectedTags.value = newVal.tag || [];

                    // 如果 ID 发生变化，或者从无 ID 变为有 ID，则重新加载内容
                    if (newVal.id && newVal.id !== prevId) {
                        await loadContent();
                    } else if (!newVal.id) {
                        editorContent.value = ''; // 如果新卡片没有ID（例如新建状态），清空内容
                    }
                    // VditorEditor 将通过其内部的 watch(props.eventId) 来响应 ID 变化
                }
            },
            { deep: true }
        );

        // 移除 Vditor 相关的 onBeforeUnmount
        // onBeforeUnmount 会在 VditorEditor 组件内部处理

        const minDate = new Date(2000, 0, 1);
        const maxDate = new Date(2100, 11, 31);

        // handleDateSelected 似乎未被直接使用，VDatePicker 直接通过 v-model 更新 dateValue -> formData.ddl
        // const handleDateSelected = (date: Date | null) => { ... }

        onErrorCaptured((err) => {
            if (String(err).includes('dayIndex')) {
                console.warn('日历组件数据异常，已拦截:', err);
                return false;
            }
            return true;
        });

        const availableIcons = [
            'mdi-home', 'mdi-account', 'mdi-briefcase',
            'mdi-shopping', 'mdi-star', 'mdi-bell',
            'mdi-calendar', 'mdi-note', 'mdi-check'
        ];

        const selectedIconIndex = computed({
            get: () => availableIcons.findIndex(icon => icon === formData.value.icon),
            set: (index: number) => {
                if (index >= 0 && index < availableIcons.length) {
                    formData.value.icon = availableIcons[index];
                } else {
                    formData.value.icon = '';
                }
            }
        });

        onMounted(() => {
            loadTags(); // 初始加载一次标签列表
            // handleOpen 将在 modelValue 变为 true 时被调用
        });

        // getDynamicLinkBase 已移至 VditorEditor

        return {
            formData,
            editorContent,
            selectedTags,
            allTags,
            isAiLoading,
            localAigcEnabled,
            handleAiSummary,
            handleAiImprovement,
            handleConfirm,
            handleClose,
            openAddTagModal,
            getTagColor,
            dateValue,
            minDate,
            maxDate,
            isDarkMode,
            availableIcons,
            selectedIconIndex,
            showAddTagModal,
            handleTagCreated,
            onEditorInitialized,
            // vditorComponentRef // 如果需要调用子组件方法时使用
        };
    }
});
</script>

<style scoped>
@import '@/styles/Modals/contentmodal.css';

.editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.ai-actions {
    display: flex;
    gap: 0.5rem;
}

.ai-actions .v-btn {
    margin: 0;
}
</style>

<style>
/* 全局 Vditor 样式导入可以保留在这里，或者移到 main.ts/App.vue */
/* 如果 VditorEditor.vue 已经导入了 'vditor/dist/index.css'，这里可能就不再需要了 */
/* 但是，你可能有一些自定义的全局 Vditor 覆盖样式 */
@import '@/styles/vditor.css';

.v-overlay--active {
    z-index: 9999999 !important;
}

.priority-select-menu {
    z-index: 9999999 !important;
    position: fixed !important;
}
</style>