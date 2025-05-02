<template>
    <div class="card-base" @click.stop="handleCardClick">
        <div class="card-content-row">
            <input type="checkbox" :checked="localData.finished" @click.stop="handleComplete()" class="card-checkbox"/>
            <h3 class="card-title" :class="{ 'completed-task': localData.finished }">{{ localData.title }}</h3>
            <span v-if="convertTimestampToTime(localData.ddl)" class="card-time"
                :style="{ color: localData.color || 'var(--md-sys-color-on-surface-variant)' }">
                {{ convertTimestampToTime(localData.ddl) }}
            </span>
        </div>
        <div v-if="localData.tag?.length" class="card-tags">
            <span v-for="(tag, i) in localData.tag" :key="i" class="card-tag">{{ tag }}</span>
        </div>
    </div>

    <CardContentModal v-model="showModal" :card-data="localData" @confirm="handleConfirm" v-if="showModal" />
</template>

<script lang="ts">
import CardContentModal from '@/components/Modals/CardContentModal.vue'
import { FEvent } from 'src-tauri/bindings/FEvent';
import { convertTimestampToDate, convertTimestampToTime } from '@/services/DateTimeService';

export default {
    name: 'EventCard',
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
    emits: ['update', 'delete', 'toggleStatus'],
    data() {
        return {
            showModal: false,
            deleteDialog: false,
            localData: { 
                ...JSON.parse(JSON.stringify(this.data)),
                finished: this.data.finished || false
            } as FEvent
        }
    },
    methods: {
        convertTimestampToDate(timestamp: string | undefined): string | null {
            return timestamp ? convertTimestampToDate(timestamp) : null;
        },
        convertTimestampToTime(timestamp: string | undefined): string | null {
            return timestamp ? convertTimestampToTime(timestamp) : null;
        },
        handleCardClick() {
            this.showModal = true
        },
        handleConfirm(updatedData: FEvent) {
            console.log('EventCard updating:', updatedData)
            this.localData = { ...updatedData } as FEvent
            this.$emit('update', updatedData)
        },
        handleComplete() {
            this.localData.finished = !this.localData.finished;
            this.$emit('toggleStatus', {
                id: this.localData.id,
                finished: this.localData.finished
            });
        },
        handleDelete() {
            this.deleteDialog = true;
        },
    }
}
</script>

<style scoped>
@import '@/styles/Cards/card.css';
</style>