import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { TagColor } from 'src-tauri/bindings/TagColor'
import { FEvent } from 'src-tauri/bindings/FEvent'
import TagService, { Tag } from '../services/TagService'

export const useTagStore = defineStore('tags', () => {
  // 状态
  const tags = ref<Tag[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedTagName = ref<string | null>(null)
  const tagEvents = ref<Map<string, FEvent[]>>(new Map()) // 以标签名为键存储事件列表

  // 服务实例
  const tagService = TagService

  // 计算属性
  const getTagByName = computed(() => {
    return (name: string) => tags.value.find(tag => tag.name === name)
  })

  const selectedTag = computed(() => {
    if (!selectedTagName.value) return null
    return tags.value.find(tag => tag.name === selectedTagName.value) || null
  })

  const getEventsByTagName = computed(() => {
    return (tagName: string) => tagEvents.value.get(tagName) || []
  })

  // 操作
  /**
   * 获取所有标签
   * @returns 标签数组
   */
  async function fetchTags() {
    isLoading.value = true
    error.value = null

    try {
      const result = await tagService.getTags()
      tags.value = result
      return [...tags.value]
    } catch (err) {
      console.error('获取标签失败:', err)
      error.value = `获取标签失败: ${err}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 添加新标签
   * @param name 标签名称
   * @param color 标签颜色
   * @returns 更新后的标签数组
   */
  async function addTag(name: string, color: TagColor) {
    isLoading.value = true
    error.value = null

    try {
      await tagService.addTag(name, color)
      // 重新加载标签列表
      await fetchTags()
      return [...tags.value]
    } catch (err) {
      console.error('添加标签失败:', err)
      error.value = `添加标签失败: ${err}`
      return [...tags.value]
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除标签
   * @param name 标签名称
   * @returns 更新后的标签数组
   */
  async function deleteTag(name: string) {
    isLoading.value = true
    error.value = null

    try {
      await tagService.deleteTag(name)
      
      // 清除相关缓存
      tagEvents.value.delete(name)
      
      // 如果删除的是当前选中的标签，清除选中状态
      if (selectedTagName.value === name) {
        clearSelectedTag()
      }
      
      // 重新加载标签列表
      await fetchTags()
      return [...tags.value]
    } catch (err) {
      console.error('删除标签失败:', err)
      error.value = `删除标签失败: ${err}`
      return [...tags.value]
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取标签内容（具有该标签的所有事件）
   * @param tagName 标签名称
   * @returns 事件数组
   */
  async function getTagContent(tagName: string) {
    isLoading.value = true
    error.value = null

    try {
      const events = await tagService.getTagContent(tagName)
      tagEvents.value.set(tagName, events)
      selectedTagName.value = tagName
      return [...events]
    } catch (err) {
      console.error('获取标签内容失败:', err)
      error.value = `获取标签内容失败: ${err}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 清除当前选中的标签
   */
  function clearSelectedTag() {
    selectedTagName.value = null
  }

  /**
   * 清除所有缓存
   */
  function clearCache() {
    tags.value = []
    tagEvents.value.clear()
    clearSelectedTag()
    tagService.clearAllCache()
  }

  return {
    // 状态
    tags,
    isLoading,
    error,
    selectedTagName,
    tagEvents,

    // 计算属性
    getTagByName,
    selectedTag,
    getEventsByTagName,

    // 操作
    fetchTags,
    addTag,
    deleteTag,
    getTagContent,
    clearSelectedTag,
    clearCache
  }
})