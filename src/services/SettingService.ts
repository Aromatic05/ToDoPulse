import { invoke } from '@tauri-apps/api/core';
import type { FList } from 'src-tauri/bindings/FList';
import { useListStore } from '@/stores/listStore';

export class SettingService {
  /**
   * 获取默认导出路径
   */
  static async getDefaultExportPath(): Promise<string> {
    try {
      return await invoke<string>('get_export_directory');
    } catch (error) {
      console.error('获取默认导出路径失败', error);
      return '';
    }
  }

  /**
   * 选择导出文件的保存路径
   * @param suggestedName 建议的文件名
   * @param extension 文件扩展名
   */
  static async selectSavePath(suggestedName: string, extension: string): Promise<string | null> {
    try {
      // 调用后端的文件选择对话框
      const result = await invoke<string | null>('select_save_path', {
        suggestedName,
        extension
      });
      return result;
    } catch (error) {
      console.error('选择保存路径失败', error);
      return null;
    }
  }

  /**
   * 保存应用设置
   * @param settings 设置对象
   */
  static async saveSettings(settings: any): Promise<void> {
    // 实际项目中可能涉及到调用Tauri的API保存设置
    // 这里暂时只打印一下
    console.log('保存设置', settings);
    return Promise.resolve();
  }

  /**
   * 获取可以导出的事件列表
   */
  static async getExportableLists(): Promise<FList[]> {
    try {
      const listStore = useListStore();
      // 调用后端API获取所有事件列表
      return await listStore.fetchLists();
    } catch (error) {
      console.error('获取事件列表失败', error);
      throw error;
    }
  }

  /**
   * 导出所有事件
   * @param format 导出格式
   * @param customPath 自定义保存路径
   */
  static async exportAllEvents(format: string, customPath?: string): Promise<string> {
    try {
      let exportContent = '';
      let filename = format === 'ics' ? 'all_todopulse_events' : 'all_todopulse_events';

      // 根据不同格式直接调用相应的全部导出API
      switch (format) {
        case 'ics':
          // 使用新增的导出所有事件的API
          exportContent = await invoke<string>('export_all_events', {fmt: "ics"});
          break;
        case 'json':
          // 使用JSON导出API
          exportContent = await invoke<string>('export_all_events', {fmt : "json"});
          break;
        case 'md':
          // 使用Markdown导出API
          exportContent = await invoke<string>('export_all_events', {fmt : "md"});
          break;
        default:
          throw new Error(`不支持的导出格式: ${format}`);
      }

      // 将导出内容保存到文件
      const filePath = await invoke<string>('save_export_file', {
        content: exportContent,
        filename,
        format,
        customPath // 添加自定义路径参数
      });

      return filePath;
    } catch (error) {
      console.error('导出所有事件失败', error);
      throw error;
    }
  }

  /**
   * 导出选定的事件
   * @param eventIds 事件ID数组
   * @param format 导出格式
   * @param customPath 自定义保存路径
   */
  static async exportLists(lists: string[], format: string, customPath?: string): Promise<string> {
    try {
      if (lists.length === 0) {
        throw new Error('没有选择任何事件');
      }

      let exportContent = '';
      let filename = format === 'ics' ? 'todopulse_events' : 'todopulse_events';

      // 根据不同格式调用不同的后端导出API
      try {
        const contentPromises = lists.map(list => {
          return invoke<string>('export_list_events', { listId: list, fmt: format });
        });
        
        const contentArray = await Promise.all(contentPromises);
        
        exportContent = contentArray.join('');
      } catch (error) {
        console.error('导出事件失败', error);
        throw new Error('导出事件失败');
      }

      // 将导出内容保存到文件
      const filePath = await invoke<string>('save_export_file', {
        content: exportContent,
        filename,
        format,
        customPath // 添加自定义路径参数
      });

      return filePath;
    } catch (error) {
      console.error('导出事件失败', error);
      throw error;
    }
  }

  /**
   * 导出单个事件
   * @param eventId 事件ID
   * @param format 导出格式
   * @param customPath 自定义保存路径
   */
  static async exportSingleEvent(eventId: string, format: string, customPath?: string): Promise<string> {
    try {
      let exportContent = '';
      
      try {
        exportContent = await invoke<string>('export_events', { eventIds:eventId, fmt: format });
      } catch (e) {
        console.error('导出事件失败', e);
        throw new Error('导出事件失败');
      }

      // 保存到文件
      const filename = `event_${eventId.substring(0, 8)}`;
      const filePath = await invoke<string>('save_export_file', {
        content: exportContent,
        filename,
        format,
        customPath // 添加自定义路径参数
      });

      return filePath;
    } catch (error) {
      console.error('导出单个事件失败', error);
      throw error;
    }
  }

  /**
   * 根据状态导出事件（已完成/未完成）
   * @param finished 是否已完成
   * @param format 导出格式
   * @param customPath 自定义保存路径
   */
  static async exportEventsByStatus(finished: boolean, format: string, customPath?: string): Promise<string> {
    try {
      let exportContent = '';
      const status = finished ? 'completed' : 'pending';
      let filename = `${status}_events`;

      // 根据格式选择相应的API
      switch (format) {
        case 'ics':
          // 调用ICS格式的状态导出API
          exportContent = await invoke<string>('export_events_by_status', { status :finished, fmt :"ics" });
          break;
        case 'json':
          // 调用JSON格式的状态导出API
          exportContent = await invoke<string>('export_events_by_status', { status: finished, fmt: "json" });
          break;
        case 'md':
          // 调用Markdown格式的状态导出API
          exportContent = await invoke<string>('export_events_by_status', {  status: finished, fmt: "md" });
          break;
        default:
          throw new Error(`不支持的导出格式: ${format}`);
      }

      // 将导出内容保存到文件
      const filePath = await invoke<string>('save_export_file', {
        content: exportContent,
        filename,
        format,
        customPath // 添加自定义路径参数
      });

      return filePath;
    } catch (error) {
      console.error(`导出${finished ? '已完成' : '未完成'}事件失败`, error);
      throw error;
    }
  }
}