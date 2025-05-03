import { reactive, ref}from 'vue';

import { FEvent } from 'src-tauri/bindings/FEvent';
import { Priority } from 'src-tauri/bindings/Priority';
import { invoke } from '@tauri-apps/api/core';

export interface TimelineGroup {
    id: string;
    title: string;
    iconName: string;
    color: string;
    dateGroup: string;
}

// 定义按组分类的数据集合类型
export interface GroupedFEvents {
    [key: string]: FEvent[];
}

const timeMap = {
    TODAY: 'today',
    TOMORROW: 'tomorrow',
    NEXT_WEEK: 'next-week',
} as const;

// 时间线组标题数据
const timelineGroups = reactive<TimelineGroup[]>([
    {
        id: timeMap.TODAY,
        title: "今天",
        iconName: "mdi-calendar-today",
        color: "primary",
        dateGroup: timeMap.TODAY
    },
    {
        id: timeMap.TOMORROW,
        title: "明天",
        iconName: "mdi-calendar-arrow-right",
        color: "secondary",
        dateGroup: timeMap.TOMORROW
    },
    {
        id: timeMap.NEXT_WEEK,
        title: "下周",
        iconName: "mdi-calendar-week",
        color: "info",
        dateGroup: timeMap.NEXT_WEEK
    }
]);

// 所有时间线项目数据 - 修改为使用UUID作为listId
const FEvents: Record<string, FEvent[]> = {};

// 添加一个标志来跟踪数据是否已加载
const dataInitialized = ref(false);
const dataLoadingPromise = ref<Promise<void> | null>(null);

// 获取数据是否已加载
export async function fetchFEvents(): Promise<void> {
    if (dataLoadingPromise.value) {
        return dataLoadingPromise.value;
    }
    const loadPromise = Promise.all(
      Object.keys(timeMap).map(async (time) => {
        const dateGroup = timeMap[time as keyof typeof timeMap];
        try {
            const events = await invoke('filter_events', { dateGroup });
            FEvents[dateGroup] = events as FEvent[];
        } catch (error) {
            console.error(`Error fetching events for ${dateGroup}:`, error);
            FEvents[dateGroup] = [];
        }
      })
    ).then(() => {
        dataInitialized.value = true;
    });
    
    dataLoadingPromise.value = loadPromise;
    return loadPromise;
}

// 一个确保数据已加载的辅助函数
async function ensureDataLoaded(): Promise<void> {
  if (!dataInitialized.value) {
    await fetchFEvents();
  }
}

// 添加一个检查是否已初始化的导出函数
export function isDataInitialized(): boolean {
  return dataInitialized.value;
}

// 获取时间线组
export function getTimelineGroups(): TimelineGroup[] {
    return timelineGroups;
}

// 获取指定ID的时间线组
export function getTimelineGroup(id: string): TimelineGroup | undefined {
    return timelineGroups.find(group => group.id === id);
}

// 更新时间线组标题或其他属性
export function updateTimelineGroup(id: string, updates: Partial<TimelineGroup>): boolean {
    const index = timelineGroups.findIndex(group => group.id === id);
    if (index !== -1) {
        timelineGroups[index] = { ...timelineGroups[index], ...updates };
        return true;
    }
    return false;
}

// 获取指定组的事项
export async function getItemsByGroup(dateGroup: string): Promise<FEvent[]> {
    await ensureDataLoaded();
    return FEvents[dateGroup] || [];
}

// 将timeline数据格式转换为EventCard所需的格式
export function formatCardData(item: FEvent, dateGroup: string): FEvent {
    // 确保tags包含dateGroup
    const tag = [...(item.tag || [])];
    if (!tag.includes(dateGroup)) {
        tag.push(dateGroup);
    }

    return {
        ...item,
        tag
    };
}

// 将颜色名称转换为CSS变量
export function getColorVariable(color: string): string {
    const colorMap: Record<string, string> = {
        'primary': 'var(--md-sys-color-primary)',
        'secondary': 'var(--md-sys-color-secondary)',
        'info': 'var(--md-sys-color-tertiary)',
        'success': 'var(--md-sys-color-success)',
        'warning': 'var(--md-sys-color-warning)',
        'error': 'var(--md-sys-color-error)'
    }
    return colorMap[color] || 'var(--md-sys-color-on-surface-variant)'
}

// // 更新项目
export async function updateItem(updatedData: FEvent, dateGroup: string): Promise<void> {
    await ensureDataLoaded();
    const items = FEvents[dateGroup];
    if (!items) return;
    
    const index = items.findIndex(item => item.id === updatedData.id);
    if (index !== -1) {
        // 更新项目
        items[index] = {
            ...items[index],
            ...updatedData
        };
    }
}

// // 根据列表ID筛选项目
// export function getItemsByList(listId: string): FEvent[] {
//     return Object.values(FEvents)
//         .flat()
//         .filter(item => item.listid === listId);
// }

// // 获取所有的列表ID
// export function getListIds(): string[] {
//     const allListIds = new Set<string>();
    
//     Object.values(FEvents)
//         .flat()
//         .forEach(item => {
//             if (item.listid) {
//                 allListIds.add(item.listid);
//             }
//         });
        
//     return Array.from(allListIds);
// }

// 按优先级排序项目
export function sortItemsByPriority(items: FEvent[]): FEvent[] {
    const priority: Record<Priority, number> = {
        'High': 3,
        'Medium': 2,
        'Low': 1,
        'Undefined': 0
    };
    
    return [...items].sort((a, b) => {
        // 安全地获取优先级权重，如果不存在则默认为0
        const weightA = priority[a.priority] ?? 0;
        const weightB = priority[b.priority] ?? 0;
        return weightB - weightA;
    });
}