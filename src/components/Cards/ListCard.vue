<template>
    <div class="card-base">
        <div class="card-content-row">
            <!-- 状态列 - 添加checked绑定 -->
            <div class="card-column status-column">
                <input type="checkbox" class="card-checkbox" :checked="localData.finished" @change="handleComplete"/>
            </div>

            <!-- 任务列 -->
            <div class="card-column title-column">
                <h3 class="card-title" :class="{ 'completed-task': localData.finished }">
                    {{ localData.title }}
                </h3>
            </div>

            <!-- 优先级列 -->
            <div class="card-column priority-column">
                <div v-if="localData.tag?.length" class="card-tags">
                    <span v-for="(tag, i) in localData.tag" :key="i" class="card-tag">{{ tag }}</span>
                </div>
            </div>

            <!-- 截止日期列 -->
            <div class="card-column date-column">
                <span v-if="localData.date" class="card-date"
                    :style="{ color: localData.dateColor || 'var(--md-sys-color-on-surface-variant)' }">
                    {{ localData.date }}
                </span>
            </div>

            <!-- 操作列 -->
            <div class="card-column actions-column">
                <div class="card-actions">
                    <v-icon size="small" class="action-icon" @click.stop="handleEdit" title="编辑">mdi-pencil</v-icon>
                    <v-icon size="small" class="action-icon" @click.stop="handleDelete" title="删除">mdi-delete</v-icon>
                </div>
            </div>
        </div>
    </div>
    
    <!-- 添加确认删除对话框 -->
    <v-dialog v-model="deleteDialog" max-width="400px">
        <v-card>
            <v-card-title class="text-h5">确认删除</v-card-title>
            <v-card-text>
                确定要删除"{{ localData.title }}"任务吗？此操作无法撤销。
            </v-card-text>
            <v-card-actions>
                <v-spacer></v-spacer>
                <v-btn color="grey darken-1" text @click="deleteDialog = false">取消</v-btn>
                <v-btn color="error" variant="elevated" @click="confirmDelete">删除</v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>

    <CardContentModal v-model="showModal" :card-data="localData" @confirm="handleConfirm" v-if="showModal" />
</template>

<script lang="ts">
import CardContentModal from '@/components/Modals/CardContentModal.vue';
import { FEvent } from 'src-tauri/bindings/FEvent';

export default {
    name: 'ListCard',
    components: {
        CardContentModal
    },
    props: {
        data: {
            type: Object as () => FEvent,
            required: true,
            validator: (value: FEvent) => !!(value?.id && value?.title),
        }
    },
    emits: ['update', 'delete', 'toggleStatus'], // 声明自定义事件
    data() {
        return {
            showModal: false,
            deleteDialog: false,
            localData: {
                ...JSON.parse(JSON.stringify(this.data)),
                finished: this.data.finished || false
            }
        };
    },
    methods: {
        handleComplete() {
            this.localData.finished = !this.localData.finished;
            this.$emit('toggleStatus', {
                id: this.localData.id,
                finished: this.localData.finished
            });
        },
        handleConfirm(updatedData: FEvent) {
            console.log('Updated data:', updatedData);
            this.localData = { ...updatedData };
            this.$emit('update', updatedData); // 触发 update 事件
        },
        handleEdit() {
            this.showModal = true;
        },
        handleDelete() {
            this.deleteDialog = true;
        },
        confirmDelete() {
            this.$emit('delete', this.localData); // 触发 delete 事件
            this.deleteDialog = false;
        }
    }
};
</script>

<style scoped>
@import '@/styles/Cards/card.css';
@import '@/styles/Cards/listcard.css'
</style>