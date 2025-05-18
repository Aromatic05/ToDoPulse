import { FList } from 'src-tauri/bindings/FList'
import { invoke } from '@tauri-apps/api/core'
import { FEvent } from 'src-tauri/bindings/FEvent'

interface ListCache {
  data: FList[]
  timestamp: number
}

interface ListContentCache {
  data: FEvent[]
  timestamp: number
  hasMore: boolean
  pageSize: number
  currentPage: number
}

export class ListService {
  private static instance: ListService
  private listsCache: ListCache | null = null
  private listContentsCache: Map<string, ListContentCache> = new Map()
  private readonly CACHE_EXPIRY = 5 * 60 * 1000 // 5分钟缓存过期
  private readonly PAGE_SIZE = 20

  private constructor() {}

  /**
   * 获取ListService单例实例
   * @returns ListService的单例实例
   */
  static getInstance(): ListService {
    if (!ListService.instance) {
      ListService.instance = new ListService()
    }
    return ListService.instance
  }

  /**
   * 获取所有列表
   * @returns 所有列表的数组
   */
  async getLists(): Promise<FList[]> {
    const now = Date.now()

    // 检查缓存是否有效
    if (this.listsCache && now - this.listsCache.timestamp < this.CACHE_EXPIRY) {
      return this.listsCache.data
    }

    // 获取新数据
    try {
      const lists = await invoke<FList[]>('get_lists')

      if (!Array.isArray(lists)) {
        throw new Error('Invalid response format')
      }

      // 更新缓存
      this.listsCache = {
        data: lists,
        timestamp: now
      }

      return lists
    } catch (error) {
      console.error('Failed to fetch lists:', error)
      throw error
    }
  }

  /**
   * 获取列表内容（支持分页）
   * @param listId 列表ID
   * @param page 页码，默认为1
   * @returns 包含事件数组和是否有更多数据的对象
   */
  async getListContent(listId: string, page: number = 1): Promise<{ events: FEvent[], hasMore: boolean }> {
    const cacheKey = listId
    const cache = this.listContentsCache.get(cacheKey)
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
        page_size: this.PAGE_SIZE
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
        
        this.listContentsCache.set(cacheKey, {
          data: newData,
          timestamp: now,
          hasMore,
          pageSize: this.PAGE_SIZE,
          currentPage: page
        })
      } else {
        this.listContentsCache.set(cacheKey, {
          data: events,
          timestamp: now,
          hasMore,
          pageSize: this.PAGE_SIZE,
          currentPage: page
        })
      }

      return { events, hasMore }
    } catch (error) {
      console.error('Failed to fetch list contents:', error)
      throw error
    }
  }

  /**
   * 创建新列表
   * @param title 列表标题
   * @param icon 列表图标，默认为"mdi-folder"
   * @returns 新创建的列表对象
   */
  async createList(title: string, icon: string = "mdi-folder"): Promise<FList> {
    try {
      const newList = await invoke<FList>('new_list', { title, icon })
      
      // 使缓存失效
      this.invalidateListsCache()
      
      return newList
    } catch (error) {
      console.error('Failed to create list:', error)
      throw error
    }
  }

  /**
   * 删除列表
   * @param listId 要删除的列表ID
   */
  async deleteList(listId: string): Promise<void> {
    try {
      await invoke('delete_list', { listid: listId })
      
      // 使缓存失效
      this.invalidateListsCache()
      this.invalidateListContentCache(listId)
    } catch (error) {
      console.error('Failed to delete list:', error)
      throw error
    }
  }

  /**
   * 重命名列表
   * @param listId 列表ID
   * @param newTitle 新的列表标题
   */
  async renameList(listId: string, newTitle: string): Promise<void> {
    try {
      await invoke('rename_list', { listid: listId, new: newTitle })
      
      // 使缓存失效
      this.invalidateListsCache()
    } catch (error) {
      console.error('Failed to rename list:', error)
      throw error
    }
  }

  /**
   * 使列表缓存失效
   * @private
   */
  private invalidateListsCache(): void {
    this.listsCache = null
  }

  /**
   * 使列表内容缓存失效
   * @param listId 列表ID
   * @private
   */
  private invalidateListContentCache(listId: string): void {
    this.listContentsCache.delete(listId)
  }

  /**
   * 清除所有缓存
   */
  clearAllCache(): void {
    this.listsCache = null
    this.listContentsCache.clear()
  }
}

export default ListService.getInstance()