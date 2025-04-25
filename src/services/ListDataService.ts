import { v4 as uuidv4 } from 'uuid';

// 任务项接口
export interface TaskItem {
    id: string;
    title: string;
    completed: boolean;
    priority: '高' | '中' | '低';
    dueDate: string;
    listId: string; // 关联的列表ID
}

// 模拟任务数据，按列表ID组织
const tasksData: Record<string, TaskItem[]> = {
    // 工作列表的任务
    '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1': [
        { 
            id: uuidv4(), 
            title: '完成项目方案', 
            completed: false, 
            priority: '高', 
            dueDate: '2025-04-28',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1'
        },
        { 
            id: uuidv4(), 
            title: '准备周会演示', 
            completed: true, 
            priority: '中', 
            dueDate: '2025-04-26',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1'
        },
        { 
            id: uuidv4(), 
            title: '回复客户邮件', 
            completed: false, 
            priority: '高', 
            dueDate: '2025-04-25',
            listId: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1'
        }
    ],
    
    // 个人列表的任务
    'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a': [
        { 
            id: uuidv4(), 
            title: '更新项目文档', 
            completed: false, 
            priority: '低', 
            dueDate: '2025-04-30',
            listId: 'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a'
        },
        { 
            id: uuidv4(), 
            title: '购买生日礼物', 
            completed: false, 
            priority: '中', 
            dueDate: '2025-05-05',
            listId: 'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a'
        }
    ]
};

/**
 * 根据列表ID获取任务
 * @param listId 列表ID
 * @returns Promise<TaskItem[]> 返回任务列表
 */
export async function getTasksByListId(listId: string): Promise<TaskItem[]> {
    // 返回指定列表的任务，如果列表不存在则返回空数组
    return [...(tasksData[listId] || [])];
}

/**
 * 添加新任务
 * @param listId 列表ID
 * @param title 任务标题
 * @param priority 优先级
 * @param dueDate 截止日期
 * @returns Promise<TaskItem[]> 返回更新后的任务列表
 */
export async function addTask(
    listId: string,
    title: string,
    priority: '高' | '中' | '低' = '中',
    dueDate: string = new Date().toISOString().substring(0, 10)
): Promise<TaskItem[]> {
    // 确保该列表的任务数组存在
    if (!tasksData[listId]) {
        tasksData[listId] = [];
    }
    
    const newTask: TaskItem = {
        id: uuidv4(),
        title,
        completed: false,
        priority,
        dueDate,
        listId
    };
    
    tasksData[listId].push(newTask);
    console.log(`Service: New task "${title}" added to list ${listId}`);
    
    return [...tasksData[listId]];
}

/**
 * 切换任务完成状态
 * @param taskId 任务ID
 * @param listId 列表ID
 * @param completed 完成状态
 * @returns Promise<TaskItem[]> 返回更新后的任务列表
 */
export async function toggleTaskStatus(
    taskId: string,
    listId: string,
    completed: boolean
): Promise<TaskItem[]> {
    const tasks = tasksData[listId] || [];
    const task = tasks.find(t => t.id === taskId);
    
    if (task) {
        task.completed = completed;
        console.log(`Service: Task "${task.title}" status changed to: ${completed ? '已完成' : '未完成'}`);
    }
    
    return [...tasks];
}

/**
 * 编辑任务
 * @param taskId 任务ID
 * @param listId 列表ID
 * @param updates 要更新的字段
 * @returns Promise<TaskItem[]> 返回更新后的任务列表
 */
export async function updateTask(
    taskId: string,
    listId: string,
    updates: Partial<Omit<TaskItem, 'id' | 'listId'>>
): Promise<TaskItem[]> {
    const tasks = tasksData[listId] || [];
    const task = tasks.find(t => t.id === taskId);
    
    if (task) {
        Object.assign(task, updates);
        console.log(`Service: Task "${task.title}" updated`);
    }
    
    return [...tasks];
}

/**
 * 删除任务
 * @param taskId 任务ID
 * @param listId 列表ID
 * @returns Promise<TaskItem[]> 返回更新后的任务列表
 */
export async function deleteTask(taskId: string, listId: string): Promise<TaskItem[]> {
    const tasks = tasksData[listId] || [];
    const index = tasks.findIndex(t => t.id === taskId);
    
    if (index !== -1) {
        const [removedTask] = tasks.splice(index, 1);
        console.log(`Service: Task "${removedTask.title}" deleted`);
    }
    
    return [...tasks];
}