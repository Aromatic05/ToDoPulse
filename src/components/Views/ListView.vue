<template>
    <div class="list-view">
        <div class="d-flex align-center justify-space-between mb-6">
            <h1 class="text-h4">{{ listTitle }}</h1>

            <!-- 显示/隐藏已完成任务的开关 -->
            <v-switch v-model="showCompleted" color="primary" hide-details density="compact"
                :label="showCompleted ? '显示已完成任务' : '隐藏已完成任务'"></v-switch>
        </div>

        <v-row>
            <v-col cols="12">
                <v-text-field v-model="newEvent" label="添加新任务" append-icon="mdi-plus" @click:append="addNewEvent"
                    class="mb-4"></v-text-field>
            </v-col>
        </v-row>

        <v-data-table :headers="headers" :items="filteredEvents" :items-per-page="10"
            class="elevation-1 rounded material-table">
            <template v-slot:item="{ item }">
                <tr>
                    <td colspan="5" class="pa-2">
                        <ListCard :data="{
                            ...item,
                            color: getPriorityColor(item.priority)
                        }" @update="handleEventUpdate($event)" @delete="deleteFEvent(item)" />
                    </td>
                </tr>
            </template>
            <template v-slot:no-data>
                <p class="text-center pa-4">暂无任务</p>
            </template>
        </v-data-table>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onActivated, onDeactivated } from 'vue'
import ListCard from '@/components/Cards/ListCard.vue';  // 导入ListCard组件
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { useListStore } from '@/stores/listStore';
import { useEventStore } from '@/stores/eventStore';

// 添加这个类型定义
type HeaderAlign = 'start' | 'end' | 'center';
interface DataTableHeader {
    title: string;
    key: string;
    sortable?: boolean;
    align?: HeaderAlign;
    width?: string;
}

const props = defineProps({
    viewId: {
        type: String,
        required: true
    }
})

const listTitle = ref('我的列表')
const listId = computed(() => {
    let match: RegExpMatchArray | null = null;
    if (props.viewId.startsWith('list/')) {
        match = props.viewId.match(/list\/([^/]+)/);
    } else if (props.viewId.startsWith('list-item/')) {
        match = props.viewId.match(/list-item\/([^/]+)/);
    }
    return match ? match[1] : null;
})

// 使用Pinia store
const listStore = useListStore();
const eventStore = useEventStore();

// 直接从store获取数据
const Events = computed(() => eventStore.getEventsByListId(listId.value || ''));

// 添加过滤后的事件计算属性
const filteredEvents = computed(() => {
    if (showCompleted.value) {
        return Events.value;
    } else {
        return Events.value.filter(event => !event.finished);
    }
});

const newEvent = ref('')

// 添加类型注解
const headers: DataTableHeader[] = [
    { title: '状态', key: 'status', sortable: false, align: 'center', width: '80px' },
    { title: '任务', key: 'title', sortable: true, align: 'start', width: '300px' },
    { title: '标签', key: 'priority', sortable: true, align: 'start', width: '100px' },
    { title: '截止日期', key: 'date', sortable: true, align: 'center', width: '150px' },
    { title: '操作', key: 'actions', sortable: false, align: 'center', width: '100px' }
]

// 根据列表ID加载数据
watch(() => props.viewId, loadListData, { immediate: true })

// 添加 activated 生命周期钩子 - 当组件被 keep-alive 激活时调用
onActivated(() => {
    console.log('ListView 组件被激活', listId.value);
    // 当视图被重新激活时，刷新数据以确保数据最新
    loadListData();
})

// 添加 deactivated 生命周期钩子 - 当组件被 keep-alive 停用时调用
onDeactivated(() => {
    console.log('ListView 组件被停用', listId.value);
    // 可以在这里保存当前视图的状态或执行其他清理操作
    // 例如：保存当前滚动位置、筛选条件等
})

async function loadListData() {
    if (listId.value) {
        try {
            // 从store获取列表信息
            await listStore.fetchLists();
            const currentList = listStore.getListById(listId.value);

            if (currentList) {
                listTitle.value = currentList.title;

                // 从store获取该列表的任务
                await eventStore.fetchEventsByListId(listId.value);
            } else {
                listTitle.value = '未找到列表';
                console.error(`未找到ID为 ${listId.value} 的列表`);
            }
        } catch (error) {
            console.error('加载列表数据失败:', error);
            listTitle.value = '加载失败';
        }
    }
}

