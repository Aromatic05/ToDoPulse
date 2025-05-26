<template>
    <Teleport to="body">
        <div v-if="modelValue" class="modal-mask" @click.self="handleClose">
            <div class="modal-container">
                <div class="modal-header">
                    <h2>新建卡片</h2>
                </div>
                <div class="modal-body">
                    <div class="modal-layout">
                        <div class="form-section">
                            <div class="form-group">
                                <v-text-field 
                                    clearable 
                                    label="标题" 
                                    v-model="formData.title"
                                    variant="outlined" 
                                    density="compact"
                                    required
                                ></v-text-field>
                            </div>

                            <div class="form-group">
                                <v-select
                                    v-model="selectedList"
                                    :items="lists"
                                    item-title="title"
                                    item-value="id"
                                    label="所属列表"
                                    variant="outlined"
                                    density="compact"
                                    required
                                    :menu-props="{ maxHeight: 400 }"
                                ></v-select>
                            </div>

                            <div class="form-group">
                                <v-select 
                                    v-model="formData.priority" 
                                    :items="['Low', 'Medium', 'High', 'Undefined']"
                                    label="优先级" 
                                    variant="outlined"
                                ></v-select>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button @click="handleClose">取消</button>
                    <button 
                        type="button" 
                        @click="handleConfirm" 
                        class="confirm-btn"
                        :disabled="!selectedList"
                    >
                        创建
                    </button>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<script lang="ts">
import { defineComponent, ref, watch, computed } from 'vue';
import { DatePicker } from 'v-calendar';
import { useListStore } from '@/stores/listStore';
import type { FEvent } from 'src-tauri/bindings/FEvent';

export default defineComponent({
    name: 'AddCardModal',
    components: {
        VDatePicker: DatePicker
    },
    props: {
        modelValue: {
            type: Boolean,
            default: false
        }
    },
    emits: ['update:modelValue', 'confirm'],
    setup(props, { emit }) {
        const listStore = useListStore();
        
        const formData = ref<FEvent>({
            id: '',
            title: '',
            ddl: '',
            listid: '',
            tag: [],
            create: new Date().getTime().toString(),
            finished: false,
            priority: 'Undefined',
            icon: '',
            color: ''
        });

        const lists = ref<Array<{ id: string; title: string }>>([]);
        const selectedList = ref<string>('');

        const dateValue = computed({
            get: () => formData.value.ddl ? new Date(Number(formData.value.ddl)) : null,
            set: (date: Date | null) => {
                formData.value.ddl = date ? date.getTime().toString() : '';
            }
        });

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark');
        };

        const loadLists = async () => {
            try {
                const result = await listStore.fetchLists();
                lists.value = result.map(list => ({
                    id: list.id,
                    title: list.title
                }));
            } catch (error) {
                console.error('加载列表失败:', error);
            }
        };

        const handleConfirm = () => {
            formData.value.listid = selectedList.value;
            emit('confirm', formData.value);
            handleClose();
        };

        const handleClose = () => {
            emit('update:modelValue', false);
            formData.value = {
                id: '',
                title: '',
                ddl: '',
                listid: '',
                tag: [],
                create: new Date().getTime().toString(),
                finished: false,
                priority: 'Undefined',
                icon: '',
                color: ''
            };
            selectedList.value = '';
        };

        watch(() => props.modelValue, async (val) => {
            if (val) {
                await loadLists();
            }
        });

        return {
            formData,
            lists,
            selectedList,
            dateValue,
            handleConfirm,
            handleClose,
            isDarkMode
        };
    }
});
</script>

<style scoped>
@import '@/styles/Modals/addmodal.css';

.modal-layout {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.form-group {
    margin-bottom: 1.2rem;
}

.modal-footer {
    margin-top: 2rem;
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
}

.confirm-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}
</style>
