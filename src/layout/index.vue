<template>
    <v-layout :class="{ 'mini_nav': rail }">
        <Sidebar @update:rail="(val: boolean) => rail = val" :rail="rail" />
        <main class="app_main">
            <Topbar @toggle-settings="toggleSettings" :current-view="currentView" />

            <!-- 视图内容 -->
            <div class="content-container content-left-aligned">
                <div class="view-content">
                    <component :is="currentViewComponent" :viewId="currentView" />
                </div>
            </div>
        </main>

        <!-- 设置面板 - 移到最外层以避免嵌套问题 -->
        <div v-if="showSettings" class="settings-container">
            <SettingsPanel @close="showSettings = false" />
        </div>
    </v-layout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import Sidebar from '@/layout/Sidebar.vue'
import Topbar from '@/layout/Topbar.vue'
import SettingsPanel from '@/components/Views/SettingsPanel.vue'

// 动态导入视图组件
import TimelineView from '@/components/Views/TimelineView.vue'
import CalendarView from '@/components/Views/CalendarView.vue'
import TagsView from '@/components/Views/TagsView.vue'
import ListView from '@/components/Views/ListView.vue'
import DefaultView from '@/components/Views/DefaultView.vue'

const rail = ref(false)
const showSettings = ref(false)
const currentView = ref('timeline') // 默认视图

// 根据当前视图名称动态加载组件
const currentViewComponent = computed(() => {
    switch (currentView.value) {
        case 'timeline':
            return TimelineView
        case 'calendar':
            return CalendarView
        case 'tags':
            return TagsView
        case 'settings':
            return SettingsPanel
        default:
            // 处理不同格式的列表路由
            if (currentView.value.startsWith('list/') || currentView.value.startsWith('list-item/')) {
                return ListView
            }
            return DefaultView
    }
})

function toggleSettings() {
    currentView.value = 'settings'
}

// 监听事件，接收来自 Sidebar 的导航通知
function handleNavigation(route: string) {
    currentView.value = route
    console.log(`视图已切换到: ${route}`)
}

onMounted(() => {
    // 监听全局事件总线的导航事件
    window.addEventListener('navigation', (e: any) => {
        if (e.detail && e.detail.route) {
            handleNavigation(e.detail.route)
        }
    })
})
</script>

<style>
@import "@/styles/layout.css";

/* 设置内容左对齐 */
.content-left-aligned {
    display: flex !important;
    flex-direction: column !important;
    align-items: flex-start !important;
    width: 100% !important;
    padding-left: 0 !important;
    margin-left: 0 !important;
}

/* 视图内容也需要完全宽度 */
.view-content {
    width: 100% !important;
    padding: 16px 16px 16px 0 !important;
    /* 减少左内边距 */
    min-height: 200px !important;
    margin-top: 86px !important;
    /* 70px(顶栏高度) + 16px(顶部间距) */
    margin-left: 0 !important;
    position: relative !important;
    z-index: 1 !important;
    display: flex !important;
    justify-content: flex-start !important;
}

/* 强制所有视图容器左对齐 */
.view-content>* {
    width: 100% !important;
    margin-left: 0 !important;
    padding-left: 0 !important;
}

/* 设置面板容器样式 - 完全覆盖在内容上方 */
.settings-container {
    position: fixed !important;
    top: 0 !important;
    left: 0 !important;
    right: 0 !important;
    bottom: 0 !important;
    padding-top: 86px !important;
    padding-left: 93px !important;
    padding-right: 16px !important;
    padding-bottom: 16px !important;
    z-index: 9999 !important;
    background-color: rgba(var(--md-sys-color-surface-rgb), 0.95) !important;
    backdrop-filter: blur(10px);
    animation: fadeIn 0.3s ease-in-out;
    display: flex !important;
    flex-direction: column !important;
    align-items: flex-start !important;
}

/* 当侧边栏展开时调整位置 */
.mini_nav .settings-container {
    padding-left: 272px !important;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(-10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>