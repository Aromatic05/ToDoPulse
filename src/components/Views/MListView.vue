<template>
    <div class="m-list-view">
        <!-- 添加返回按钮和标题布局 -->
        <div class="d-flex align-center mb-4">
            <v-btn icon="mdi-arrow-left" variant="text" density="comfortable" class="me-2" 
                @click="navigateBack"></v-btn>
            <h1 class="text-h5 text-sm-h4 mb-0">{{ listTitle }}</h1>
        </div>

        <!-- 显示/隐藏已完成任务的开关 - 单独一行 -->
        <div class="d-flex justify-end mb-4">
            <v-switch v-model="showCompleted" color="primary" hide-details density="compact"
                :label="showCompleted ? '显示已完成' : '隐藏已完成'" class="switch-label"></v-switch>
        </div>

        <!-- 移动端优化的任务添加区域 -->
        <v-row>
            <v-col cols="12">
                <v-text-field v-model="newEvent" label="添加新任务" append-icon="mdi-plus" @click:append="addNewEvent"
                    class="mb-3" density="comfortable" variant="outlined" hide-details></v-text-field>
            </v-col>
        </v-row>

        <!-- 移动端优化的任务列表 - 使用MListCard组件 -->
        <div v-if="filteredEvents.length > 0" class="task-list">
            <div v-for="event in filteredEvents" :key="event.id" class="mb-3">
                <MListCard :data="{
                    ...event,
                    color: getPriorityColor(event.priority)
                }" @update="handleEventUpdate($event)" @delete="deleteFEvent(event)" />
            </div>
        </div>
        <v-card v-else class="pa-4 text-center">
            <p class="text-body-1 text-medium-emphasis">暂无任务</p>
        </v-card>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onActivated, onDeactivated } from 'vue'
import MListCard from '@/components/Cards/MListCard.vue';  // 替换为移动端优化的MListCard
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { useListStore } from '@/stores/listStore';
import { useEventStore } from '@/stores/eventStore';

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

// 根据列表ID加载数据
watch(() => props.viewId, loadListData, { immediate: true })

// 添加 activated 生命周期钩子 - 当组件被 keep-alive 激活时调用
onActivated(() => {
    console.log('MListView 组件被激活', listId.value);
    // 当视图被重新激活时，刷新数据以确保数据最新
    loadListData();
})

// 添加 deactivated 生命周期钩子 - 当组件被 keep-alive 停用时调用
onDeactivated(() => {
    console.log('MListView 组件被停用', listId.value);
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

// 添加导航回列表视图的函数
function navigateBack() {
    // 使用与应用程序一致的导航方式
    window.dispatchEvent(new CustomEvent('navigation', {
        detail: { route: 'lists' }
    }));
}

</script>

<style scoped>
.m-list-view {
    padding: 12px;
    max-width: 100%;
}

.task-list {
    display: flex;
    flex-direction: column;
}

/* 移动端优化样式 */
@media (max-width: 600px) {
    .switch-label :deep(.v-label) {
        font-size: 0.875rem;
    }

    h1 {
        margin-bottom: 12px;
    }
}

/* Material Design 3 样式覆盖 */
:deep(.v-card) {
    background-color: var(--md-sys-color-surface-container) !important;
    color: var(--md-sys-color-on-surface) !important;
    border-radius: 16px;
    border: 1px solid var(--md-sys-color-outline-variant);
    box-shadow: 0 1px 8px rgba(0, 0, 0, 0.06);
}

:deep(.v-card:hover) {
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1) !important;
}

/* 输入框样式 */
:deep(.v-field) {
    border-radius: 12px !important;
}
</style>