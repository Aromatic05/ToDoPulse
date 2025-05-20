<template>
    <Teleport to="body">
        <div v-if="show" class="modal-mask tag-modal-mask" @click.self="closeDialog">
            <div class="modal-container tag-modal-container">
                <div class="modal-header">
                    <div class="text-h5 d-flex align-center">
                        <v-chip :color="props.tag.color" class="me-3" variant="tonal">{{ props.tag.name }}</v-chip>
                        <span>标签下的所有任务</span>
                        <v-spacer></v-spacer>
                        <span class="text-caption">({{ tagEvents.length }} 项)</span>
                    </div>
                </div>
                
                <v-divider></v-divider>
                
                <div class="modal-body">
                    <v-progress-circular v-if="isLoading" indeterminate color="primary" class="ma-4"></v-progress-circular>
                    <v-alert v-if="error" type="error" class="mb-4">{{ error }}</v-alert>
                    <v-data-table v-else :headers="headers" :items="tagEvents" :items-per-page="5"
                        class="elevation-1 rounded material-table" density="compact">
                        <template v-slot:item="{ item }">
                            <tr>
                                <td colspan="5" class="pa-2">
                                    <TagCard :data="{
                                        ...item,
                                        color: getPriorityColor(item.priority)
                                    }" @update="() => { }" @delete="removeTagFromEvent" />
                                </td>
                            </tr>
                        </template>
                        <template v-slot:no-data>
                            <p class="text-center pa-4">此标签下暂无任务</p>
                        </template>
                    </v-data-table>
                </div>
                
                <div class="modal-footer">
                    <v-spacer></v-spacer>
                    <button class="tag-modal-close-btn" @click="closeDialog">关闭</button>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import ListCard from '@/components/Cards/ListCard.vue';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { useTagStore, useEventStore } from '@/stores/index';
import TagCard from '@/components/Cards/TagCard.vue';

// 获取tagStore和eventStore
const tagStore = useTagStore();
const eventStore = useEventStore();

// Define interfaces
interface Tag {
    id: number;
    name: string;
    color: string;
    count: number;
}


type HeaderAlign = 'start' | 'end' | 'center';
interface DataTableHeader {
    title: string;
    key: string;
    sortable?: boolean;
    align?: HeaderAlign;
    width?: string;
}

// Props
const props = defineProps({
    modelValue: Boolean,
    tag: {
        type: Object as () => Tag,
        default: () => ({ id: 0, name: '', color: 'primary', count: 0 })
    }
});

const emit = defineEmits(['update:modelValue', 'tag-removed']);

// Local state
const show = ref(props.modelValue);
const tagEvents = ref<FEvent[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);

// Headers for the data table
const headers: DataTableHeader[] = [
    { title: '状态', key: 'status', sortable: false, align: 'center', width: '80px' },
    { title: '任务', key: 'title', sortable: true, align: 'start' },
    { title: '截止日期', key: 'date', sortable: true, align: 'center', width: '150px' },
    { title: '操作', key: 'actions', sortable: false, align: 'center', width: '100px' }
];

// 加载标签内容
async function loadTagContent() {
    if (!props.tag.name) return;
    
    isLoading.value = true;
    error.value = null;
    
    try {
        // 先清除特定标签的缓存
        tagStore.tagEvents.delete(props.tag.name);
        
        // 重新获取最新内容
        const events = await tagStore.getTagContent(props.tag.name);
        tagEvents.value = events;
    } catch (err) {
        console.error('加载标签内容失败:', err);
        error.value = `加载标签内容失败: ${err}`;
    } finally {
        isLoading.value = false;
    }
}

// Watch for changes in the modelValue prop
watch(() => props.modelValue, (val) => {
    show.value = val;
    if (val) {
        loadTagContent();
    }
});

// Watch for changes in the show value
watch(show, (val) => {
    emit('update:modelValue', val);
});

// Watch for changes in the tag prop
watch(() => props.tag, () => {
    if (show.value) {
        loadTagContent();
    }
}, { deep: true });

// Load tag content when component is mounted
onMounted(() => {
    if (show.value) {
        loadTagContent();
    }
});

// Get color based on priority
function getPriorityColor(priority: string) {
    switch (priority) {
        case 'High': return 'error';
        case 'Medium': return 'warning';
        case 'Low': return 'success';
        default: return 'grey';
    }
}

// Close the dialog
function closeDialog() {
    show.value = false;
}

// 从事件中移除当前标签
async function removeTagFromEvent(event: FEvent) {
    if (!event.tag || !props.tag.name) return;
    
    try {
        // 创建事件的副本，并确保tag数组存在
        const updatedEvent = {
            ...event,
            tag: event.tag ? event.tag.filter(tag => tag !== props.tag.name) : []
        };
        
        // 更新事件
        await eventStore.updateEvent(updatedEvent);
        
        // 手动从本地事件列表中移除该事件
        const index = tagEvents.value.findIndex(e => e.id === event.id);
        if (index !== -1) {
            tagEvents.value.splice(index, 1);
        }
        
        // 更新tagStore中的缓存
        if (tagStore.tagEvents.has(props.tag.name)) {
            const storedEvents = tagStore.tagEvents.get(props.tag.name) || [];
            const filteredEvents = storedEvents.filter(e => e.id !== event.id);
            tagStore.tagEvents.set(props.tag.name, filteredEvents);
        }
        
        // 通知父组件标签已被移除
        emit('tag-removed', props.tag.name);
    } catch (err) {
        console.error('移除标签失败:', err);
        error.value = `移除标签失败: ${err}`;
    }
}
</script>

<style scoped>
/* 导入共享模态框样式 */
@import '@/styles/Modals/contentmodal.css';

/* 覆盖z-index以确保在CardContentModal下层 */
.tag-modal-mask {
    z-index: 1000 !important; /* 低于CardContentModal的z-index 99999 */
}

.tag-modal-container {
    z-index: 1001 !important; /* 低于CardContentModal的z-index 10002 */
}

/* 按钮样式 */
.tag-modal-close-btn {
    background: var(--md-sys-color-primary);
    color: white;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
}

.tag-modal-close-btn:hover {
    opacity: 0.9;
}
.modal-card {
    max-height: 80vh;
    overflow-y: auto;
}

/* Reuse styles from ListView.vue */
:deep(.v-data-table) {
    background-color: var(--md-sys-color-surface-container) !important;
    color: var(--md-sys-color-on-surface) !important;
    border-radius: 18px;
    border: 1px solid var(--md-sys-color-outline-variant);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
    transition: all 0.3s ease;
}

:deep(.v-data-table__header) {
    background-color: var(--md-sys-color-surface-container-high) !important;
}

:deep(.v-data-table__header th) {
    color: var(--md-sys-color-on-surface) !important;
    font-weight: 600;
    border-bottom: 1px solid var(--md-sys-color-outline-variant) !important;
}

:deep(.v-data-table__row) {
    transition: background-color 0.2s ease;
    border-bottom: 1px solid var(--md-sys-color-outline-variant) !important;
}

:deep(.v-data-table__row td) {
    padding-top: 4px !important;
    padding-bottom: 4px !important;
}

:deep(.v-data-table__row:hover) {
    background-color: var(--md-sys-color-surface-container-high) !important;
}

:deep(.v-data-table-footer) {
    background-color: var(--md-sys-color-surface-container) !important;
    color: var(--md-sys-color-on-surface) !important;
}
</style>
