<template>
    <v-dialog v-model="show" max-width="500px">
        <v-card>
            <v-card-title class="text-h5">添加新标签</v-card-title>
            <v-card-text>
                <v-form @submit.prevent="handleSubmit" class="d-flex flex-column gap-4">
                    <v-text-field
                        v-model="newTag.name"
                        label="标签名称"
                        required
                        :rules="[v => !!v || '标签名称不能为空']"
                    ></v-text-field>

                    <v-select
                        v-model="newTag.color"
                        label="标签颜色"
                        :items="availableColors"
                        item-title="value"
                        item-value="value"
                    >
                        <template v-slot:selection="{ item }">
                            <div class="d-flex align-center">
                                <div class="color-square mr-2"
                                    :style="{ backgroundColor: getColorValue(item.value) }"></div>
                                <span :style="{ color: getColorValue(item.value) }">{{ item.value }}</span>
                            </div>
                        </template>
                    </v-select>

                    <div>
                        <label class="text-subtitle-2 mb-2 d-block">标签预览</label>
                        <v-chip :color="newTag.color?.toLowerCase()" class="ma-1" variant="tonal">
                            {{ newTag.name || '标签预览' }}
                        </v-chip>
                    </div>
                </v-form>
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="error" text @click="closeDialog">取消</v-btn>
                <v-btn color="primary" @click="handleSubmit" :disabled="!newTag.name">确定</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useTagStore } from '@/stores';
import { TagColor } from 'src-tauri/bindings/TagColor';

interface ColorOption {
    value: TagColor;
}

const props = defineProps({
    modelValue: {
        type: Boolean,
        default: false
    },
});

const emit = defineEmits(['update:modelValue', 'created']);

const tagStore = useTagStore();

const show = ref(props.modelValue);

const newTag = ref({
    name: '',
    color: 'Primary' as TagColor
});

// 颜色选项
const availableColors: ColorOption[] = [
    { value: 'Primary' as TagColor },
    { value: 'Secondary' as TagColor },
    { value: 'Success' as TagColor },
    { value: 'Info' as TagColor },
    { value: 'Warning' as TagColor },
    { value: 'Error' as TagColor }
];

function getColorValue(color: TagColor | string): string {
    return `var(--v-theme-${color.toLowerCase()})`;
}

async function handleSubmit() {
    if (!newTag.value.name) return;

    try {
        await tagStore.addTag(newTag.value.name, newTag.value.color);
        emit('created', newTag.value.name);
        closeDialog();
    } catch (error) {
        console.error('添加标签失败:', error);
    }
}

function closeDialog() {
    show.value = false;
    newTag.value = {
        name: '',
        color: 'Primary' as TagColor
    };
}

watch(() => props.modelValue, (val) => {
    show.value = val;
});

watch(show, (val) => {
    emit('update:modelValue', val);
});
</script>

<style scoped>
.color-square {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    margin-right: 8px;
}

.gap-4 {
    gap: 16px;
}
</style>