<template>
    <div class="lists-view">
        <v-container fluid class="pa-0">
            <!-- 添加列表按钮 -->
            <v-btn color="primary" prepend-icon="mdi-plus" class="mb-4" block @click="showAddListModal">
                添加列表
            </v-btn>

            <!-- 列表展示区域 -->
            <div class="lists-container">
                <template v-if="lists.length > 0">
                    <v-list>
                        <div v-for="(list, index) in lists" :key="list.id" class="list-item-wrapper mb-2">
                            <v-list-item :value="list" rounded="lg" @click="handleClick(`list-item/${list.id}`)"
                                v-ripple v-touch="{
                                    left: () => showEditButtons(index),
                                    right: () => hideEditButtons(index)
                                }">
                                <template v-slot:prepend>
                                    <v-icon :icon="list.icon" class="me-2"></v-icon>
                                </template>

                                <v-list-item-title>{{ list.title }}</v-list-item-title>
                            </v-list-item>

                            <!-- 编辑按钮组 -->
                            <div class="edit-buttons" :class="{ 'show': editingIndex === index }">
                                <v-btn icon="mdi-pencil" color="primary" variant="text" density="comfortable"
                                    @click.stop="showRenameDialog(list)"></v-btn>
                                <v-btn icon="mdi-delete" color="error" variant="text" density="comfortable"
                                    @click.stop="confirmDelete(list)"></v-btn>
                            </div>
                        </div>
                    </v-list>

                    <!-- 首次使用提示 -->
                    <div class="list-hint" v-if="lists.length > 0">
                        向左滑动列表项可显示更多操作
                    </div>
                </template>
                <v-alert v-else type="info" variant="tonal" text="还没有任何列表，点击上方按钮创建一个吧！"></v-alert>
            </div>
        </v-container>

        <!-- 添加列表弹窗 -->
        <AddListModal v-model:show="addListModalVisible" @create="handleAddList" />

        <!-- 重命名对话框 -->
        <v-dialog v-model="editDialog.show" max-width="500px">
            <v-card>
                <v-card-title>重命名列表</v-card-title>
                <v-card-text>
                    <v-text-field v-model="editDialog.newName" label="列表名称" variant="outlined"
                        hide-details="auto"></v-text-field>
                </v-card-text>
                <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn color="grey" variant="text" @click="editDialog.show = false">
                        取消
                    </v-btn>
                    <v-btn color="primary" variant="text" @click="handleRename(editDialog.list?.id, editDialog.newName)"
                        :disabled="!editDialog.newName">
                        确定
                    </v-btn>
                </v-card-actions>
            </v-card>
        </v-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useListStore } from '@/stores';
import { FList } from 'src-tauri/bindings/FList';
import AddListModal from '@/components/Modals/AddListModal.vue';

// 初始化listStore
const listStore = useListStore();

// 列表数据
const lists = ref<FList[]>([]);

// 在组件挂载时获取列表
onMounted(async () => {
    try {
        lists.value = await listStore.fetchLists();
    } catch (error) {
        console.error('获取列表失败:', error);
    }
});

// 编辑状态
const editingIndex = ref(-1);
const editDialog = reactive({
    show: false,
    list: null as FList | null,
    newName: ''
});

// 显示/隐藏编辑按钮
function showEditButtons(index: number) {
    editingIndex.value = index;
}

function hideEditButtons(index: number) {
    if (editingIndex.value === index) {
        editingIndex.value = -1;
    }
}

// 重命名对话框
function showRenameDialog(list: FList) {
    editDialog.list = list;
    editDialog.newName = list.title;
    editDialog.show = true;
}

// 确认删除对话框
function confirmDelete(list: FList) {
    if (confirm(`确定要删除列表 "${list.title}" 吗？`)) {
        handleDelete(list.id);
    }
}

// 重命名列表
async function handleRename(id: string | undefined, newName: string) {
    if (!id || !newName) return;

    try {
        lists.value = await listStore.renameList(id, newName);
        editDialog.show = false;
    } catch (error) {
        console.error('重命名列表失败:', error);
    }
}

// 删除列表
async function handleDelete(id: string) {
    try {
        lists.value = await listStore.deleteList(id);
        editingIndex.value = -1;
    } catch (error) {
        console.error('删除列表失败:', error);
    }
}

// 添加列表模态框状态
const addListModalVisible = ref(false);

// 显示添加列表模态框
function showAddListModal() {
    addListModalVisible.value = true;
}

// 处理添加列表
async function handleAddList() {
    try {
        lists.value = await listStore.fetchLists();
    } catch (error) {
        console.error('获取列表失败:', error);
    }
}

// 检测是否为移动设备
function isMobileDevice() {
    return window.innerWidth <= 768;
}

// 处理列表点击 - 修改以支持移动端路由
function handleClick(route: string) {
    // 如果是列表详情路由，并且检测到是移动设备
    if (route.startsWith('list-item/') && isMobileDevice()) {
        // 提取列表ID
        const listId = route.split('/')[1];
        // 导航到移动端专用路由
        window.dispatchEvent(new CustomEvent('navigation', {
            detail: { route: `m-list-item/${listId}` }
        }));
    } else {
        // 桌面端或其他路由保持不变
        window.dispatchEvent(new CustomEvent('navigation', {
            detail: { route }
        }));
    }
}
</script>

<style>
.lists-view {
    height: 100%;
    padding: 16px;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--md-sys-color-background);
    z-index: 1000;
}

.mobile-header {
    margin-top: 8px;
}

.lists-container {
    margin-top: 16px;
}

.list-item-wrapper {
    position: relative;
    overflow: hidden;
}

.edit-buttons {
    position: absolute;
    right: -96px;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    gap: 8px;
    transition: right 0.3s ease;
    background: rgba(var(--v-theme-surface-variant), 0.8);
    padding: 8px;
    border-radius: 8px;
}

.edit-buttons.show {
    right: 8px;
}

/* 添加触摸提示 */
.list-hint {
    text-align: center;
    color: var(--v-theme-on-surface-variant);
    font-size: 0.875rem;
    margin-top: 8px;
    opacity: 0.7;
}

/* 移动端适配 */
@media (max-width: 768px) {
    .lists-view {
        padding: 8px;
    }

    .v-list-item {
        margin-bottom: 8px;
    }
}
</style>