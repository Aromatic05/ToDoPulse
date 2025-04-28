<template>
    <div class="tags-view">
        <!-- <h1 class="text-h4 mb-6">标签管理</h1> -->

        <v-row>
            <v-col cols="12" md="8">
                <v-card class="pa-4">
                    <v-card-title>所有标签</v-card-title>
                    <v-card-text>
                        <v-chip-group>
                            <v-chip v-for="tag in tags" :key="tag.id" :color="tag.color" closable
                                @click:close="removeTag(tag.id)" class="ma-1">
                                {{ tag.name }}
                                <span class="ms-2 text-caption">({{ tag.count }})</span>
                            </v-chip>
                        </v-chip-group>
                    </v-card-text>
                </v-card>
            </v-col>

            <v-col cols="12" md="4">
                <v-card class="pa-4">
                    <v-card-title>添加新标签</v-card-title>
                    <v-card-text>
                        <v-form @submit.prevent="addTag">
                            <v-text-field v-model="newTag.name" label="标签名称" required class="mb-2"></v-text-field>

                            <v-select v-model="newTag.color" label="标签颜色" :items="availableColors"
                                class="mb-4"></v-select>

                            <v-btn color="primary" block type="submit" :disabled="!newTag.name">
                                创建标签
                            </v-btn>
                        </v-form>
                    </v-card-text>
                </v-card>
            </v-col>
        </v-row>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

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
</script>

<style scoped>
.tags-view {
    max-width: 1000px;
    margin: 0 auto;
}
</style>