function getPriorityColor(priority: string) {
    switch (priority) {
        case 'High': return 'red';
        case 'Medium': return 'orange';
        case 'Low': return 'green';
        default: return 'grey';
    }
}

async function addNewEvent() {
    if (newEvent.value.trim() && listId.value) {
        try {
            // 使用store添加任务
            await eventStore.addEvent(
                listId.value,
                newEvent.value,
                'Medium',
            );
            newEvent.value = '';
        } catch (error) {
            console.error('添加任务失败:', error);
        }
    }
}

// async function toggleEvent(Event: FEvent) {
//     if (listId.value) {
//         try {
//             Events.value = await toggleEventStatus(Event.id, listId.value, Event.finished);
//         } catch (error) {
//             console.error('更新任务状态失败:', error);
//         }
//     }
// }

// async function editEvent(Event: FEvent) {
//     console.log(`编辑任务: ${Event.title}`);
//     // 这里可以显示编辑对话框，然后调用 updateEvent 服务
//     // 示例：如果有编辑对话框的结果
//     const updatedFields = { title: '新标题', priority: "High" as Priority, date: '2025-05-01' };
//     if (listId.value) {
//         Events.value = await updateEvent(Event.id, listId.value, updatedFields);
//     } else {
//         console.error('更新任务失败: listId 为 null');
//     }
// }

async function deleteFEvent(Event: FEvent) {
    if (listId.value) {
        try {
            await eventStore.deleteEvent(Event.id, listId.value);
        } catch (error) {
            console.error('删除任务失败:', error);
        }
    }
}

// 添加处理ListCard更新的函数
async function handleEventUpdate(updatedData: FEvent) {
    if (listId.value) {
        try {
            await eventStore.updateEvent(updatedData);
        } catch (error) {
            console.error('更新任务失败:', error);
        }
    }
}

// 新增响应式变量控制已完成任务的显示/隐藏
const showCompleted = ref(true);

</script>

<style scoped>
.list-view {
    max-width: 1000px;
    margin: 0 auto;
}

/* 使用Material Design变量适配v-data-table */
:deep(.v-data-table) {
    /* 表格背景使用与卡片相同的表面容器颜色 */
    background-color: var(--md-sys-color-surface-container) !important;
    color: var(--md-sys-color-on-surface) !important;
    border-radius: 18px;
    border: 1px solid var(--md-sys-color-outline-variant);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
    transition: all 0.3s ease;
}

/* 表头样式 */
:deep(.v-data-table__header) {
    background-color: var(--md-sys-color-surface-container-high) !important;
}

:deep(.v-data-table__header th) {
    color: var(--md-sys-color-on-surface) !important;
    font-weight: 600;
    border-bottom: 1px solid var(--md-sys-color-outline-variant) !important;
}

/* 表格行 */
:deep(.v-data-table__row) {
    transition: background-color 0.2s ease;
    border-bottom: 1px solid var(--md-sys-color-outline-variant) !important;
}

/* 表格行中的单元格 */
:deep(.v-data-table__row td) {
    padding-top: 4px !important;
    padding-bottom: 4px !important;
}

:deep(.v-data-table__row:hover) {
    background-color: var(--md-sys-color-surface-container-high) !important;
}

/* 分页控件 */
:deep(.v-data-table-footer) {
    background-color: var(--md-sys-color-surface-container) !important;
    color: var(--md-sys-color-on-surface) !important;
}

:deep(.v-pagination__item) {
    color: var(--md-sys-color-on-surface) !important;
}

:deep(.v-pagination__item--active) {
    background-color: var(--md-sys-color-primary) !important;
    color: var(--md-sys-color-on-primary) !important;
}

/* 无数据提示 */
:deep(.v-data-table__empty-wrapper) {
    color: var(--md-sys-color-on-surface-variant) !important;
}

/* 如果表格有阴影，可以使用与卡片一致的阴影样式 */
:deep(.elevation-1) {
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08) !important;
}

:deep(.elevation-1:hover) {
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.12) !important;
}
</style>