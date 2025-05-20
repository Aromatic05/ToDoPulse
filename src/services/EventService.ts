import type { FEvent } from 'src-tauri/bindings/FEvent'
import { invoke } from '@tauri-apps/api/core'
import type { Priority } from 'src-tauri/bindings/Priority'

interface EventCache {
  data: FEvent[]
  timestamp: number
  hasMore: boolean
  pageSize: number
  currentPage: number
}

interface ContentCache {
  content: string
  timestamp: number
}

export class EventService {
  private static instance: EventService
  private eventCache: Map<string, EventCache> = new Map()
  private contentCache: Map<string, ContentCache> = new Map()
  private readonly CACHE_EXPIRY = 5 * 60 * 1000 // 5分钟缓存过期
  private readonly PAGE_SIZE = 20

  private constructor() {}

  /**
   * 获取EventService单例实例
   * @returns EventService的单例实例
   */
  static getInstance(): EventService {
    if (!EventService.instance) {
      EventService.instance = new EventService()
    }
    return EventService.instance
  }

  /**
   * 获取列表事件（支持分页）
   * @param listId 列表ID
   * @param page 页码，默认为1
   * @returns 包含事件数组和是否有更多数据的对象
   */
  async getEventsByListId(listId: string, page = 1): Promise<{ events: FEvent[], hasMore: boolean }> {
    const cacheKey = `${listId}`
    const cache = this.eventCache.get(cacheKey)
    const now = Date.now()

    // 检查缓存是否有效
    if (cache && 
        now - cache.timestamp < this.CACHE_EXPIRY && 
        page <= cache.currentPage) {
      const start = (page - 1) * this.PAGE_SIZE
      const end = start + this.PAGE_SIZE
      return {
        events: cache.data.slice(start, end),
        hasMore: cache.hasMore
      }
    }

    // 获取新数据
    try {
      const events = await invoke<FEvent[]>('list_content', { 
        listid: listId,
        page,
        pageSize: this.PAGE_SIZE
      })

      if (!Array.isArray(events)) {
        throw new Error('Invalid response format')
      }

      // 更新或创建缓存
      const hasMore = events.length === this.PAGE_SIZE
      if (cache) {
        // 合并新数据
        const newData = page === 1 ? 
          events : 
          [...cache.data.slice(0, (page - 1) * this.PAGE_SIZE), ...events]
        
        this.eventCache.set(cacheKey, {
          data: newData,
          timestamp: now,
          hasMore,
          pageSize: this.PAGE_SIZE,
          currentPage: page
        })
      } else {
        this.eventCache.set(cacheKey, {
          data: events,
          timestamp: now,
          hasMore,
          pageSize: this.PAGE_SIZE,
          currentPage: page
        })
      }

      return { events, hasMore }
    } catch (error) {
      console.error('Failed to fetch events:', error)
      throw error
    }
  }

  /**
   * 获取事件内容
   * @param eventId 事件ID
   * @returns 事件内容字符串
   */
  async getEventContent(eventId: string): Promise<string> {
    const cache = this.contentCache.get(eventId)
    const now = Date.now()

    // 检查缓存是否有效
    if (cache && now - cache.timestamp < this.CACHE_EXPIRY) {
      return cache.content
    }

    // 获取新数据
    try {
      const content = await invoke<string>('event_content', { uuid: eventId })
      
      // 更新缓存
      this.contentCache.set(eventId, {
        content,
        timestamp: now
      })

      return content
    } catch (error) {
      console.error('Failed to fetch event content:', error)
      throw error
    }
  }

  /**
   * 添加事件
   * @param listId 列表ID
   * @param title 事件标题
   * @param priority 事件优先级，默认为Medium
   * @param timestamp 时间戳，默认为当前时间
   */
  async addEvent(listId: string, title: string, priority: Priority = "Medium", timestamp: string = Date.now().toString()): Promise<void> {
    try {
      await invoke('add_event', { listid: listId, title, priority, ddl: timestamp })
      // 使相关缓存失效
      this.invalidateListCache(listId)
    } catch (error) {
      console.error('Failed to add event:', error)
      throw error
    }
  }

  /**
   * 更新事件
   * @param event 要更新的事件对象
   */
  async updateEvent(event: FEvent): Promise<void> {
    try {
      await invoke('update_event', { fEvent: event })
      // 使相关缓存失效
      this.invalidateListCache(event.listid)
      this.invalidateContentCache(event.id)
    } catch (error) {
      console.error('Failed to update event:', error)
      throw error
    }
  }

  /**
   * 删除事件
   * @param eventId 事件ID
   * @param listId 列表ID
   */
  async deleteEvent(eventId: string, listId: string): Promise<void> {
    try {
      await invoke('delete_event', { uuid: eventId })
      // 使相关缓存失效
      this.invalidateListCache(listId)
      this.invalidateContentCache(eventId)
    } catch (error) {
      console.error('Failed to delete event:', error)
      throw error
    }
  }

  /**
   * 保存事件内容
   * @param eventId 事件ID
   * @param content 事件内容
   * @returns 保存的事件内容
   */
  async saveEventContent(eventId: string, content: string): Promise<string> {
    try {
      await invoke('write_content', { uuid: eventId, content })
      // 更新缓存
      this.contentCache.set(eventId, {
        content,
        timestamp: Date.now()
      })
      return content
    } catch (error) {
      console.error('Failed to save event content:', error)
      throw error
    }
  }

  /**
   * 使列表缓存失效
   * @param listId 列表ID
   * @private
   */
  private invalidateListCache(listId: string): void {
    this.eventCache.delete(listId)
  }

  /**
   * 使内容缓存失效
   * @param eventId 事件ID
   * @private
   */
  private invalidateContentCache(eventId: string): void {
    this.contentCache.delete(eventId)
  }

  /**
   * 清除所有缓存
   */
  clearAllCache(): void {
    this.eventCache.clear()
    this.contentCache.clear()
  }
}

export default EventService.getInstance()