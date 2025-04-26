<template>
    <div class="card-base table-card">
        <div class="card-content-row">
            <!-- 状态列 - 修改点击事件处理 -->
            <div class="card-column status-column">
                <input type="checkbox" class="card-checkbox" @change="handleCheckboxChange"/>
            </div>

            <!-- 任务列 -->
            <div class="card-column title-column">
                <h3 class="card-title" :class="{ 'completed-task': localData.isCompleted }">
                    {{ localData.title }}
                </h3>
            </div>

            <!-- 优先级列 -->
            <div class="card-column priority-column">
                <div v-if="localData.tags?.length" class="card-tags">
                    <span v-for="(tag, i) in localData.tags" :key="i" class="card-tag">{{ tag }}</span>
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

    <CardContentModal v-model="showModal" :card-data="localData" @confirm="handleConfirm" />
</template>

<script>
import CardContentModal from '@/components/Modals/CardContentModal.vue'
import { h } from 'vue';

export default {
    name: 'ListCard',
    components: {
        CardContentModal
    },
    // 添加新的emit事件类型
    emits: ['update', 'delete', 'toggleStatus'],
    props: {
        data: {
            type: Object,
            required: true,
            validator: (value) => {
                return value?.id && 
                       value?.title !== undefined && 
                       value?.content !== undefined; // 允许空字符串
            }
        }
    },
    data() {
        return {
            showModal: false,
            deleteDialog: false,
            // 删除 isUpdatingFromWatch 标志，不再需要
            // 使用深拷贝防止引用问题
            localData: {
                ...JSON.parse(JSON.stringify(this.data)),
                isCompleted: this.data.isCompleted || false
            }
        }
    },
    methods: {
        handleCheckboxChange() {
            // 直接切换状态
            this.localData.isCompleted = !this.localData.isCompleted;
            this.$emit('toggleStatus', {
                id: this.localData.id,
                listId: this.localData.listId,
                isCompleted: this.localData.isCompleted
            });
            
            console.log('任务状态已切换:', this.localData.title, this.localData.isCompleted);
        },
        handleConfirm(updatedData) {
            console.log('EventCard updating:', updatedData);
            this.localData = { ...updatedData };  // 更新本地数据
            this.$emit('update', updatedData);
        },
        handleEdit() {
            this.showModal = true;
        },
        handleDelete() {
            this.deleteDialog = true;
        },
        confirmDelete() {
            this.$emit('delete', this.localData);
            this.deleteDialog = false;
        }
    }
    // 删除所有watch逻辑
}
</script>

<style scoped>
@import '@/styles/card.css';

/* 表格中卡片的特殊样式 */
.table-card {
    width: 100%;
    padding: 8px 0;
    border-radius: 8px;
    transition: background-color 0.2s;
    box-sizing: border-box;
    overflow: hidden;
}

.table-card:hover {
    background-color: rgba(0, 0, 0, 0.03);
}

.card-content-row {
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 36px;
}

/* 列样式 */
.card-column {
    padding: 0 8px;
    display: flex;
    align-items: center;
}

.status-column {
    flex: 0 0 80px;
    /* 对应状态列宽度 */
    justify-content: center;
}

.title-column {
    flex: 1 1 auto;
    /* 任务列可伸缩 */
    min-width: 200px;
    overflow: hidden;
}

.priority-column {
    flex: 0 0 120px;
    /* 对应优先级列宽度 */
    justify-content: center;
}

.date-column {
    flex: 0 0 150px;
    /* 对应截止日期列宽度 */
    justify-content: center;
}

.actions-column {
    flex: 0 0 100px;
    /* 对应操作列宽度 */
    justify-content: center;
}

/* 其他样式保持不变 */
.card-title {
    margin: 0;
    font-size: 1rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.87);
    width: 100%;
}

.completed-task {
    text-decoration: line-through;
    opacity: 0.7;
}

.card-tags {
    display: flex;
    gap: 6px;
    justify-content: center;
    width: 100%;
}

.card-tag {
    padding: 3px 8px;
    border-radius: 4px;
    background-color: #f0f0f0;
    font-size: 0.85rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
}

.card-date {
    white-space: nowrap;
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
}

.card-actions {
    display: flex;
    gap: 8px;
}

.action-icon {
    color: rgba(0, 0, 0, 0.6);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s;
}

.action-icon:hover {
    background-color: rgba(0, 0, 0, 0.05);
    color: rgba(0, 0, 0, 0.87);
}
</style>