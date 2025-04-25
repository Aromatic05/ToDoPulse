import { v4 as uuidv4 } from 'uuid'; // 需要先安装：npm install uuid @types/uuid

// 列表项接口定义
export interface ListItem {
    id: string;
    title: string;
    icon: string;
}

// 内部存储，使用 UUID 格式的 ID
let listsData: ListItem[] = [
    { id: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1', title: '工作', icon: 'mdi-briefcase' },
    { id: 'c7d8f418-39c1-4c0b-9c1d-2f3e8ea5968a', title: '个人', icon: 'mdi-account' }
];

/**
 * 获取任务列表数据
 * @returns Promise<ListItem[]> 返回列表数据的Promise
 */
export async function getLists(): Promise<ListItem[]> {
    return [...listsData];
}

/**
 * 创建新列表
 * @param title 列表标题
 * @param icon 列表图标，默认为清单图标
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function createList(title: string, icon: string = 'mdi-format-list-bulleted'): Promise<ListItem[]> {
    const newList: ListItem = {
        id: uuidv4(), // 生成新的UUID
        title,
        icon
    };
    
    listsData.push(newList);
    console.log(`Service: New list created with ID ${newList.id}`);
    
    return [...listsData];
}

/**
 * 创建新列表并存储到localStorage
 * @param title 列表标题
 * @param icon 列表图标
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function createListAndStore(title: string, icon: string): Promise<ListItem[]> {
    try {
        // 获取当前列表
        const currentLists = await getLists();
        
        // 创建新列表项
        const newList: ListItem = {
            id: uuidv4(),
            title,
            icon,
            // 可以添加其他必要的属性
        };
        
        // 添加到列表中
        const updatedLists = [...currentLists, newList];
        
        // 存储更新后的列表
        localStorage.setItem('lists', JSON.stringify(updatedLists));
        
        return updatedLists;
    } catch (error) {
        console.error('创建列表失败:', error);
        throw error;
    }
}

/**
 * 重命名列表
 * @param id 列表ID
 * @param newName 新的列表名称
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function renameList(id: string, newName: string): Promise<ListItem[]> {
    const listItem = listsData.find(l => l.id === id);
    if (listItem) {
        listItem.title = newName;
        console.log(`Service: List ${id} renamed to ${newName}`);
    } else {
        console.error(`Service: List ${id} not found for renaming`);
    }
    
    return [...listsData];
}

/**
 * 删除列表
 * @param id 列表ID
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function deleteList(id: string): Promise<ListItem[]> {
    const index = listsData.findIndex(l => l.id === id);
    if (index !== -1) {
        listsData.splice(index, 1);
        console.log(`Service: List ${id} deleted`);
    } else {
        console.error(`Service: List ${id} not found for deletion`);
    }
    
    return [...listsData];
}