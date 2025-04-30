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
    // 返回指定列表的事件，如果列表不存在则返回空数组
    console.log("getEventsBylistid", listid);
    try {
        const listEvents = await invoke<FEvent[]>('list_content', { listid: listid });
        console.log("listEvents", listEvents);
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
        console.log(`Service: New event "${title}" added to list ${listid}`);
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
    Event : FEvent,
): Promise<FEvent[]> {
    if (Event) {
        console.log("updateEvent", Event);
        invoke( 'put_event', { event: Event });
        return invoke('list_content', { listid :Event.listid });
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
    return invoke<string>('event_content', { uuid: EventId });
}