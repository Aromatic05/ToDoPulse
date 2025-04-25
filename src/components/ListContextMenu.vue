<template>
    <!-- 单一全局菜单 -->
    <v-menu
        v-model="show"
        :activator="activatorElement"
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
            <v-card-title>删除列表</v-card-title>
            <v-card-text>
                确定要删除列表 "{{ deleteDialog.listTitle }}" 吗？此操作无法撤销。
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
import { ref, reactive, nextTick, computed } from 'vue';

interface ListItem {
    id: string;
    title: string;
    icon: string;
}

const props = defineProps({
    show: {
        type: Boolean,
        default: false
    },
    activatorElement: {
        type: [Object, null],
        default: null
    },
    targetList: {
        type: Object as () => ListItem | null,
        default: null
    },
    targetIndex: {
        type: Number,
        default: -1
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
    targetList: null as ListItem | null
});

// 删除确认对话框
const deleteDialog = reactive({
    show: false,
    listTitle: '',
    targetList: null as ListItem | null,
    targetIndex: -1
});

function triggerRename() {
    if (props.targetList) {
        renameDialog.targetList = props.targetList;
        renameDialog.newName = props.targetList.title;
        renameDialog.show = true;
        show.value = false;
    }
}

function triggerDelete() {
    if (props.targetList !== null && props.targetIndex !== -1) {
        deleteDialog.targetList = props.targetList;
        deleteDialog.listTitle = props.targetList.title;
        deleteDialog.targetIndex = props.targetIndex;
        deleteDialog.show = true;
        show.value = false;
    }
}

function confirmRename() {
    if (renameDialog.targetList && renameDialog.newName.trim() !== '') {
        emit('rename', renameDialog.targetList.id, renameDialog.newName.trim());
        renameDialog.show = false;
        renameDialog.targetList = null;
    }
}

function confirmDelete() {
    if (deleteDialog.targetList && deleteDialog.targetIndex !== -1) {
        emit('delete', deleteDialog.targetList.id, deleteDialog.targetIndex);
        deleteDialog.show = false;
        deleteDialog.targetList = null;
        deleteDialog.targetIndex = -1;
    }
}
</script>