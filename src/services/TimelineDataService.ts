import { reactive } from 'vue';
import { v4 as uuidv4 } from 'uuid';

import { FEvent } from 'src-tauri/bindings/FEvent';
import { Priority } from 'src-tauri/bindings/Priority';

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

// 时间线组标题数据
const timelineGroups = reactive<TimelineGroup[]>([
    {
        id: 'today',
        title: "今天",
        iconName: "mdi-calendar-today",
        color: "primary",
        dateGroup: "today"
    },
    {
        id: 'tomorrow',
        title: "明天",
        iconName: "mdi-calendar-arrow-right",
        color: "secondary",
        dateGroup: "tomorrow"
    },
    {
        id: 'next-week',
        title: "下周",
        iconName: "mdi-calendar-week",
        color: "info",
        dateGroup: "next-week"
    }
]);

// 所有时间线项目数据 - 修改为使用UUID作为listId
const FEvents: Record<string, FEvent[]> = {
    "today": [
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
            listid: "4788325718170490349",
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
            listid: "4788325718170490349",
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
            listid: "4788325718170490349",
            tag: ['客户', '邮件']
        }
    ],

    // 个人列表的事件
    "tomorrow": [
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
    ], 
    "next-week": [
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
export function getItemsByGroup(dateGroup: string): FEvent[] {
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
export function updateItem(updatedData: FEvent, dateGroup: string): void {
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