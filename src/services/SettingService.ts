import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';

interface Event {
  id: string;
  title: string;
  create: string;
  status: string;
  finished: boolean;
}

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
  static async getExportableEvents(): Promise<Event[]> {
    try {
      // 调用后端API获取所有事件列表
      const events = await invoke<any[]>('get_all_events');
      
      // 将后端数据格式转换为前端需要的格式
      return events.map(event => ({
        id: event.metadata.uuid,
        title: event.title,
        create: new Date(event.metadata.timestamp).toLocaleString(),
        status: event.finished ? '已完成' : '未完成',
        finished: event.finished
      }));
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
      // 首先获取所有事件
      const events = await this.getExportableEvents();
      const eventIds = events.map(event => event.id);
      
      // 导出这些事件
      return await this.exportEvents(eventIds, format, customPath);
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
  static async exportEvents(eventIds: string[], format: string, customPath?: string): Promise<string> {
    try {
      if (eventIds.length === 0) {
        throw new Error('没有选择任何事件');
      }

      let exportContent = '';
      let filename = format === 'ics' ? 'todopulse_events' : 'todopulse_events';

      // 根据不同格式调用不同的后端导出API
      switch (format) {
        case 'ics':
          // 使用ICS导出API
          exportContent = await invoke<string>('export_events_to_ics', { uuids: eventIds });
          break;
        case 'json':
          // 这里可以添加JSON导出API
          throw new Error('JSON导出功能尚未实现');
        case 'markdown':
          // 这里可以添加Markdown导出API
          throw new Error('Markdown导出功能尚未实现');
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

      // 可以选择打开文件所在目录
      await open(filePath.substring(0, filePath.lastIndexOf('/')));

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
      
      // 根据格式调用相应API
      switch (format) {
        case 'ics':
          exportContent = await invoke<string>('export_event_to_ics', { uuid: eventId });
          break;
        default:
          throw new Error(`不支持的导出格式: ${format}`);
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
}