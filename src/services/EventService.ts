import { FEvent } from 'src-tauri/bindings/FEvent';
import { Priority } from 'src-tauri/bindings/Priority';
import { FList } from 'src-tauri/bindings/FList';
import { invoke } from '@tauri-apps/api/core';

/**
 * 根据列表ID获取事件
 * @param listid 列表ID
 * @returns Promise<FEvent[]> 返回事件列表
 */
export async function getEventsBylistid(listid: string): Promise<FEvent[]> {
    try {
        const listEvents = await invoke<FEvent[]>('list_content', { listid: listid });
    return listEvents;
    } catch (error) {    
        console.error('getEventsBylistid获取列表失败:', error);  
        return [];
    }
    
}

/**
 * 添加新事件
 * @param listid 列表ID
 * @param title 事件标题
 * @param timestamp 事件时间戳
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function addEvent(
    listid: string,
    title: string,
    priority: Priority = "Medium",
    timestamp: string = Date.now().toString(),
): Promise<FEvent[]> {
    try {
        const lists = await invoke<FList[]>('get_lists');
        if (lists.find(l => l.id === listid) === undefined) {
            console.error(`列表ID ${listid} 不存在`);
            return [];
        }
        // 此处的参数不代表真实情况，请自行修改
        invoke('add_event', { listid: listid, title: title, priority: priority, ddl: timestamp })
        return invoke('list_content', { listid :listid });
    } catch (error) {
        console.error('获取列表失败:', error);  
        return [];
    }
}

/**
 * 切换事件完成状态
 * @param EventId 事件ID
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function updateEvent(
    fEvent : FEvent,
): Promise<FEvent[]> {
    if (fEvent) {
        invoke( 'put_event', { fEvent });
        return invoke('list_content', { listid :fEvent.listid });
    } else {
        console.error('Service: updateEvent: Event not found');
        return [];
    }
}

/**
 * 删除事件
 * @param EventId 事件ID
 * @param ListId 列表ID
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function deleteEvent(EventId: string, ListId: string): Promise<FEvent[]> {
    invoke('delete_event', { uuid: EventId });
    return invoke<FEvent[]>('list_content', { listid: ListId });
}

export async function getEventContent(EventId: string): Promise<string> {
    console.log(EventId);
    return invoke<string>('event_content', { uuid: EventId });
}

export async function putEventContent(EventId: string, content: string): Promise<string> {
    invoke('write_content', { uuid: EventId, content: content });
    return invoke<string>('event_content', { uuid: EventId });
}