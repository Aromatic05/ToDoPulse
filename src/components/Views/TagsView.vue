<template>
    <div class="tags-view">
        <!-- <h1 class="text-h4 mb-6">标签管理</h1> -->

        <v-row class="fill-height">
            <v-col cols="12" md="8" class="d-flex">
                <v-card class="pa-4 flex-grow-1">
                    <v-card-title class="d-flex align-center">
                        所有标签
                        <v-spacer></v-spacer>
                        <v-btn v-if="!isLoading" icon="mdi-refresh" size="small" @click="fetchTags"
                            variant="text"></v-btn>
                        <v-progress-circular v-else size="24" indeterminate color="primary"></v-progress-circular>
                    </v-card-title>

                    <v-card-text>
                        <v-alert v-if="error" type="error" class="mb-4">{{ error }}</v-alert>

                        <div v-if="tags.length === 0 && !isLoading" class="text-center pa-4">
                            <p>暂无标签</p>
                        </div>

                        <div v-else class="d-flex flex-wrap">
                            <v-chip v-for="tag in tags" :key="tag.id" :color="mapTagColor(tag.color)"
                                @click="openTagModal(tag)"
                                @contextmenu.prevent="showContextMenu(convertTagForUI(tag), $event.currentTarget)"
                                class="ma-1" variant="tonal">
                                {{ tag.name }}
                                <span class="ms-2 text-caption">({{ getTagEventCount(tag.name) }})</span>
                            </v-chip>
                        </div>
                    </v-card-text>
                </v-card>
            </v-col>

            <v-col cols="12" md="4" class="d-flex">
                <v-card class="pa-4 flex-grow-1">
                    <v-card-title>添加新标签</v-card-title>
                    <v-card-text class="flex-grow-1 d-flex flex-column">
                        <v-form @submit.prevent="addTag" class="flex-grow-1 d-flex flex-column">
                            <v-text-field v-model="newTag.name" label="标签名称" required class="mb-2"></v-text-field>

                            <v-select v-model="newTag.color" label="标签颜色" :items="availableColors" item-title="value"
                                item-value="value" class="mb-4">
                                <template v-slot:selection="{ item }">
                                    <div class="d-flex align-center">
                                        <div class="color-square mr-2"
                                            :style="{ backgroundColor: getColorValue(item.value) }"></div>
                                        <span :style="{ color: getColorValue(item.value) }">{{ item.value }}</span>
                                    </div>
                                </template>
                            </v-select>

                            <div class="mb-4">
                                <label class="text-subtitle-2 mb-2 d-block">标签预览</label>
                                <v-chip :color="newTag.color" class="ma-1" variant="tonal">
                                    {{ newTag.name || '标签预览' }}
                                </v-chip>
                            </div>

                            <v-spacer></v-spacer>

                            <v-btn color="primary" block type="submit" :disabled="!newTag.name">
                                创建标签
                            </v-btn>
                        </v-form>
                    </v-card-text>
                </v-card>
            </v-col>
        </v-row>

        <!-- Add the TagModal component -->
        <TagModal v-model="showTagModal" :tag="selectedTag" @tag-removed="handleTagRemoved" />

        <!-- Add the TagContextMenu component -->
        <TagContextMenu v-model:show="contextMenu.show" :activator-element="contextMenu.activatorElement"
            :target-tag="contextMenu.targetTag" @rename="handleRename" @delete="handleDelete" />
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick, reactive, computed, onMounted } from 'vue'
import { useTagStore } from '@/stores/index'
import TagModal from '@/components/Modals/TagModal.vue';
import TagContextMenu from '@/components/Menus/TagContextMenu.vue';
import { TagColor } from 'src-tauri/bindings/TagColor';
import { Tag } from '@/services/TagService';

// UI中使用的标签类型
interface UITag {
    id: number;
    name: string;
    color: string;
    count: number;
}

// 获取tagStore
const tagStore = useTagStore();

// 从store获取状态
const tags = computed(() => tagStore.tags);
const isLoading = computed(() => tagStore.isLoading);
const error = computed(() => tagStore.error);

const newTag = ref({
    name: '',
    color: 'Primary' as TagColor
})

// 颜色选项，与TagColor枚举对应
const availableColors = [
    { value: 'Primary' as TagColor },
    { value: 'Secondary' as TagColor },
    { value: 'Success' as TagColor },
    { value: 'Info' as TagColor },
    { value: 'Warning' as TagColor },
    { value: 'Error' as TagColor }
]

