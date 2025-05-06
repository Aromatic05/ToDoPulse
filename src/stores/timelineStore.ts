import { defineStore } from 'pinia'
import { ref, reactive, computed } from 'vue'
import { FEvent } from 'src-tauri/bindings/FEvent'
import { Priority } from 'src-tauri/bindings/Priority'
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
    NEXT_WEEK: 'next_week',
  } as const

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
      id: timeMap.NEXT_WEEK,
      title: "下周",
      iconName: "mdi-calendar-week",
      color: "info",
      dateGroup: timeMap.NEXT_WEEK
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
  async function fetchEvents() {
    if (dataInitialized.value) return
    
    isLoading.value = true
    error.value = null
    
    try {
      await Promise.all(
        Object.keys(timeMap).map(async (time) => {
          const dateGroup = timeMap[time as keyof typeof timeMap]
          try {
            const groupEvents = await invoke('filter_events', { filter: dateGroup })
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
  
  function formatCardData(item: FEvent, dateGroup: string): FEvent {
    // 确保tags包含dateGroup
    const tag = [...(item.tag || [])]
    if (!tag.includes(dateGroup)) {
      tag.push(dateGroup)
    }

    return {
      ...item,
      tag
    }
  }

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
    formatCardData,
    getColorVariable,
    sortItemsByPriority
  }
})