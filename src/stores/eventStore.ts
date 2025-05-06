import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { FEvent } from 'src-tauri/bindings/FEvent'
import { Priority } from 'src-tauri/bindings/Priority'
import { useListStore } from './listStore'
import EventService from '../services/EventService'

export const useEventStore = defineStore('events', () => {
  // 状态
  const events = ref<Map<string, FEvent[]>>(new Map()) // 以列表ID为键存储事件列表
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedEventId = ref<string | null>(null)
  const selectedEventContent = ref<string>('')
  const pageInfo = ref<Map<string, { currentPage: number, hasMore: boolean }>>(new Map())

  // 服务实例
  const eventService = EventService

  // 获取listStore
  const listStore = useListStore()

  // 计算属性
  const getEventsByListId = computed(() => {
    return (listId: string) => events.value.get(listId) || []
  })

  const getEventById = computed(() => {
    return (eventId: string, listId?: string) => {
      if (listId && events.value.has(listId)) {
        return events.value.get(listId)?.find(event => event.id === eventId)
      }
      
      // 如果没有提供listId，则在所有事件中查找
      for (const eventList of events.value.values()) {
        const found = eventList.find(event => event.id === eventId)
        if (found) return found
      }
      
      return undefined
    }
  })

  const selectedEvent = computed(() => {
    if (!selectedEventId.value) return null
    
    for (const eventList of events.value.values()) {
      const found = eventList.find(event => event.id === selectedEventId.value)
      if (found) return found
    }
    
    return null
  })

  // 获取列表的分页信息
  const getPageInfo = computed(() => {
    return (listId: string) => {
      const info = pageInfo.value.get(listId)
      return info || { currentPage: 0, hasMore: true }
    }
  })

  // 操作
  async function fetchEventsByListId(listId: string, loadMore: boolean = false) {
    isLoading.value = true
    error.value = null

    try {
      const info = getPageInfo.value(listId)
      const nextPage = loadMore ? info.currentPage + 1 : 1
      
      const result = await eventService.getEventsByListId(listId, nextPage)
      
      // 更新事件数据
      if (loadMore && events.value.has(listId)) {
        const currentEvents = events.value.get(listId) || []
        events.value.set(listId, [...currentEvents, ...result.events])
      } else {
        events.value.set(listId, result.events)
      }
      
      // 更新分页信息
      pageInfo.value.set(listId, {
        currentPage: nextPage,
        hasMore: result.hasMore
      })

      return [...events.value.get(listId) || []]
    } catch (err) {
      console.error('获取列表事件失败:', err)
      error.value = `获取事件失败: ${err}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function addEvent(listId: string, title: string, priority: Priority = "Medium", timestamp: string = Date.now().toString()) {
    isLoading.value = true
    error.value = null

    try {
      // 确认列表存在
      if (!listStore.getListById(listId)) {
        await listStore.fetchLists()
        if (!listStore.getListById(listId)) {
          console.error(`列表ID ${listId} 不存在`)
          error.value = `列表 ${listId} 不存在`
          return []
        }
      }

      await eventService.addEvent(listId, title, priority, timestamp)
      
      // 重新加载第一页数据
      await fetchEventsByListId(listId)
      
      return [...(events.value.get(listId) || [])]
    } catch (err) {
      console.error('添加事件失败:', err)
      error.value = `添加事件失败: ${err}`
      return [...(events.value.get(listId) || [])]
    } finally {
      isLoading.value = false
    }
  }

  async function updateEvent(fEvent: FEvent) {
    isLoading.value = true
    error.value = null

    try {
      await eventService.updateEvent(fEvent)
      
      // 更新本地数据
      const listEvents = events.value.get(fEvent.listid) || []
      const index = listEvents.findIndex(e => e.id === fEvent.id)
      if (index !== -1) {
        listEvents[index] = fEvent
        events.value.set(fEvent.listid, [...listEvents])
      }
      
      return [...(events.value.get(fEvent.listid) || [])]
    } catch (err) {
      console.error('更新事件失败:', err)
      error.value = `更新事件失败: ${err}`
      return [...(events.value.get(fEvent.listid) || [])]
    } finally {
      isLoading.value = false
    }
  }

  async function deleteEvent(eventId: string, listId: string) {
    isLoading.value = true
    error.value = null

    try {
      await eventService.deleteEvent(eventId, listId)
      
      // 更新本地数据
      const listEvents = events.value.get(listId) || []
      events.value.set(listId, listEvents.filter(e => e.id !== eventId))
      
      // 如果删除的是当前选中的事件，清除选中状态
      if (selectedEventId.value === eventId) {
        clearSelectedEvent()
      }
      
      return [...(events.value.get(listId) || [])]
    } catch (err) {
      console.error('删除事件失败:', err)
      error.value = `删除事件失败: ${err}`
      return [...(events.value.get(listId) || [])]
    } finally {
      isLoading.value = false
    }
  }

  async function getEventContent(eventId: string) {
    isLoading.value = true
    error.value = null

    try {
      const content = await eventService.getEventContent(eventId)
      selectedEventId.value = eventId
      selectedEventContent.value = content
      return content
    } catch (err) {
      console.error('获取事件内容失败:', err)
      error.value = `获取事件内容失败: ${err}`
      return ''
    } finally {
      isLoading.value = false
    }
  }

  async function saveEventContent(eventId: string, content: string) {
    isLoading.value = true
    error.value = null

    try {
      const updatedContent = await eventService.saveEventContent(eventId, content)
      
      // 更新选中事件内容
      if (selectedEventId.value === eventId) {
        selectedEventContent.value = updatedContent
      }
      
      return updatedContent
    } catch (err) {
      console.error('保存事件内容失败:', err)
      error.value = `保存事件内容失败: ${err}`
      return selectedEventContent.value
    } finally {
      isLoading.value = false
    }
  }

  function clearSelectedEvent() {
    selectedEventId.value = null
    selectedEventContent.value = ''
  }

  // 清除所有缓存
  function clearCache() {
    events.value.clear()
    pageInfo.value.clear()
    clearSelectedEvent()
    eventService.clearAllCache()
  }

  return {
    // 状态
    events,
    isLoading,
    error,
    selectedEventId,
    selectedEventContent,
    
    // 计算属性
    getEventsByListId,
    getEventById,
    selectedEvent,
    getPageInfo,
    
    // 操作
    fetchEventsByListId,
    addEvent,
    updateEvent,
    deleteEvent,
    getEventContent,
    saveEventContent,
    clearSelectedEvent,
    clearCache
  }
})