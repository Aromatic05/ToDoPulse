<template>
    <div class="header_title pl-2">
        <v-breadcrumbs :items="items" class="pa-0" density="compact">
            <template v-slot:prepend>
                <v-icon icon="mdi-home-outline" size="small" class="home-icon"></v-icon>
            </template>
            <template v-slot:divider>
                <v-icon icon="mdi-chevron-right" size="small"></v-icon>
            </template>
            <template v-slot:title="{ item }">
                <span :class="{ 'font-weight-medium': item.disabled }">{{ item.title }}</span>
            </template>
        </v-breadcrumbs>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useListStore } from '@/stores';
import { FList } from 'src-tauri/bindings/FList';

// 初始化listStore
const listStore = useListStore();

// 当前列表数据
const currentList = ref<FList | null>(null);
const lists = ref<any[]>([]);

// 初始化获取列表数据
const initLists = async () => {
    try {
        // 使用listStore获取列表数据
        lists.value = await listStore.fetchLists();
        // 数据加载后立即检查当前URL
        checkCurrentRoute();
    } catch (error) {
        console.error('Breadcrumbs获取列表失败:', error);
    }
};

// 检查当前路由
function checkCurrentRoute() {
    // 从URL中获取当前路径
    const hash = window.location.hash.replace('#', '');
    
    // 处理列表路由
    if (hash.startsWith('list/')) {
        const listId = hash.replace('list/', '');
        updateListById(listId);
    } else if (hash.includes('list-')) {
        // 兼容旧格式 - 处理list-xxx、list-view-xxx和list-item-xxx三种格式
        let listId = hash;
        if (hash.startsWith('list-view-')) {
            listId = hash.replace('list-view-', '');
        } else if (hash.startsWith('list-item-')) {
            listId = hash.replace('list-item-', '');
        } else {
            listId = hash.replace('list-', '');
        }
        updateListById(listId);
    } else if (hash === 'timeline') {
        currentList.value = null;
    }
}

// 添加事件监听
const updateFromEvent = (event: CustomEvent) => {
    const route = event.detail.route;
    
    if (route.startsWith('list/')) {
        const listId = route.replace('list/', '');
        updateListById(listId);
    } else if (route.startsWith('list-item/')) {
        const listId = route.replace('list-item/', '');
        updateListById(listId);
    } else if (route === 'timeline') {
        currentList.value = null; // 重置当前列表
    }
};

// 根据ID更新当前列表
function updateListById(listId: string) {
    const list = lists.value.find(item => item.id === listId);
    if (list) {
        currentList.value = { id: list.id, title: list.title, icon: list.icon };
    } else {
        console.warn('未找到ID对应的列表:', listId);
    }
}

// 初始化和监听
onMounted(() => {
    // 先添加事件监听，确保不会错过事件
    window.addEventListener('navigation', updateFromEvent as EventListener);
    
    // 然后初始化数据
    initLists();
});

// 清理事件监听
onUnmounted(() => {
    window.removeEventListener('navigation', updateFromEvent as EventListener);
});

// 面包屑路径计算属性
const breadcrumbPath = computed(() => {
    if (currentList.value) {
        return [
            { title: 'Lists', disabled: false },
            { title: currentList.value.title, disabled: true }
        ];
    }

    // 默认路径
    return [{ title: 'Lists', disabled: true }];
});

// 面包屑项目（用于展示）
const items = computed(() => {
    return breadcrumbPath.value.map(item => ({
        title: item.title,
        disabled: item.disabled,
    }));
});
</script>

<style scoped>
.header_title {
    display: flex;
    flex-direction: column;
    justify-content: center;
}

.page_title {
    margin-top: 4px;
    font-size: 16px;
    font-weight: 700;
}

/* 减少Home图标前的空白 */
:deep(.v-breadcrumbs) {
    padding-left: 0;
}

.home-icon {
    margin-left: -4px; /* 负margin减少左侧空白 */
}

/* 避免图标与文字间距过大 */
:deep(.v-breadcrumbs-item) {
    padding: 0 8px;
    color: var(--md-sys-color-on-surface);
}

/* 当前项目（disabled=true的项）颜色设置 */
:deep(.v-breadcrumbs-item--disabled) {
    opacity: 0.9; /* 提高不透明度 */
    color: var(--md-sys-color-on-surface);
    font-weight: 500; /* 稍微加粗 */
}

:deep(.v-breadcrumbs-divider) {
    padding: 0 4px;
    color: var(--md-sys-color-on-surface-variant);
}

:deep(.v-icon) {
    opacity: 0.9;
    color: var(--md-sys-color-on-surface);
}
</style>