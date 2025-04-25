<template>
    <div class="default-view">
        <v-container>
            <v-row>
                <v-col cols="12" class="text-center">
                    <h1 class="text-h3 mb-6">欢迎使用 ToDoPulse</h1>
                    <p class="text-body-1">请从左侧导航栏选择一个视图开始</p>
                </v-col>
            </v-row>

            <v-row class="mt-8">
                <v-col v-for="(item, i) in features" :key="i" cols="12" md="4">
                    <v-card class="pa-4 h-100">
                        <v-card-title class="d-flex align-center">
                            <v-icon :color="item.color" size="large" class="me-2">{{ item.icon }}</v-icon>
                            {{ item.title }}
                        </v-card-title>
                        <v-card-text>{{ item.description }}</v-card-text>
                    </v-card>
                </v-col>
            </v-row>
        </v-container>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const features = ref([
    {
        title: '时间线视图',
        icon: 'mdi-timeline',
        color: 'primary',
        description: '以时间线形式查看和管理您的任务，直观地了解工作流程和进度。'
    },
    {
        title: '日历视图',
        icon: 'mdi-calendar',
        color: 'secondary',
        description: '在日历中安排和查看任务，帮助您更好地规划时间和管理截止日期。'
    },
    {
        title: '标签管理',
        icon: 'mdi-tag-multiple',
        color: 'success',
        description: '使用标签分类和组织您的任务，让任务管理更加灵活高效。'
    }
])
</script>

<style scoped>
.default-view {
    max-width: 1200px;
    margin: 0 auto;
    padding: 24px;
}
</style>