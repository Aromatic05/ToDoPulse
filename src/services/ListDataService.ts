import { v4 as uuidv4 } from 'uuid';
import { FEvent } from 'src-tauri/bindings/FEvent';
import { Priority } from 'src-tauri/bindings/Priority';

// export type FEvent = { 
//     id: string, 
//     time: string, 
//     date: string, 
//     listid: string, 
//     tag: Array<string> | null, 
//     title: string, 
//     create: string, 
//     finished: boolean, 
//     priority: Priority, 
//     icon: string, 
//     color: string, 
// };


// 模拟事件数据，按列表ID组织
const eventsData: Record<string, FEvent[]> = {
    // 工作列表的事件
    "1": [
        {
            id: uuidv4(),
            title: '完成项目方案',
            finished: false,
            priority: "High",
            date: '2025-04-28',
            time: '18:00',
            create: "2025-04-20",
            color: '#f1c40f',
            icon: 'work',
            listid: "1",
            tag: ['项目', '文档'],
        },
        {
            id: uuidv4(),
            title: '准备周会演示',
            finished: true,
            priority: "Medium",
            date: '2025-04-26',
            time: '10:00',
            create: "2025-04-20",
            color: '#3498db',
            icon: 'presentation',
            listid: "1",
            tag: ['会议', '演示']
        },
        {
            id: uuidv4(),
            title: '回复客户邮件',
            finished: false,
            priority: "High",
            date: '2025-04-25',
            time: '14:30',
            create: "2025-04-20",
            color: '#e74c3c',
            icon: 'email',
            listid: "1",
            tag: ['客户', '邮件']
        }
    ],

    // 个人列表的事件
    "2": [
        {
            id: uuidv4(),
            title: '更新项目文档',
            finished: false,
            priority: "Low",
            date: '2025-04-30',
            time: '12:00',
            create: "2025-04-20",
            color: '#9b59b6',
            icon: 'document',
            listid: "2",
            tag: ['文档', '更新']
        },
        {
            id: uuidv4(),
            title: '购买生日礼物',
            finished: false,
            priority: "Medium",
            date: '2025-05-05',
            time: '15:00',
            create: "2025-04-20",
            color: '#1abc9c',
            icon: 'gift',
            listid: "2",
            tag: ['生日', '购物']
        }
    ]
};

/**
 * 根据列表ID获取事件
 * @param listid 列表ID
 * @returns Promise<FEvent[]> 返回事件列表
 */
export async function getEventsBylistid(listid: string): Promise<FEvent[]> {
    // 返回指定列表的事件，如果列表不存在则返回空数组
    console.log(listid);
    return [...(eventsData[listid] || [])];
}

/**
 * 添加新事件
 * @param listid 列表ID
 * @param title 事件标题
 * @param date 日期
 * @param time 时间
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function addEvent(
    listid: string,
    title: string,
    priority: Priority = "Medium",
    date: string = new Date().toISOString().substring(0, 10),
    time: string = '12:00',
    create: string = new Date().toISOString().substring(0, 10)
): Promise<FEvent[]> {
    // 确保该列表的事件数组存在
    if (!eventsData[listid]) {
        eventsData[listid] = [];
    }

    const newEvent: FEvent = {
        id: uuidv4(),
        title,
        finished: false,
        priority,
        date,
        time,
        create,
        color: '#3498db',
        icon: 'Event',
        listid,
        tag: [],
    };

    eventsData[listid].push(newEvent);
    console.log(`Service: New event "${title}" added to list ${listid}`);

    return [...eventsData[listid]];
}

/**
 * 切换事件完成状态
 * @param EventId 事件ID
 * @param listid 列表ID
 * @param finished 完成状态
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function toggleEventStatus(
    EventId: string,
    listid: string,
    finished: boolean
): Promise<FEvent[]> {
    const Events = eventsData[listid] || [];
    const Event = Events.find(t => t.id === EventId);

    if (Event) {
        Event.finished = finished;
        console.log(`Service: Event "${Event.title}" status changed to: ${finished ? '已完成' : '未完成'}`);
    }

    return [...Events];
}

/**
 * 编辑事件
 * @param EventId 事件ID
 * @param listid 列表ID
 * @param updates 要更新的字段
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function updateEvent(
    EventId: string,
    listid: string,
    updates: Partial<Omit<FEvent, 'id' | 'listid'>>
): Promise<FEvent[]> {
    const Events = eventsData[listid] || [];
    const Event = Events.find(t => t.id === EventId);

    if (Event) {
        // 保证finished不会被设置为undefined
        const safeUpdates = {
            ...updates,
            finished: updates.finished === undefined ? Event.finished : Boolean(updates.finished)
        };

        Object.assign(Event, safeUpdates);
        console.log(`Service: Event "${Event.title}" updated, finished: ${Event.finished}`);
    }

    return [...Events];
}

/**
 * 删除事件
 * @param EventId 事件ID
 * @param listid 列表ID
 * @returns Promise<FEvent[]> 返回更新后的事件列表
 */
export async function deleteEvent(EventId: string, listid: string): Promise<FEvent[]> {
    const Events = eventsData[listid] || [];
    const index = Events.findIndex(t => t.id === EventId);

    if (index !== -1) {
        const [removedEvent] = Events.splice(index, 1);
        console.log(`Service: Event "${removedEvent.title}" deleted`);
    }

    return [...Events];
}