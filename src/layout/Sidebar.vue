<template>
    <v-navigation-drawer v-model="drawer" class="my-4 layout_navigation" :rail="railModel" permanent rail-width="77"
        style="position: fixed" @click="expandDrawer">
        <v-list class="py-4 mx-2 logo" nav>
            <v-list-item rounded class="mx-1">
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
            <v-list-item prepend-icon="mdi-timeline" title="Timeline" class="mx-1" active-class="nav_active"
                rounded="lg" color="var(--md-sys-color-primary)" active-color="var(--md-sys-color-on-primary)" link
                @click="handleClick('timeline')" :ripple="true" :href="'#'"></v-list-item>

            <!-- Calendar -->
            <v-list-item prepend-icon="mdi-calendar" title="Calendar" class="mx-1" active-class="nav_active"
                rounded="lg" color="var(--md-sys-color-primary)" active-color="var(--md-sys-color-on-primary)" link
                @click="handleClick('calendar')" :ripple="true" :href="'#'"></v-list-item>

            <!-- Tags -->
            <v-list-item prepend-icon="mdi-tag-multiple" title="Tags" class="mx-1" active-class="nav_active"
                rounded="lg" color="var(--md-sys-color-primary)" active-color="var(--md-sys-color-on-primary)" link
                @click="handleClick('tags')" :ripple="true" :href="'#'"></v-list-item>

            <!-- Lists (as dropdown group) -->
            <v-list-group>
                <template v-slot:activator="{ props }">
                    <v-list-item v-bind="props" prepend-icon="mdi-format-list-bulleted" title="Lists" rounded="lg"
                        color="var(--md-sys-color-primary)" />
                </template>
                <!-- 这里的列表项将从其他地方获取，暂时为空 -->
                <template v-if="lists.length > 0">
                    <v-list-item v-for="(list, index) in lists" :key="index" :prepend-icon="list.icon"
                        :title="list.title" class="mx-1" rounded="lg" color="var(--md-sys-color-primary)"
                        @click="handleClick(`list/${list.id}`)" link :ripple="true" :href="'#'"></v-list-item>
                </template>
                <v-list-item v-else prepend-icon="mdi-information-outline" title="没有可用的列表" class="mx-1" rounded="lg"
                    disabled></v-list-item>
            </v-list-group>
        </v-list>
    </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

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

// 这里是接口，从外部获取列表数据
const lists = ref([
    // 示例数据，实际使用时可以通过API获取
    { id: '1', title: '工作', icon: 'mdi-briefcase' },
    { id: '2', title: '个人', icon: 'mdi-account' }
])

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
    // 触发自定义事件以通知 index.vue
    window.dispatchEvent(new CustomEvent('navigation', {
        detail: { route }
    }))
}
</script>

<style scoped>
/* 确保图标颜色正确应用 */
:deep(.v-list-item__prepend > .v-icon) {
    color: var(--md-sys-color-on-surface-variant);
}

:deep(.v-list-item--active .v-list-item__prepend > .v-icon) {
    color: var(--md-sys-color-on-primary);
}
</style>
