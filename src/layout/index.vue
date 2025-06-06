<template>
    <v-layout :class="{ 'mini_nav': rail, 'mobile': isMobile }">
        <!-- 在非移动端显示侧边栏 -->
        <Sidebar v-if="!isMobile" @update:rail="(val: boolean) => rail = val" :rail="rail" />
        
        <main class="app_main">
            <Topbar @toggle-settings="toggleSettings" :current-view="currentView" />

            <!-- 视图内容 -->
            <div class="content-container content-left-aligned">
                <div class="view-content">
                    <keep-alive>
                        <component :is="currentViewComponent" :viewId="currentView" />
                    </keep-alive>
                </div>
            </div>
            
            <!-- 在移动端显示底部导航 -->
            <BottomNavigation v-if="isMobile" :current-view="currentView" />
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
import BottomNavigation from '@/layout/BottomNavigation.vue'

// 动态导入视图组件
import TimelineView from '@/components/Views/TimelineView.vue'
import CalendarView from '@/components/Views/CalendarView.vue'
import TagsView from '@/components/Views/TagsView.vue'
import ListView from '@/components/Views/ListView.vue'
import ListsView from '@/components/Views/ListsView.vue'
import DefaultView from '@/components/Views/DefaultView.vue'
// 导入移动端列表视图组件
import MListView from '@/components/Views/MListView.vue'

const rail = ref(false)
const showSettings = ref(false)
const currentView = ref('timeline') // 默认视图
const isMobile = ref(false)

// 检测是否为移动设备
const checkMobile = () => {
    isMobile.value = window.innerWidth <= 768
}

// 监听窗口大小变化
window.addEventListener('resize', checkMobile)

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
        case 'lists':
            return ListsView
        default:
            // 处理不同格式的列表路由
            if (currentView.value.startsWith('list/') || currentView.value.startsWith('list-item/')) {
                // 根据设备类型选择列表视图组件
                return isMobile.value ? MListView : ListView
            }
            // 专门处理移动端列表视图路由前缀
            if (currentView.value.startsWith('m-list-item/')) {
                return MListView
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
    // 初始化检查设备类型
    checkMobile()
    
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

/* 移动端样式调整 */
.mobile .app_main {
    margin-left: 0 !important;
    width: 100% !important;
}

.mobile .header {
    width: calc(100% - 16px) !important;
    left: 8px !important;
}

/* 移动端内容区域调整 */
.mobile .content-container {
    padding: 8px !important;
}

.mobile .view-content {
    margin-top: 78px !important;
    padding: 8px !important;
}

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