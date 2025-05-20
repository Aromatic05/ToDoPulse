<template>
    <!-- 单一全局菜单 -->
    <v-menu
        v-model="show"
        :activator="activatorElement as unknown as Element"
        location="end"
        :close-on-content-click="true"
        :close-on-back="true"
        :open-on-click="false"
    >
        <v-list density="compact" bg-color="var(--md-sys-color-surface-container-high)">
            <v-list-item @click="triggerRename" prepend-icon="mdi-pencil" title="重命名"></v-list-item>
            <v-list-item @click="triggerDelete" prepend-icon="mdi-delete" title="删除"></v-list-item>
        </v-list>
    </v-menu>

    <!-- 重命名对话框 -->
    <v-dialog v-model="renameDialog.show" max-width="500px">
        <v-card>
            <v-card-title>重命名列表</v-card-title>
            <v-card-text>
                <v-text-field 
                    v-model="renameDialog.newName" 
                    label="列表名称" 
                    placeholder="输入新名称"
                    variant="outlined" 
                    density="compact" 
                    autofocus
                    @keyup.enter="confirmRename"
                ></v-text-field>
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="primary" variant="text" @click="renameDialog.show = false">取消</v-btn>
                <v-btn color="primary" variant="elevated" @click="confirmRename">确定</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>

    <!-- 删除确认对话框 -->
    <v-dialog v-model="deleteDialog.show" max-width="500px">
        <v-card>
            <v-card-title>删除标签</v-card-title>
            <v-card-text>
                确定要删除标签 "{{ deleteDialog.tagName }}" 吗？此操作无法撤销。
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="primary" variant="text" @click="deleteDialog.show = false">取消</v-btn>
                <v-btn color="error" variant="elevated" @click="confirmDelete">删除</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>

<script setup lang="ts">
import { reactive, computed } from 'vue';
import { Tag } from '@/services/TagService';

// 定义一个更通用的标签接口，兼容UI中使用的标签类型
interface TagLike {
    id: number | string;
    name: string;
    color: string | any; // 可以是字符串或TagColor枚举
    [key: string]: any; // 允许其他任意属性
}

const props = defineProps({
    show: {
        type: Boolean,
        default: false
    },
    activatorElement: {
        type: Object,
        default: undefined
    },
    targetTag: {
        type: Object as () => TagLike | null,
        default: null
    }
});

const emit = defineEmits(['update:show', 'rename', 'delete']);

// 菜单显示状态
const show = computed({
    get: () => props.show,
    set: (value) => {
        emit('update:show', value);
    }
});

// 重命名对话框
const renameDialog = reactive({
    show: false,
    newName: '',
    targetTag: null as TagLike | null
});

// 删除确认对话框
const deleteDialog = reactive({
    show: false,
    tagName: '',
    targetTag: null as TagLike | null
});

function triggerRename() {
    if (props.targetTag) {
        renameDialog.targetTag = props.targetTag;
        renameDialog.newName = props.targetTag.name;
        renameDialog.show = true;
        show.value = false;
    }
}

function triggerDelete() {
    if (props.targetTag) {
        deleteDialog.targetTag = props.targetTag;
        deleteDialog.tagName = props.targetTag.name;
        deleteDialog.show = true;
        show.value = false;
    }
}

function confirmRename() {
    if (renameDialog.targetTag && renameDialog.newName.trim() !== '') {
        emit('rename', renameDialog.targetTag.name, renameDialog.newName.trim());
        renameDialog.show = false;
        renameDialog.targetTag = null;
    }
}

function confirmDelete() {
    if (deleteDialog.targetTag) {
        emit('delete', deleteDialog.targetTag.name);
        deleteDialog.show = false;
        deleteDialog.targetTag = null;
    }
}
</script>