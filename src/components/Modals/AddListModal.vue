<template>
    <v-dialog v-model="dialogVisible" max-width="500px" persistent>
        <v-card>
            <v-card-title class="text-h5">
                创建新列表
            </v-card-title>

            <v-card-text>
                <v-form ref="form" v-model="valid" @submit.prevent="createListLocal">
                    <v-text-field v-model="listName" label="列表名称" :rules="[v => !!v || '名称不能为空']" required
                        variant="outlined" class="mb-3"></v-text-field>

                    <p class="text-subtitle-1 mb-2">选择图标</p>
                    <v-chip-group v-model="selectedIconIndex" column mandatory>
                        <v-chip v-for="(icon, index) in availableIcons" :key="index" filter :value="index">
                            <v-icon :icon="icon"></v-icon>
                        </v-chip>
                    </v-chip-group>
                </v-form>
            </v-card-text>

            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="grey-darken-1" variant="text" @click="closeModal">
                    取消
                </v-btn>
                <v-btn color="primary" variant="elevated" :disabled="!valid" @click="createListLocal">
                    创建
                </v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
// 导入 createListAndStore 函数
import { createList } from '@/services/GetListsService';

const props = defineProps({
    show: {
        type: Boolean,
        default: false
    }
});

const emit = defineEmits(['update:show', 'create']);

// 表单数据
const listName = ref('');
const selectedIconIndex = ref(0);
const valid = ref(false);
// 添加类型注解
const form = ref<{ resetValidation: () => void } | null>(null);

// 可用的图标列表
const availableIcons = [
    'mdi-format-list-bulleted',
    'mdi-home',
    'mdi-account',
    'mdi-briefcase',
    'mdi-shopping',
    'mdi-star',
    'mdi-school',
    'mdi-book',
    'mdi-basket',
    'mdi-heart',
    'mdi-account-group',
    'mdi-food',
    'mdi-gift',
    'mdi-cart',
    'mdi-alarm',
    'mdi-airplane',
    'mdi-wallet'
];

// 控制对话框显示
const dialogVisible = computed({
    get: () => props.show,
    set: (value) => emit('update:show', value)
});

// 监听对话框关闭时重置表单
watch(dialogVisible, (newVal) => {
    if (!newVal) {
        resetForm();
    }
});

// 重置表单
function resetForm() {
    listName.value = '';
    selectedIconIndex.value = 0;
    if (form.value) {
        form.value.resetValidation();
    }
}

// 关闭模态框
function closeModal() {
    dialogVisible.value = false;
}

// 创建列表
async function createListLocal() {
    if (!valid.value) return;

    const selectedIcon = availableIcons[selectedIconIndex.value];
    
    try {
        // 调用服务创建并存储列表
        const updatedLists = await createList(listName.value, selectedIcon);
        // 发出创建成功的事件，传递列表名称、图标和更新后的列表
        emit('create', updatedLists);
        closeModal();
    } catch (error) {
        console.error('创建列表失败:', error);
        // 这里可以添加错误处理，比如显示错误提示
    }
}
</script>