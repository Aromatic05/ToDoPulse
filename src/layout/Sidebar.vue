<template>
    <v-navigation-drawer v-model="drawer" class="my-4 layout_navigation" :rail="railModel" permanent rail-width="77"
        style="position: fixed" @click="expandDrawer">
        <v-list class="py-4 mx-2 logo" nav>
            <v-list-item rounded class="mx-1" id="logo-item">
                <template v-slot:prepend>
                    <v-btn variant="text" :icon="railModel ? 'mdi-arrow-expand-right' : 'mdi-arrow-collapse-left'"
                        size="small" @click.stop="toggleRail" color="var(--md-sys-color-primary)"></v-btn>
                </template>
                <v-list-item-title class="title"
                    style="color: var(--md-sys-color-primary)">ToDoPulse</v-list-item-title>
                <v-list-item-subtitle style="color: var(--md-sys-color-on-surface-variant)">Task
                    Management</v-list-item-subtitle>
            </v-list-item>
        </v-list>
        <v-divider class="mx-5"></v-divider>

        <v-list nav class="mx-2">
            <!-- Timeline -->
            <v-list-item id="nav-timeline" prepend-icon="mdi-timeline" title="Timeline" class="mx-1"
                active-class="nav_active" rounded="lg" color="var(--md-sys-color-primary)"
                base-color="var(--md-sys-color-on-primary)" link @click="handleClick('timeline')" :ripple="true"
                :href="'#timeline'"> </v-list-item>

            <v-list-item id="nav-calendar" prepend-icon="mdi-calendar" title="Calendar" class="mx-1"
                active-class="nav_active" rounded="lg" color="var(--md-sys-color-primary)"
                base-color="var(--md-sys-color-on-primary)" link @click="handleClick('calendar')" :ripple="true"
                :href="'#calendar'"> </v-list-item>

            <v-list-item id="nav-tags" prepend-icon="mdi-tag-multiple" title="Tags" class="mx-1"
                active-class="nav_active" rounded="lg" color="var(--md-sys-color-primary)"
                base-color="var(--md-sys-color-on-primary)" link @click="handleClick('tags')" :ripple="true"
                :href="'#tags'"> </v-list-item>

            <v-list-group>
                <template v-slot:activator="{ props }">
                    <v-list-item id="nav-lists-header" v-bind="props" prepend-icon="mdi-format-list-bulleted"
                        title="Lists" rounded="lg" color="var(--md-sys-color-primary)" />
                </template>
                <template v-if="lists.length > 0">
                    <div v-for="(list, index) in lists" :key="index">
                        <v-list-item
                            :id="`nav-list-item-${list.id}`"
                            :prepend-icon="list.icon"
                            :title="list.title"
                            class="mx-1"
                            rounded="lg"
                            color="var(--md-sys-color-primary)"
                            @click.stop="handleClick(`list-item/${list.id}`)"
                            @contextmenu.prevent="showContextMenu(list, index, $event.currentTarget)"
                            link
                            :ripple="true"
                            :href="`#list-item-${list.id}`">
                        </v-list-item>
                    </div>
                    
                    <!-- 使用新组件 -->
                    <ListContextMenu
                        v-model:show="contextMenu.show"
                        :activator-element="contextMenu.activatorElement"
                        :target-list="contextMenu.targetList"
                        :target-index="contextMenu.targetIndex"
                        @rename="handleRename"
                        @delete="handleDelete"
                    />
                </template>
                <v-list-item v-else id="nav-lists-empty" prepend-icon="mdi-information-outline" title="没有可用的列表"
                    class="mx-1" rounded="lg" disabled></v-list-item>
                
                <!-- 添加列表按钮 -->
                <v-list-item 
                    id="nav-add-list" 
                    prepend-icon="mdi-plus-circle-outline" 
                    title="添加列表"
                    class="mx-1 mt-2" 
                    rounded="lg" 
                    color="var(--md-sys-color-primary)"
                    @click="showAddListModal"
                    link
                    :ripple="true">
                </v-list-item>
                
                <!-- 添加列表弹窗 -->
                <AddListModal 
                    v-model:show="addListModalVisible"
                    @create="handleAddList"
                />
            </v-list-group>
        </v-list>
    </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed, reactive, nextTick, onMounted } from 'vue';
import ListContextMenu from '@/components/ListContextMenu.vue';
import AddListModal from '@/components/Modals/AddListModal.vue';
import { getLists, renameList, deleteList } from '@/services/ListService';
import { FList } from 'src-tauri/bindings/FList';

const props = defineProps({
    rail: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits(['update:rail'])

const drawer = ref(true)
const railModel = computed({
    get: () => props.rail,
    set: (value) => {
        emit('update:rail', value)
    }
})

// 列表数据现在从服务中获取
const lists = ref<FList[]>([]);

// 在组件挂载时获取列表
onMounted(async () => {
    try {
        lists.value = await getLists();
    } catch (error) {
        console.error('获取列表失败:', error);
        // 可以添加错误处理逻辑，例如显示一个提示
    }
});

function toggleRail() {
    railModel.value = !railModel.value
}

function expandDrawer() {
    if (railModel.value) {
        railModel.value = false
    }
}

function handleClick(route: string) {
    console.log(`Navigating to ${route}`)
    window.dispatchEvent(new CustomEvent('navigation', {
        detail: { route }
    }))
}

const contextMenu = reactive({
    show: false,
    targetList: undefined as FList | undefined,
    targetIndex: -1,
    activatorElement: undefined as HTMLElement | undefined
});

async function showContextMenu(list: FList, index: number, element: HTMLElement) {
    if (contextMenu.show) {
        contextMenu.show = false;
        await nextTick();
    }
    contextMenu.targetList = list;
    contextMenu.targetIndex = index;
    contextMenu.activatorElement =  element;
    contextMenu.show = true;
}

async function handleRename(id: string, newName: string) {
    try {
        // 调用服务层重命名方法，并获取更新后的列表
        lists.value = await renameList(id, newName);
    } catch (error) {
        console.error('重命名列表失败:', error);
    }
}

async function handleDelete(id: string) {
    try {
        // 调用服务层删除方法，并获取更新后的列表
        lists.value = await deleteList(id);
    } catch (error) {
        console.error('删除列表失败:', error);
    }
}

const addListModalVisible = ref(false);

function showAddListModal() {
    addListModalVisible.value = true;
}

async function handleAddList() {
    lists.value = await getLists();
    // try {
    //     // 调用服务创建新列表
    //     // lists.value = await createList(title, icon);
    //     lists.value = await getLists();
    // } catch (error) {
    //     console.error('创建列表失败:', error);
    // }
}
</script>

<style scoped>
:deep(.v-list-item__prepend > .v-icon) {
    color: var(--md-sys-color-on-surface-variant);
}

:deep(.v-list-item--active .v-list-item__prepend > .v-icon) {
    color: var(--md-sys-color-on-primary);
}
</style>