import { FList } from 'src-tauri/bindings/FList';
import { invoke } from '@tauri-apps/api/core';

// 内部存储，使用 UUID 格式的 ID
let listsData: FList[] = [];

/**
 * 获取任务列表数据
 * @returns Promise<List[]> 返回列表数据的Promise
 */
// export async function getLists(): Promise<FList[]> {
//   listsData = await invoke<FList[]>('get_lists');
//   console.log(listsData);
//   return [...listsData];
// }

export async function getLists(): Promise<FList[]> {
  try {
    const result = await invoke<FList[]>('get_lists');
    
    // 验证返回数据类型
    if (!Array.isArray(result)) {
      console.error('后端返回的数据不是数组:', result);
      return [];
    }
    
    listsData = result;
    console.log('获取到的列表数据:', listsData);
    return [...listsData];
  } catch (error) {
    console.error('获取列表失败:', error);
    // 确保总是返回一个数组，避免其他地方出错
    return [];
  }
}

/**
 * 创建新列表
 * @param title 列表标题
 * @param icon 列表图标，默认为清单图标
 * @returns Promise<List[]> 返回更新后的列表数据
 */
export async function createList(title: string, icon: string = 'mdi-format-list-bulleted'): Promise<FList[]> {
  try {
    let newList: FList = await invoke<FList>('new_list', { title: String(title), icon: String(icon) });
    console.log(newList);
    listsData.push(newList);
    console.log(`Service: New list created with ID ${newList.id}`);
  } catch (error) {
    console.error(`Service: Error creating new list - ${error}`);
  }
  return [...listsData];
}

/**
 * 重命名列表
 * @param id 列表ID
 * @param newName 新的列表名称
 * @returns Promise<List[]> 返回更新后的列表数据
 */
export async function renameList(id: string, newTitle: string): Promise<FList[]> {
  const listItem = listsData.find(l => l.id === id);
  if (listItem) {
    listItem.title = newTitle;
    console.log(`Service: List ${id} renamed to ${newTitle}`);
  } else {
    console.error(`Service: List ${id} not found for renaming`);
  }

  return [...listsData];
}

/**
 * 删除列表
 * @param id 列表ID
 * @returns Promise<List[]> 返回更新后的列表数据
 */
export async function deleteList(id: string): Promise<FList[]> {
  const index = listsData.findIndex(l => l.id === id);
  if (index !== -1) {
    listsData.splice(index, 1);
    console.log(`Service: List ${id} deleted`);
  } else {
    console.error(`Service: List ${id} not found for deletion`);
  }

  return [...listsData];
}