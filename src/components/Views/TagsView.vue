<template>
    <div class="tags-view">
        <!-- <h1 class="text-h4 mb-6">标签管理</h1> -->

        <v-row class="fill-height">
            <v-col cols="12" md="8" class="d-flex">
                <v-card class="pa-4 flex-grow-1">
                    <v-card-title>所有标签</v-card-title>
                    <v-card-text>
                        <div class="d-flex flex-wrap">
                            <v-chip v-for="tag in tags" :key="tag.id" :color="tag.color" closable
                                @click:close="removeTag(tag.id)" @click="openTagModal(tag)" class="ma-1" variant="tonal">
                                {{ tag.name }}
                                <span class="ms-2 text-caption">({{ tag.count }})</span>
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

                            <v-select v-model="newTag.color" label="标签颜色" :items="availableColors" item-title="text"
                                item-value="value" class="mb-4">
                                <template v-slot:selection="{ item }">
                                    <div class="d-flex align-center">
                                        <div class="color-square mr-2"
                                            :style="{ backgroundColor: getColorValue(item.value) }"></div>
                                        {{ item.value }}
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
        <TagModal v-model="showTagModal" :tag="selectedTag" @save="updateTag" />
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import TagModal from '@/components/Modals/TagModal.vue';

const tags = ref([
    { id: 1, name: '工作', color: 'primary', count: 5 },
    { id: 2, name: '个人', color: 'secondary', count: 3 },
    { id: 3, name: '紧急', color: 'error', count: 2 },
    { id: 4, name: '学习', color: 'info', count: 4 },
    { id: 5, name: '家庭', color: 'success', count: 1 }
])

const newTag = ref({
    name: '',
    color: 'primary'
})

const availableColors = [
    'primary',
    'secondary',
    'success',
    'info',
    'warning',
    'error'
]

// Add new refs for the modal
const showTagModal = ref(false);
const selectedTag = ref({ id: 0, name: '', color: 'primary', count: 0 });

function getColorValue(color: string): string {
    // 这里可以根据需要返回实际的颜色值
    // 为了简单起见，我们直接返回Vuetify的颜色变量名
    return `var(--v-theme-${color})`;
}

function removeTag(id: number) {
    const index = tags.value.findIndex(tag => tag.id === id)
    if (index !== -1) {
        tags.value.splice(index, 1)
    }
}

function addTag() {
    if (newTag.value.name) {
        const id = Math.max(0, ...tags.value.map(t => t.id)) + 1
        tags.value.push({
            id,
            name: newTag.value.name,
            color: newTag.value.color,
            count: 0
        })
        newTag.value.name = ''
    }
}

// Function to open the tag modal
function openTagModal(tag: { id: number; name: string; color: string; count: number; } | { id: number; name: string; color: string; count: number; }) {
    selectedTag.value = { ...tag };
    showTagModal.value = true;
}

// Function to update the tag when saved
function updateTag(updatedTag: { id: number; name: string; color: string; count: number; }) {
    const index = tags.value.findIndex(tag => tag.id === updatedTag.id);
    if (index !== -1) {
        tags.value[index] = updatedTag;
    }
}
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