// 模态框状态
const showTagModal = ref(false);
const selectedTag = ref<UITag>({ id: 0, name: '', color: 'Primary', count: 0 });

function getColorValue(color: string): string {
    // 返回颜色变量
    return `var(--v-theme-${color.toLowerCase()})`;
}

// 将TagColor映射为Vuetify颜色
function mapTagColor(color: TagColor): string {
    // Vuetify使用小写颜色名，而TagColor枚举是大写开头
    return color.toLowerCase();
}

// 将后端Tag转换为UI使用的Tag
function convertTagForUI(tag: Tag): UITag {
    return {
        id: typeof tag.id === 'number' ? tag.id : parseInt(String(tag.id)),
        name: tag.name,
        color: mapTagColor(tag.color),
        count: getTagEventCount(tag.name)
    };
}

// 获取标签下的事件数量
function getTagEventCount(tagName: string): number {
    return tagStore.getEventsByTagName(tagName).length;
}

// 加载所有标签
async function fetchTags() {
    await tagStore.fetchTags();
}

// 添加新标签
async function addTag() {
    if (newTag.value.name) {
        await tagStore.addTag(newTag.value.name, newTag.value.color);
        newTag.value.name = '';
        newTag.value.color = 'Primary';
    }
}

// 打开标签模态框
async function openTagModal(tag: Tag) {
    // 先加载标签内容
    await tagStore.getTagContent(tag.name);

    // 设置选中的标签
    selectedTag.value = convertTagForUI(tag);
    showTagModal.value = true;
}

// 不再使用此函数，因为TagModal现在直接使用tagStore获取数据
// function updateTag(updatedTag: UITag) {
//     const index = tags.value.findIndex(tag => tag.id === updatedTag.id);
//     if (index !== -1) {
//         tags.value[index] = updatedTag;
//     }
// }

// 不再需要此函数，因为TagModal现在直接使用tagStore获取数据
// function updateTag(updatedTag: { id: number; name: string; color: string; count: number; }) {
//     const index = tags.value.findIndex(tag => tag.id === updatedTag.id);
//     if (index !== -1) {
//         tags.value[index] = updatedTag;
//     }
// }

// Context menu state
const contextMenu = reactive({
    show: false,
    targetTag: undefined as UITag | undefined,
    activatorElement: undefined as HTMLElement | undefined
});

// Function to show context menu
async function showContextMenu(tag: UITag, element: HTMLElement) {
    if (contextMenu.show) {
        contextMenu.show = false;
        await nextTick();
    }
    contextMenu.targetTag = { ...tag };
    contextMenu.activatorElement = element;
    contextMenu.show = true;
}

// 处理从上下文菜单发起的重命名
function handleRename(oldName: string, newName: string) {
    renameTag(oldName, newName);
}

// 处理从上下文菜单发起的删除
function handleDelete(name: string) {
    deleteTag(name);
}

// 处理标签重命名
// 注意：后端没有直接提供重命名API，需要删除旧标签并创建新标签
async function renameTag(oldName: string, newName: string) {
    if (oldName === newName) return;

    try {
        // 获取原标签的颜色
        const oldTag = tagStore.getTagByName(oldName);
        if (!oldTag) return;

        // 创建新标签
        await tagStore.addTag(newName, oldTag.color);

        // 删除旧标签
        await tagStore.deleteTag(oldName);
    } catch (err) {
        console.error('重命名标签失败:', err);
    }
}

// 处理标签删除
async function deleteTag(name: string) {
    try {
        await tagStore.deleteTag(name);
    } catch (err) {
        console.error('删除标签失败:', err);
    }
}

// 处理标签从事件中移除
async function handleTagRemoved(tagName: string) {
    // 清除缓存并刷新数据
    tagStore.clearCache();

    // 重新获取所有标签
    await fetchTags();

    // 重新加载当前选中标签的内容（如果需要）
    if (selectedTag.value && selectedTag.value.name === tagName) {
        await tagStore.getTagContent(tagName);
    }
}

// 组件挂载时加载标签
onMounted(() => {
    fetchTags();
});
</script>

<style scoped>
.tags-view {
    max-width: 1000px;
    margin: 0 auto;
    height: calc(100vh - 64px);
    /* 减去顶部导航栏的高度，根据实际情况调整 */
    display: flex;
    flex-direction: column;
}

.color-square {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    margin-right: 8px;
}

.gap-2 {
    gap: 8px;
}
</style>