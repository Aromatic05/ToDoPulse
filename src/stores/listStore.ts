import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { FList } from 'src-tauri/bindings/FList'
import ListService from '../services/ListService'

export const useListStore = defineStore('lists', () => {
  // 状态
  const lists = ref<FList[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // 服务实例
  const listService = ListService

  // 计算属性
  const getListById = computed(() => {
    return (id: string) => lists.value.find(list => list.id === id)
  })

  const sortedLists = computed(() => {
    return [...lists.value].sort((a, b) => a.title.localeCompare(b.title))
  })

  // 操作
  async function fetchLists() {
    isLoading.value = true
    error.value = null

    try {
      // 使用 ListService 获取列表数据
      const result = await listService.getLists()
      
      // 验证返回数据类型
      if (!Array.isArray(result)) {
        console.error('后端返回的数据不是数组:', result)
        lists.value = []
        error.value = '获取列表失败: 数据格式错误'
        return []
      }
      
      lists.value = result
      return [...lists.value]
    } catch (err) {
      console.error('获取列表失败:', err)
      error.value = `获取列表失败: ${err}`
      // 确保总是返回一个数组，避免其他地方出错
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function createList(title: string, icon: string = 'mdi-format-list-bulleted') {
    isLoading.value = true
    error.value = null

    try {
      // 使用 ListService 创建列表
      const newList = await listService.createList(title, icon)
      
      // 添加到本地列表
      lists.value.push(newList)
      console.log(`Store: 已创建ID为 ${newList.id} 的新列表`)
      return [...lists.value]
    } catch (err) {
      console.error(`Store: 创建新列表失败 - ${err}`)
      error.value = `创建列表失败: ${err}`
      return [...lists.value]
    } finally {
      isLoading.value = false
    }
  }

  async function renameList(id: string, newTitle: string) {
    isLoading.value = true
    error.value = null

    const listItem = lists.value.find(l => l.id === id)
    if (!listItem) {
      error.value = `列表 ${id} 不存在`
      isLoading.value = false
      return [...lists.value]
    }

    try {
      // 使用 ListService 重命名列表
      await listService.renameList(id, newTitle)
      
      // 更新本地数据
      listItem.title = newTitle
      console.log(`Store: 列表 ${id} 已重命名为 ${newTitle}`)
      return [...lists.value]
    } catch (err) {
      console.error(`Store: 重命名列表失败 - ${err}`)
      error.value = `重命名列表失败: ${err}`
      return [...lists.value]
    } finally {
      isLoading.value = false
    }
  }

  async function deleteList(id: string) {
    isLoading.value = true
    error.value = null

    const index = lists.value.findIndex(l => l.id === id)
    if (index === -1) {
      error.value = `列表 ${id} 不存在`
      isLoading.value = false
      return [...lists.value]
    }

    try {
      // 使用 ListService 删除列表
      await listService.deleteList(id)
      
      // 更新本地数据
      lists.value.splice(index, 1)
      console.log(`Store: 列表 ${id} 已删除`)
      return [...lists.value]
    } catch (err) {
      console.error(`Store: 删除列表失败 - ${err}`)
      error.value = `删除列表失败: ${err}`
      return [...lists.value]
    } finally {
      isLoading.value = false
    }
  }

  // 清除缓存
  function clearCache() {
    lists.value = []
    listService.clearAllCache()
  }

  return {
    // 状态
    lists,
    isLoading,
    error,
    
    // 计算属性
    getListById,
    sortedLists,
    
    // 操作
    fetchLists,
    createList,
    renameList,
    deleteList,
    clearCache
  }
})