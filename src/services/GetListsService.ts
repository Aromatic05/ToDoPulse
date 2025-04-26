import { List } from 'src-tauri/bindings/List';
import { invoke } from '@tauri-apps/api/core';

// 列表项接口定义
export interface ListItem {
  id: number;
  title: string;
  icon: string;
}

// 内部存储，使用 UUID 格式的 ID
let listsData: List[] = [];

/**
 * 获取任务列表数据
 * @returns Promise<ListItem[]> 返回列表数据的Promise
 */
export async function getLists(): Promise<List[]> {
  listsData = await invoke<List[]>('get_lists');
  return [...listsData];
}

/**
 * 创建新列表
 * @param title 列表标题
 * @param icon 列表图标，默认为清单图标
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function createList(title: string, icon: string = 'mdi-format-list-bulleted'): Promise<List[]> {
  let newList: List = await invoke<List>('create_list', { title, icon });

  listsData.push(newList);
  console.log(`Service: New list created with ID ${newList.id}`);

  return [...listsData];
}

/**
 * 重命名列表
 * @param id 列表ID
 * @param newName 新的列表名称
 * @returns Promise<ListItem[]> 返回更新后的列表数据
 */
export async function renameList(id: bigint, newName: string): Promise<List[]> {
  const listItem = listsData.find(l => l.id === id);
  if (listItem) {
    listItem.name = newName;
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
export async function deleteList(id: bigint): Promise<List[]> {
  const index = listsData.findIndex(l => l.id === id);
  if (index !== -1) {
    listsData.splice(index, 1);
    console.log(`Service: List ${id} deleted`);
  } else {
    console.error(`Service: List ${id} not found for deletion`);
  }

  return [...listsData];
}