<template>
    <div class="list-view">
        <h1 class="text-h4 mb-6">{{ listTitle }}</h1>

        <v-row>
            <v-col cols="12">
                <v-text-field v-model="newTask" label="添加新任务" append-icon="mdi-plus" @click:append="addNewTask"
                    @keyup.enter="addNewTask" class="mb-4"></v-text-field>
            </v-col>
        </v-row>

        <v-data-table :headers="headers" :items="tasks" :items-per-page="10" class="elevation-1 rounded">
            <template v-slot:item.status="{ item }">
                <v-checkbox v-model="item.completed" hide-details @change="toggleTask(item)"></v-checkbox>
            </template>

            <template v-slot:item.title="{ item }">
                <div :class="{ 'text-decoration-line-through': item.completed }">
                    {{ item.title }}
                </div>
            </template>

            <template v-slot:item.priority="{ item }">
                <v-chip :color="getPriorityColor(item.priority)" size="small" text-color="white">
                    {{ item.priority }}
                </v-chip>
            </template>

            <template v-slot:item.actions="{ item }">
                <v-icon small class="mr-2" @click="editTask(item)">
                    mdi-pencil
                </v-icon>
                <v-icon small @click="deleteTaskItem(item)">
                    mdi-delete
                </v-icon>
            </template>
        </v-data-table>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { getLists} from '@/services/GetListsServices';
import { getTasksByListId, addTask, toggleTaskStatus, updateTask, deleteTask, TaskItem } from '@/services/ListDataService';

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
const tasks = ref<TaskItem[]>([])

const newTask = ref('')

// 添加类型注解
const headers: DataTableHeader[] = [
    { title: '状态', key: 'status', sortable: false, align: 'center', width: '80px' },
    { title: '任务', key: 'title', sortable: true, align: 'start' },
    { title: '优先级', key: 'priority', sortable: true, align: 'center', width: '120px' },
    { title: '截止日期', key: 'dueDate', sortable: true, align: 'center', width: '150px' },
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
                tasks.value = await getTasksByListId(listId.value);
            } else {
                listTitle.value = '未找到列表';
                console.error(`未找到ID为 ${listId.value} 的列表`);
                tasks.value = [];
            }
        } catch (error) {
            console.error('加载列表数据失败:', error);
            listTitle.value = '加载失败';
            tasks.value = [];
        }
    }
}

function getPriorityColor(priority: string) {
    switch (priority) {
        case '高': return 'red';
        case '中': return 'orange';
        case '低': return 'green';
        default: return 'grey';
    }
}

async function addNewTask() {
    if (newTask.value.trim() && listId.value) {
        try {
            // 使用服务添加任务
            tasks.value = await addTask(
                listId.value,
                newTask.value,
                '中'
            );
            newTask.value = '';
        } catch (error) {
            console.error('添加任务失败:', error);
        }
    }
}

async function toggleTask(task: TaskItem) {
    if (listId.value) {
        try {
            tasks.value = await toggleTaskStatus(task.id, listId.value, task.completed);
        } catch (error) {
            console.error('更新任务状态失败:', error);
        }
    }
}

async function editTask(task: TaskItem) {
    console.log(`编辑任务: ${task.title}`);
    // 这里可以显示编辑对话框，然后调用 updateTask 服务
    // 示例：如果有编辑对话框的结果
    const updatedFields = { title: '新标题', priority: '高' as '高' | '中' | '低', dueDate: '2025-05-01' };
    if (listId.value) {
        tasks.value = await updateTask(task.id, listId.value, updatedFields);
    } else {
        console.error('更新任务失败: listId 为 null');
    }
}

async function deleteTaskItem(task: TaskItem) {
    if (listId.value) {
        try {
            tasks.value = await deleteTask(task.id, listId.value);
        } catch (error) {
            console.error('删除任务失败:', error);
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