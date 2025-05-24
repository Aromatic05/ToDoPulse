import { defineStore } from 'pinia'
import { ref, reactive, computed } from 'vue'
import type { FEvent } from 'src-tauri/bindings/FEvent'
import type { Priority } from 'src-tauri/bindings/Priority'
import { invoke } from '@tauri-apps/api/core'

export interface TimelineGroup {
    id: string
    title: string
    iconName: string
    color: string
    dateGroup: string
}

export const useTimelineStore = defineStore('timeline', () => {
    // 常量
    const timeMap = {
        TODAY: 'today',
        TOMORROW: 'tomorrow',
        THIS_WEEK: 'this_week',
        NEXT_WEEK: 'next_week',
        OVERDUE: 'overdue',
    } as const

    const showedTimelineGroups = reactive<TimelineGroup[]>([]);

    // 状态
    const timelineGroups = reactive<TimelineGroup[]>([
        {
            id: timeMap.TODAY,
            title: "今天",
            iconName: "mdi-calendar-today",
            color: "primary",
            dateGroup: timeMap.TODAY
        },
        {
            id: timeMap.TOMORROW,
            title: "明天",
            iconName: "mdi-calendar-arrow-right",
            color: "secondary",
            dateGroup: timeMap.TOMORROW
        },
        {
            id: timeMap.THIS_WEEK,
            title: "本周",
            iconName: "mdi-calendar-week",
            color: "secondary",
            dateGroup: timeMap.THIS_WEEK
        },
        {
            id: timeMap.NEXT_WEEK,
            title: "下周",
            iconName: "mdi-calendar-week",
            color: "info",
            dateGroup: timeMap.NEXT_WEEK
        },
        {
            id: timeMap.OVERDUE,
            title: "逾期",
            iconName: "mdi-calendar-alert",
            color: "error",
            dateGroup: timeMap.OVERDUE
        }
    ])

    const events = ref<Record<string, FEvent[]>>({})
    const dataInitialized = ref(false)
    const isLoading = ref(false)
    const error = ref<string | null>(null)

    // 计算属性
    const getGroupItems = computed(() => {
        return (dateGroup: string) => events.value[dateGroup] || []
    })

    const getAllEvents = computed(() => {
        return Object.values(events.value).flat()
    })

    // 操作
    /**
     * 获取时间线上的所有事件
     */
    async function fetchEvents() {
        if (dataInitialized.value) return

        isLoading.value = true
        error.value = null

        try {
            await Promise.all(
                Object.keys(timeMap).map(async (time) => {
                    const dateGroup = timeMap[time as keyof typeof timeMap]
                    try {
                        const groupEvents: FEvent[] = await invoke('filter_events', { filter: dateGroup })
                        if (groupEvents.length) {
                            const matchedGroup = timelineGroups.find(group => group.dateGroup === dateGroup);
                            if (matchedGroup) {
                                showedTimelineGroups.push(matchedGroup);
                            }
                        }
                        events.value[dateGroup] = groupEvents as FEvent[]
                    } catch (err) {
                        console.error(`获取${dateGroup}事件失败:`, err)
                        error.value = `获取${dateGroup}事件失败: ${err}`
                        events.value[dateGroup] = []
                    }
                })
            )

            dataInitialized.value = true
        } catch (err) {
            console.error('加载时间线数据失败:', err)
            error.value = `加载时间线数据失败: ${err}`
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 更新时间线上的事件
     * @param updatedData 更新后的事件数据
     * @param dateGroup 日期分组
     * @returns 更新是否成功
     */
    async function updateEvent(updatedData: FEvent, dateGroup: string) {
        isLoading.value = true
        error.value = null

        try {
            const items = events.value[dateGroup]
            if (!items) return

            const index = items.findIndex(item => item.id === updatedData.id)
            if (index !== -1) {
                // 先更新本地数据
                items[index] = { ...items[index], ...updatedData }

                // 再调用后端API
                await invoke('update_event', { fEvent: updatedData })
                return true
            }
            return false
        } catch (err) {
            console.error('更新事件失败:', err)
            error.value = `更新事件失败: ${err}`
            return false
        } finally {
            isLoading.value = false
        }
    }


    /**
     * 获取颜色变量对应的CSS变量值
     * @param color 颜色名称
     * @returns CSS变量值
     */
    function getColorVariable(color: string): string {
        const colorMap: Record<string, string> = {
            'primary': 'var(--md-sys-color-primary)',
            'secondary': 'var(--md-sys-color-secondary)',
            'info': 'var(--md-sys-color-tertiary)',
            'success': 'var(--md-sys-color-success)',
            'warning': 'var(--md-sys-color-warning)',
            'error': 'var(--md-sys-color-error)'
        }
        return colorMap[color] || 'var(--md-sys-color-on-surface-variant)'
    }

    /**
     * 按优先级对事件进行排序
     * @param items 要排序的事件数组
     * @returns 排序后的事件数组
     */
    function sortItemsByPriority(items: FEvent[]): FEvent[] {
        const priority: Record<Priority, number> = {
            'High': 3,
            'Medium': 2,
            'Low': 1,
            'Undefined': 0
        }

        return [...items].sort((a, b) => {
            const weightA = priority[a.priority] ?? 0
            const weightB = priority[b.priority] ?? 0
            return weightB - weightA
        })
    }

    return {
        // 状态
        timelineGroups,
        showedTimelineGroups,
        events,
        dataInitialized,
        isLoading,
        error,

        // 计算属性
        getGroupItems,
        getAllEvents,

        // 操作
        fetchEvents,
        updateEvent,
        getColorVariable,
        sortItemsByPriority
    }
})