<template>
    <!-- 事件上下文菜单 -->
    <v-menu
        v-model="show"
        :activator="actualActivator as unknown as Element"
        location="end"
        :close-on-content-click="true"
        :close-on-back="true"
        :open-on-click="false"
    >
        <v-list density="compact" bg-color="var(--md-sys-color-surface-container-high)">
            <v-list-item
                @click="toggleCompletion"
                prepend-icon="mdi-check-circle"
                :title="targetEvent?.extendedProps?.finished ? '标记为未完成' : '标记为已完成'"
            >
            </v-list-item>
            <v-list-item
                @click="triggerDelete"
                prepend-icon="mdi-delete"
                title="删除事件"
                class="text-error"
            >
            </v-list-item>
        </v-list>
    </v-menu>

    <!-- 删除确认对话框 -->
    <v-dialog v-model="deleteDialog.show" max-width="500px">
        <v-card>
            <v-card-title>删除事件</v-card-title>
            <v-card-text>
                <p>确定要删除事件 "{{ deleteDialog.eventTitle }}" 吗？</p>
                <p class="text-caption mt-2">此操作无法撤销。</p>
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
import { reactive, computed, type Ref } from 'vue';
import type { EventApi } from '@fullcalendar/core';

const props = defineProps({
    show: {
        type: Boolean,
        default: false
    },
    activatorElement: {
        type: [Object, null] as unknown as () => HTMLElement | null | Ref<HTMLElement | null>,
        default: null
    },
    targetEvent: {
        type: Object as () => EventApi | null,
        default: null
    }
});

// 获取正确的activator元素
const actualActivator = computed(() => {
    // 如果是ref对象，则获取其value
    if (props.activatorElement && 'value' in props.activatorElement) {
        return props.activatorElement.value;
    }
    return props.activatorElement;
});

const emit = defineEmits(['update:show', 'toggle-completion', 'delete']);

// 菜单显示状态
const show = computed({
    get: () => props.show,
    set: (value) => {
        emit('update:show', value);
    }
});

// 删除确认对话框
const deleteDialog = reactive({
    show: false,
    eventTitle: '',
    targetEvent: null as EventApi | null
});

function toggleCompletion() {
    if (props.targetEvent) {
        emit('toggle-completion', props.targetEvent);
        show.value = false;
    }
}

function triggerDelete() {
    if (props.targetEvent) {
        deleteDialog.targetEvent = props.targetEvent;
        deleteDialog.eventTitle = props.targetEvent.title;
        deleteDialog.show = true;
        show.value = false;
    }
}

function confirmDelete() {
    if (deleteDialog.targetEvent) {
        emit('delete', deleteDialog.targetEvent);
        deleteDialog.show = false;
        deleteDialog.targetEvent = null;
    }
}
</script>