<template>
    <div class="list-view">
        <h1 class="text-h4 mb-6">{{ listTitle }}</h1>

        <v-row>
            <v-col cols="12">
                <v-text-field v-model="newEvent" label="添加新任务" append-icon="mdi-plus" @click:append="addNewEvent"
                    @keyup.enter="addNewEvent" class="mb-4"></v-text-field>
            </v-col>
        </v-row>

        <v-data-table :headers="headers" :items="Events" :items-per-page="10" class="elevation-1 rounded">
            <template v-slot:item="{ item }">
                <tr>
                    <td colspan="5" class="pa-2">
                        <ListCard 
                            :data="{ 
                                id: item.id, 
                                title: item.title, 
                                date: item.date, 
                                finished: item.finished,
                                tags: [item.priority],
                                color: getPriorityColor(item.priority)
                            }" 
                            @update="handleEventUpdate($event, item)"
                            @delete="deleteFEvent(item)"
                        />
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
import { ref, computed, watch } from 'vue'
import { getLists } from '@/services/GetListsService.ts';
import { getEventsBylistid, addEvent, toggleEventStatus, updateEvent, deleteEvent } from '@/services/ListDataService';
import ListCard from '@/components/Cards/ListCard.vue';  // 导入ListCard组件
import { FEvent } from 'src-tauri/bindings/FEvent';
import { Priority } from 'src-tauri/bindings/Priority';

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
    const match = props.viewId.match(/list\/([^\/]+)/)
    return match ? match[1] : null
})

// 使用服务获取数据，不再使用硬编码数据
const Events = ref<FEvent[]>([])

const newEvent = ref('')

// 添加类型注解
const headers: DataTableHeader[] = [
    { title: '状态', key: 'status', sortable: false, align: 'center', width: '80px' },
    { title: '任务', key: 'title', sortable: true, align: 'start' },
    { title: '优先级', key: 'priority', sortable: true, align: 'center', width: '120px' },
    { title: '截止日期', key: 'date', sortable: true, align: 'center', width: '150px' },
    { title: '操作', key: 'actions', sortable: false, align: 'center', width: '100px' }
]

// 根据列表ID加载数据
watch(() => props.viewId, loadListData, { immediate: true })

async function loadListData() {
    if (listId.value) {
        try {
            // 获取列表信息
            const lists = await getLists();
            const currentList = lists.find(list => list.id === listId.value);

            if (currentList) {
                listTitle.value = currentList.title;
                console.log(`加载列表: ${currentList.title} (ID: ${currentList.id})`);

                // 获取该列表的任务
                Events.value = await getEventsBylistid(listId.value);
            } else {
                listTitle.value = '未找到列表';
                console.error(`未找到ID为 ${listId.value} 的列表`);
                Events.value = [];
            }
        } catch (error) {
            console.error('加载列表数据失败:', error);
            listTitle.value = '加载失败';
            Events.value = [];
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
            // 使用服务添加任务
            Events.value = await addEvent(
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

async function toggleEvent(Event: FEvent) {
    if (listId.value) {
        try {
            Events.value = await toggleEventStatus(Event.id, listId.value, Event.finished);
        } catch (error) {
            console.error('更新任务状态失败:', error);
        }
    }
}

async function editEvent(Event: FEvent) {
    console.log(`编辑任务: ${Event.title}`);
    // 这里可以显示编辑对话框，然后调用 updateEvent 服务
    // 示例：如果有编辑对话框的结果
    const updatedFields = { title: '新标题', priority: "High" as Priority, date: '2025-05-01' };
    if (listId.value) {
        Events.value = await updateEvent(Event.id, listId.value, updatedFields);
    } else {
        console.error('更新任务失败: listId 为 null');
    }
}

async function deleteFEvent(Event: FEvent) {
    if (listId.value) {
        try {
            Events.value = await deleteEvent(Event.id, listId.value);
        } catch (error) {
            console.error('删除任务失败:', error);
        }
    }
}

// 添加处理ListCard更新的函数
async function handleEventUpdate(updatedData: any, originalEvent: FEvent) {
    if (listId.value) {
        try {
            // 处理完成状态切换
            if (updatedData.Finished !== originalEvent.finished) {
                Events.value = await toggleEventStatus(originalEvent.id, listId.value, updatedData.finished);
                return;
            }
            
            // 处理其他更新
            const updatedFields = { 
                title: updatedData.title, 
                priority: updatedData.tags?.[0] || originalEvent.priority, 
                date: updatedData.date || originalEvent.date,
            };
            
            Events.value = await updateEvent(originalEvent.id, listId.value, updatedFields);
        } catch (error) {
            console.error('更新任务失败:', error);
        }
    }
}
</script>

<style scoped>
.list-view {
    max-width: 1000px;
    margin: 0 auto;
}
</style>