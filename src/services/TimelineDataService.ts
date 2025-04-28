import { reactive } from 'vue';
import { v4 as uuidv4 } from 'uuid';

import { FEvent } from 'src-tauri/bindings/FEvent';

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

class TimelineDataService {
    // 列表ID常量 - 使用UUID替代简单字符串
    private readonly LIST_IDS = {
        WORK: '93a36bf8-e3f0-4e2d-8b3c-5fec9d399ef1',
        MEETINGS: '7d8e6f45-9c0b-4a2d-8e7f-5a6b3c9d8e7f',
        DESIGN: 'f1e2d3c4-b5a6-7890-1234-567890abcdef',
        MARKETING: 'a1b2c3d4-e5f6-7890-1234-567890fedcba'
    };

    // 时间线组标题数据
    private timelineGroups = reactive<TimelineGroup[]>([
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
    private FEvents: Record<string, FEvent[]> = {
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
    getTimelineGroups(): TimelineGroup[] {
        return this.timelineGroups;
    }

    // 获取指定ID的时间线组
    getTimelineGroup(id: string): TimelineGroup | undefined {
        return this.timelineGroups.find(group => group.id === id);
    }

    // 更新时间线组标题或其他属性
    updateTimelineGroup(id: string, updates: Partial<TimelineGroup>): boolean {
        const index = this.timelineGroups.findIndex(group => group.id === id);
        if (index !== -1) {
            this.timelineGroups[index] = { ...this.timelineGroups[index], ...updates };
            return true;
        }
        return false;
    }

    // 获取指定组的事项
    getItemsByGroup(dateGroup: string): FEvent[] {
        return this.FEvents[dateGroup] || [];
    }

    // 将timeline数据格式转换为EventCard所需的格式
    formatCardData(item: FEvent, dateGroup: string): FEvent {
        // 确保tags包含dateGroup
        const tag = [...(item.tag || [])];
        if (!tag.includes(dateGroup)) {
            tag.push(dateGroup);
        }
        // if (!tags.includes(item.listId) && item.listId) {
        //     tags.push(item.listId);
        // }

        return {
            ...item,
            tag
        };
    }

    // 将颜色名称转换为CSS变量
    getColorVariable(color: string): string {
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

    // 更新项目
    updateItem(updatedData: FEvent, dateGroup: string): void {
        const items = this.FEvents[dateGroup];
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
    
    // 添加新项目 - 使用UUID作为ID
    addItem(item: Omit<FEvent, 'id'>, dateGroup: string): string {
        if (!this.FEvents[dateGroup]) {
            this.FEvents[dateGroup] = [];
        }
        
        // 生成唯一UUID
        const newId = uuidv4();
        
        const newItem: FEvent = { ...item, id: newId };
        this.FEvents[dateGroup].push(newItem);
        return newId;
    }
    
    // 删除项目
    deleteItem(id: string, dateGroup: string): boolean {
        const items = this.FEvents[dateGroup];
        if (!items) return false;
        
        const index = items.findIndex(item => item.id === id);
        if (index !== -1) {
            items.splice(index, 1);
            return true;
        }
        return false;
    }
    
    // 跨分组移动项目
    moveItem(id: string, fromGroup: string, toGroup: string): boolean {
        const fromItems = this.FEvents[fromGroup];
        if (!fromItems) return false;
        
        const index = fromItems.findIndex(item => item.id === id);
        if (index === -1) return false;
        
        // 确保目标组存在
        if (!this.FEvents[toGroup]) {
            this.FEvents[toGroup] = [];
        }
        
        // 复制项目并从源组删除
        const item = { ...fromItems[index] };
        fromItems.splice(index, 1);
        
        // 更新标签，添加新分组标签并移除旧分组标签
        const tag = (item.tag || []).filter(tag => tag !== fromGroup);
        if (!tag.includes(toGroup)) {
            tag.push(toGroup);
        }
        item.tag = tag;
        
        // 添加到目标组
        this.FEvents[toGroup].push(item);
        return true;
    }
    
    // 根据列表ID筛选项目
    getItemsByList(listId: string): FEvent[] {
        return Object.values(this.FEvents)
            .flat()
            .filter(item => item.listid === listId);
    }
    
    // 获取所有的列表ID
    getListIds(): string[] {
        const allListIds = new Set<string>();
        
        Object.values(this.FEvents)
            .flat()
            .forEach(item => {
                if (item.listid) {
                    allListIds.add(item.listid);
                }
            });
            
        return Array.from(allListIds);
    }
    
    // 按优先级排序项目
    sortItemsByPriority(items: FEvent[]): FEvent[] {
        const priorityWeight = {
            'High': 3,
            'Medium': 2,
            'Low': 1
        };
        
        return [...items].sort((a, b) => 
            priorityWeight[b.priority] - priorityWeight[a.priority]
        );
    }
}

// 创建单例实例
const timelineService = new TimelineDataService();
export default timelineService;