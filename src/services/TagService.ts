import { invoke } from '@tauri-apps/api/core'
import type { TagColor } from 'src-tauri/bindings/TagColor'
import type { FEvent } from 'src-tauri/bindings/FEvent'

// 标签接口
export interface Tag {
  id: number
  name: string
  color: TagColor
}

// 标签缓存接口
interface TagCache {
  tags: Tag[]
  timestamp: number
}

// 标签内容缓存接口
interface TagContentCache {
  events: FEvent[]
  timestamp: number
}

export class TagService {
  private static instance: TagService
  private tagCache: TagCache | null = null
  private tagContentCache: Map<string, TagContentCache> = new Map()
  private readonly CACHE_EXPIRY = 5 * 60 * 1000 // 5分钟缓存过期

  private constructor() {}

  /**
   * 获取TagService单例实例
   * @returns TagService的单例实例
   */
  static getInstance(): TagService {
    if (!TagService.instance) {
      TagService.instance = new TagService()
    }
    return TagService.instance
  }

  /**
   * 获取所有标签
   * @returns 标签数组
   */
  async getTags(): Promise<Tag[]> {
    const now = Date.now()

    // 检查缓存是否有效
    if (this.tagCache && now - this.tagCache.timestamp < this.CACHE_EXPIRY) {
      return this.tagCache.tags
    }

    // 获取新数据
    try {
      const tags = await invoke<Tag[]>('get_tags')

      if (!Array.isArray(tags)) {
        throw new Error('Invalid response format')
      }

      // 更新缓存
      this.tagCache = {
        tags,
        timestamp: now
      }

      return tags
    } catch (error) {
      console.error('获取标签失败:', error)
      throw error
    }
  }

  /**
   * 添加新标签
   * @param name 标签名称
   * @param color 标签颜色
   */
  async addTag(name: string, color: TagColor): Promise<void> {
    try {
      await invoke('add_tag', { tag: name, color })
      // 使缓存失效
      this.invalidateTagCache()
    } catch (error) {
      console.error('添加标签失败:', error)
      throw error
    }
  }

  /**
   * 删除标签
   * @param name 标签名称
   */
  async deleteTag(name: string): Promise<void> {
    try {
      await invoke('delete_tag', { tag: name })
      // 使缓存失效
      this.invalidateTagCache()
      this.invalidateTagContentCache(name)
    } catch (error) {
      console.error('删除标签失败:', error)
      throw error
    }
  }

  /**
   * 获取标签内容（具有该标签的所有事件）
   * @param tagName 标签名称
   * @returns 事件数组
   */
  async getTagContent(tagName: string): Promise<FEvent[]> {
    const cache = this.tagContentCache.get(tagName)
    const now = Date.now()

    // 检查缓存是否有效
    if (cache && now - cache.timestamp < this.CACHE_EXPIRY) {
      return cache.events
    }

    // 获取新数据
    try {
      const events = await invoke<FEvent[]>('tag_content', { tag: tagName })

      if (!Array.isArray(events)) {
        throw new Error('Invalid response format')
      }

      // 更新缓存
      this.tagContentCache.set(tagName, {
        events,
        timestamp: now
      })

      return events
    } catch (error) {
      console.error('获取标签内容失败:', error)
      throw error
    }
  }

  /**
   * 使标签缓存失效
   * @private
   */
  private invalidateTagCache(): void {
    this.tagCache = null
  }

  /**
   * 使标签内容缓存失效
   * @param tagName 标签名称
   * @private
   */
  private invalidateTagContentCache(tagName: string): void {
    this.tagContentCache.delete(tagName)
  }

  /**
   * 清除所有缓存
   */
  clearAllCache(): void {
    this.tagCache = null
    this.tagContentCache.clear()
  }
}

export default TagService.getInstance()