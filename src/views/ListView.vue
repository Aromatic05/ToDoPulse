<template>
    <div class="list-view">
        <h1 class="text-h4 mb-6">{{ listTitle }}</h1>

        <v-row>
            <v-col cols="12">
                <v-text-field v-model="newTask" label="添加新任务" append-icon="mdi-plus" @click:append="addTask"
                    @keyup.enter="addTask" class="mb-4"></v-text-field>
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
                <v-icon small @click="deleteTask(item)">
                    mdi-delete
                </v-icon>
            </template>
        </v-data-table>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'

const props = defineProps({
    viewId: {
        type: String,
        required: true
    }
})

const listTitle = ref('我的列表')
const listId = computed(() => {
    const match = props.viewId.match(/list\/(\d+)/)
    return match ? match[1] : null
})

// 模拟数据，实际应从API获取
const tasks = ref([
    { id: 1, title: '完成项目方案', completed: false, priority: '高', dueDate: '2025-04-28' },
    { id: 2, title: '准备周会演示', completed: true, priority: '中', dueDate: '2025-04-26' },
    { id: 3, title: '回复客户邮件', completed: false, priority: '高', dueDate: '2025-04-25' },
    { id: 4, title: '更新项目文档', completed: false, priority: '低', dueDate: '2025-04-30' }
])

const newTask = ref('')

const headers = [
    { title: '状态', key: 'status', sortable: false, align: 'center', width: '80px' },
    { title: '任务', key: 'title', sortable: true, align: 'start' },
    { title: '优先级', key: 'priority', sortable: true, align: 'center', width: '120px' },
    { title: '截止日期', key: 'dueDate', sortable: true, align: 'center', width: '150px' },
    { title: '操作', key: 'actions', sortable: false, align: 'center', width: '100px' }
]

// 根据列表ID加载数据
watch(() => props.viewId, loadListData, { immediate: true })

function loadListData() {
    if (listId.value) {
        // 这里应该从API获取数据
        console.log(`加载列表ID: ${listId.value}的数据`)

        // 模拟获取列表标题
        if (listId.value === '1') {
            listTitle.value = '工作'
        } else if (listId.value === '2') {
            listTitle.value = '个人'
        } else {
            listTitle.value = `列表 ${listId.value}`
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

function addTask() {
    if (newTask.value.trim()) {
        tasks.value.push({
            id: Date.now(),
            title: newTask.value,
            completed: false,
            priority: '中',
            dueDate: new Date().toISOString().substring(0, 10)
        })
        newTask.value = ''
    }
}

function toggleTask(task: any) {
    console.log(`任务 "${task.title}" 状态变更为: ${task.completed ? '已完成' : '未完成'}`)
}

function editTask(task: any) {
    console.log(`编辑任务: ${task.title}`)
    // 实现编辑功能
}

function deleteTask(task: any) {
    const index = tasks.value.findIndex(t => t.id === task.id)
    if (index !== -1) {
        tasks.value.splice(index, 1)
    }
}
</script>

<style scoped>
.list-view {
    max-width: 1000px;
    margin: 0 auto;
}
</style>