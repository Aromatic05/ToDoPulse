import { v4 as uuidv4 } from 'uuid';

// 事件卡片数据接口
export interface EventCardData {
    id: string;
    title: string;
    isCompleted: boolean;
    content: string;
    priority?: '高' | '中' | '低'; 
    time: string;
    date: string;
    dateColor: string;
    color: string;
    icon: string;
    listId: string;
    tags: string[];
}

// 模拟事件数据，按列表ID组织
const tasksData: Record<string, EventCardData[]> = {
    // 工作列表的事件
    '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1': [
        { 
            id: uuidv4(), 
            title: '完成项目方案', 
            isCompleted: false, 
            content: '准备项目方案文档及相关材料',
            priority: '高', 
            date: '2025-04-28',
            time: '18:00',
            dateColor: '#ff4757',
            color: '#f1c40f',
            icon: 'work',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1',
            tags: ['项目', '文档']
        },
        { 
            id: uuidv4(), 
            title: '准备周会演示', 
            isCompleted: true, 
            content: '准备周会PPT和演示材料',
            priority: '中', 
            date: '2025-04-26',
            time: '10:00',
            dateColor: '#2ecc71',
            color: '#3498db',
            icon: 'presentation',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1',
            tags: ['会议', '演示']
        },
        { 
            id: uuidv4(), 
            title: '回复客户邮件', 
            isCompleted: false, 
            content: '回复关于项目进展的客户邮件',
            priority: '高', 
            date: '2025-04-25',
            time: '14:30',
            dateColor: '#ff4757',
            color: '#e74c3c',
            icon: 'email',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1',
            tags: ['客户', '邮件']
        }
    ],
    
    // 个人列表的事件
    'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a': [
        { 
            id: uuidv4(), 
            title: '更新项目文档', 
            isCompleted: false, 
            content: '更新项目相关文档和说明',
            priority: '低', 
            date: '2025-04-30',
            time: '12:00',
            dateColor: '#3498db',
            color: '#9b59b6',
            icon: 'document',
            listId: 'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a',
            tags: ['文档', '更新']
        },
        { 
            id: uuidv4(), 
            title: '购买生日礼物', 
            isCompleted: false, 
            content: '为朋友购买生日礼物',
            priority: '中', 
            date: '2025-05-05',
            time: '18:00',
            dateColor: '#2ecc71',
            color: '#1abc9c',
            icon: 'gift',
            listId: 'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a',
            tags: ['生日', '购物']
        }
    ]
};

/**
 * 根据列表ID获取事件
 * @param listId 列表ID
 * @returns Promise<EventCardData[]> 返回事件列表
 */
export async function getTasksByListId(listId: bigint): Promise<EventCardData[]> {
    // 返回指定列表的事件，如果列表不存在则返回空数组
    console.log(listId);
    return [...(tasksData[listId.toString()] || [])];
}

/**
 * 添加新事件
 * @param listId 列表ID
 * @param title 事件标题
 * @param content 事件内容
 * @param priority 优先级
 * @param date 日期
 * @param time 时间
 * @returns Promise<EventCardData[]> 返回更新后的事件列表
 */
export async function addTask(
    listId: string,
    title: string,
    content: string = '',
    priority: '高' | '中' | '低' = '中',
    date: string = new Date().toISOString().substring(0, 10),
    time: string = '12:00'
): Promise<EventCardData[]> {
    // 确保该列表的事件数组存在
    if (!tasksData[listId]) {
        tasksData[listId] = [];
    }
    
    const newTask: EventCardData = {
        id: uuidv4(),
        title,
        isCompleted: false,
        content,
        priority,
        date,
        time,
        dateColor: priority === '高' ? '#ff4757' : priority === '中' ? '#2ecc71' : '#3498db',
        color: '#3498db',
        icon: 'task',
        listId,
        tags: []
    };
    
    tasksData[listId].push(newTask);
    console.log(`Service: New event "${title}" added to list ${listId}`);
    
    return [...tasksData[listId]];
}

/**
 * 切换事件完成状态
 * @param taskId 事件ID
 * @param listId 列表ID
 * @param isCompleted 完成状态
 * @returns Promise<EventCardData[]> 返回更新后的事件列表
 */
export async function toggleTaskStatus(
    taskId: string,
    listId: string,
    isCompleted: boolean
): Promise<EventCardData[]> {
    const tasks = tasksData[listId] || [];
    const task = tasks.find(t => t.id === taskId);
    
    if (task) {
        task.isCompleted = isCompleted;
        console.log(`Service: Event "${task.title}" status changed to: ${isCompleted ? '已完成' : '未完成'}`);
    }
    
    return [...tasks];
}

/**
 * 编辑事件
 * @param taskId 事件ID
 * @param listId 列表ID
 * @param updates 要更新的字段
 * @returns Promise<EventCardData[]> 返回更新后的事件列表
 */
export async function updateTask(
    taskId: string,
    listId: string,
    updates: Partial<Omit<EventCardData, 'id' | 'listId'>>
): Promise<EventCardData[]> {
    const tasks = tasksData[listId] || [];
    const task = tasks.find(t => t.id === taskId);
    
    if (task) {
        // 保证isCompleted不会被设置为undefined
        const safeUpdates = {
            ...updates,
            isCompleted: updates.isCompleted === undefined ? task.isCompleted : Boolean(updates.isCompleted)
        };
        
        Object.assign(task, safeUpdates);
        console.log(`Service: Event "${task.title}" updated, isCompleted: ${task.isCompleted}`);
    }
    
    return [...tasks];
}

/**
 * 删除事件
 * @param taskId 事件ID
 * @param listId 列表ID
 * @returns Promise<EventCardData[]> 返回更新后的事件列表
 */
export async function deleteTask(taskId: string, listId: string): Promise<EventCardData[]> {
    const tasks = tasksData[listId] || [];
    const index = tasks.findIndex(t => t.id === taskId);
    
    if (index !== -1) {
        const [removedTask] = tasks.splice(index, 1);
        console.log(`Service: Event "${removedTask.title}" deleted`);
    }
    
    return [...tasks];
}