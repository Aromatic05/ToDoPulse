<template>
    <v-bottom-navigation v-model="activeTab" class="mobile-bottom-nav" color="primary" grow>
        <v-btn value="timeline" @click="navigate('timeline')">
            <v-icon>mdi-timeline-outline</v-icon>
            <span>时间线</span>
        </v-btn>

        <v-btn value="calendar" @click="navigate('calendar')">
            <v-icon>mdi-calendar</v-icon>
            <span>日历</span>
        </v-btn>

        <v-btn value="tags" @click="navigate('tags')">
            <v-icon>mdi-tag-multiple-outline</v-icon>
            <span>标签</span>
        </v-btn>

        <v-btn value="lists" @click="navigate('lists')">
            <v-icon>mdi-format-list-bulleted</v-icon>
            <span>列表</span>
        </v-btn>
    </v-bottom-navigation>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps({
    currentView: {
        type: String,
        required: true
    }
});

const activeTab = ref(props.currentView);

// 监听当前视图的变化
watch(() => props.currentView, (newView) => {
    activeTab.value = newView;
});

// 导航到指定路由
function navigate(route: string) {
    // 触发导航事件
    window.dispatchEvent(new CustomEvent('navigation', {
        detail: { route }
    }));
}
</script>

<style>
.mobile-bottom-nav {
    position: fixed !important; /* 添加 !important 确保优先级 */
    bottom: 0 !important;
    left: 0 !important;
    right: 0 !important;
    z-index: 1000 !important; /* 增加一个高优先级的 z-index */
    display: flex;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.15);
    border-top: 1px solid var(--md-sys-color-outline-variant);
    background-color: var(--md-sys-color-surface-container-high) !important;
    padding-bottom: env(safe-area-inset-bottom);
    /* 适配iPhone X及以上机型 */
    height: 64px; /* 确保高度固定 */
    transition: all 0.3s ease; /* 平滑过渡效果 */
}

/* 修改媒体查询，移除冗余的 display: flex */
@media (min-width: 769px) {
    /* 大屏设备上的样式调整（如果需要） */
    .mobile-bottom-nav {
        /* 如果需要在大屏上隐藏，则取消注释下面这行 */
        display: none;
    }
}

@media (max-width: 768px) {
    /* 为底部导航腾出空间 */
    .app_main .content-container {
        padding-bottom: 80px !important;
    }
    
    /* 增加额外的安全距离，防止内容被底部导航覆盖 */
    .view-content {
        margin-bottom: 64px !important;
    }
}

/* 优化触摸体验 */
.mobile-bottom-nav .v-btn {
    min-height: 56px;
    border-radius: 0;
    transition: all 0.2s ease; /* 按钮过渡效果 */
    position: relative;
    overflow: hidden;
}

/* 高亮效果 */
.mobile-bottom-nav .v-btn::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%) scaleX(0);
    width: 24px;
    height: 3px;
    background-color: var(--md-sys-color-primary);
    border-radius: 3px 3px 0 0;
    transition: transform 0.2s ease;
}

/* 激活状态样式 */
.mobile-bottom-nav .v-btn--active {
    color: var(--md-sys-color-primary);
}

.mobile-bottom-nav .v-btn--active::after {
    transform: translateX(-50%) scaleX(1);
}

/* 图标和文字样式优化 */
.mobile-bottom-nav .v-btn .v-icon {
    margin-bottom: 4px;
    transition: transform 0.2s ease;
}

.mobile-bottom-nav .v-btn--active .v-icon {
    transform: scale(1.1);
}

.mobile-bottom-nav .v-btn span {
    font-size: 12px;
    font-weight: 500;
}
</style>