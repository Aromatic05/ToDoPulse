import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FEvent } from 'src-tauri/bindings/FEvent'

export type ExportFormat = 'ics' | 'json' | 'md' | 'markdown'

interface AppSettings {
  theme: string
  exportDirectory: string
  notificationsEnabled: boolean
  [key: string]: string | boolean | number | undefined
}

export const useSettingStore = defineStore('settings', () => {
    // 状态
    const settings = reactive<AppSettings>({
        theme: 'system',
        exportDirectory: '',
        notificationsEnabled: true
    })

    const isLoading = ref(false)
    const error = ref<string | null>(null)
    const exportableEvents = ref<FEvent[]>([])

    // 操作
    /**
     * 加载应用设置
     */
    async function loadSettings() {
        isLoading.value = true
        error.value = null

        try {
            // 加载默认导出路径
            settings.exportDirectory = await getDefaultExportPath()

            // 这里可以添加其他设置的加载

        } catch (err) {
            console.error('加载设置失败:', err)
            error.value = `加载设置失败: ${err}`
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 保存应用设置
     * @returns 保存是否成功
     */
    async function saveSettings() {
        isLoading.value = true
        error.value = null

        try {
            // 这里可以实现设置的保存
            console.log('保存设置', settings)

            // 实际项目中可能需要调用后端API保存设置
            // await invoke('save_settings', { settings: JSON.stringify(settings) })

            return true
        } catch (err) {
            console.error('保存设置失败:', err)
            error.value = `保存设置失败: ${err}`
            return false
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 获取默认导出路径
     * @returns 默认导出路径字符串
     */
    async function getDefaultExportPath() {
        try {
            return await invoke<string>('get_export_directory')
        } catch (err) {
            console.error('获取默认导出路径失败', err)
            error.value = `获取默认导出路径失败: ${err}`
            return ''
        }
    }

    /**
     * 选择文件保存路径
     * @param suggestedName 建议的文件名
     * @param extension 文件扩展名
     * @returns 选择的文件路径或null
     */
    async function selectSavePath(suggestedName: string, extension: string) {
        try {
            const result = await invoke<string | null>('select_save_path', {
                suggestedName,
                extension
            })
            return result
        } catch (err) {
            console.error('选择保存路径失败', err)
            error.value = `选择保存路径失败: ${err}`
            return null
        }
    }

    /**
     * 获取可导出的所有事件
     * @returns 可导出的事件数组
     */
    async function fetchExportableEvents() {
        isLoading.value = true
        error.value = null

        try {
            exportableEvents.value = await invoke<FEvent[]>('get_all_events')
            return [...exportableEvents.value]
        } catch (err) {
            console.error('获取可导出事件失败', err)
            error.value = `获取可导出事件失败: ${err}`
            exportableEvents.value = []
            return []
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 导出所有事件
     * @param format 导出格式
     * @param customPath 自定义保存路径
     * @returns 保存的文件路径
     */
    async function exportAllEvents(format: ExportFormat, customPath?: string) {
        isLoading.value = true
        error.value = null

    try {
      let exportContent = ''
      const filename = format === 'ics' ? 'all_todopulse_events' : 'all_todopulse_events'

            // 根据不同格式调用相应API
            switch (format) {
                case 'ics':
                    exportContent = await invoke<string>('export_all_events_to_ics')
                    break
                case 'json':
                    exportContent = await invoke<string>('export_all_events_to_json')
                    break
                case 'md':
                case 'markdown':
                    exportContent = await invoke<string>('export_all_events_to_md')
                    break
                default:
                    throw new Error(`不支持的导出格式: ${format}`)
            }

            // 将导出内容保存到文件
            const filePath = await invoke<string>('save_export_file', {
                content: exportContent,
                filename,
                format,
                customPath
            })

            return filePath
        } catch (err) {
            console.error('导出所有事件失败', err)
            error.value = `导出所有事件失败: ${err}`
            throw err
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 导出选定的事件
     * @param eventIds 事件ID数组
     * @param format 导出格式
     * @param customPath 自定义保存路径
     * @returns 保存的文件路径
     */
    async function exportEvents(eventIds: string[], format: ExportFormat, customPath?: string) {
        isLoading.value = true
        error.value = null

        try {
            if (eventIds.length === 0) {
                throw new Error('没有选择任何事件')
            }

      const exportContent = ''
      const filename = format === 'ics' ? 'todopulse_events' : 'todopulse_events'

            // 根据不同格式调用相应API
            await invoke<string>('export_events_to_ics', { format, eventIds })

            // 将导出内容保存到文件
            const filePath = await invoke<string>('save_export_file', {
                content: exportContent,
                filename,
                format,
                customPath
            })

            return filePath
        } catch (err) {
            console.error('导出事件失败', err)
            error.value = `导出事件失败: ${err}`
            throw err
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 导出单个事件
     * @param eventId 事件ID
     * @param format 导出格式
     * @param customPath 自定义保存路径
     * @returns 保存的文件路径
     */
    async function exportSingleEvent(eventId: string, format: ExportFormat, customPath?: string) {
        isLoading.value = true
        error.value = null

        try {
            let exportContent = ''

            // 根据格式调用相应API
            switch (format) {
                case 'ics':
                    exportContent = await invoke<string>('export_events_to_ics', { uuids: [eventId] })
                    break
                case 'json':
                    exportContent = await invoke<string>('export_events_to_json', { uuids: [eventId] })
                    break
                case 'md':
                case 'markdown':
                    exportContent = await invoke<string>('export_events_to_md', { uuids: [eventId] })
                    break
                default:
                    throw new Error(`不支持的导出格式: ${format}`)
            }

            // 保存到文件
            const filename = `event_${eventId.substring(0, 8)}`
            const filePath = await invoke<string>('save_export_file', {
                content: exportContent,
                filename,
                format,
                customPath
            })

            return filePath
        } catch (err) {
            console.error('导出单个事件失败', err)
            error.value = `导出单个事件失败: ${err}`
            throw err
        } finally {
            isLoading.value = false
        }
    }

    async function exportEventsByStatus(finished: boolean, format: ExportFormat, customPath?: string) {
        isLoading.value = true
        error.value = null

    try {
      let exportContent = ''
      const status = finished ? 'completed' : 'pending'
      const filename = `${status}_events`

            // 根据格式选择相应的API
            switch (format) {
                case 'ics':
                    exportContent = await invoke<string>('export_events_by_status', { finished })
                    break
                case 'json':
                    exportContent = await invoke<string>('export_events_by_status_to_json', { finished })
                    break
                case 'md':
                case 'markdown':
                    exportContent = await invoke<string>('export_events_by_status_to_md', { finished })
                    break
                default:
                    throw new Error(`不支持的导出格式: ${format}`)
            }

            // 将导出内容保存到文件
            const filePath = await invoke<string>('save_export_file', {
                content: exportContent,
                filename,
                format,
                customPath
            })

            return filePath
        } catch (err) {
            console.error(`导出${finished ? '已完成' : '未完成'}事件失败`, err)
            error.value = `导出${finished ? '已完成' : '未完成'}事件失败: ${err}`
            throw err
        } finally {
            isLoading.value = false
        }
    }

    // 设置更新函数
    function updateSettings(newSettings: Partial<AppSettings>) {
        Object.assign(settings, newSettings)
    }

    return {
        // 状态
        settings,
        isLoading,
        error,
        exportableEvents,

        // 操作
        loadSettings,
        saveSettings,
        getDefaultExportPath,
        selectSavePath,
        fetchExportableEvents,
        exportAllEvents,
        exportEvents,
        exportSingleEvent,
        exportEventsByStatus,
        updateSettings
    }
})