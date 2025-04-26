import { reactive } from 'vue';
import { v4 as uuidv4 } from 'uuid';

// 定义类型接口 - 按照新的定义
export interface TimelineItem {
    id: string;
    title: string;
    isCompleted: boolean;
    content: string;
    priority: '高' | '中' | '低';
    time: string;
    date: string;
    dateColor: string;
    color: string;
    icon: string;
    listId: string;
    tags: string[];
}

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

export interface TimelineGroup {
    id: string;
    title: string;
    iconName: string;
    color: string;
    dateGroup: string;
}

// 定义按组分类的数据集合类型
export interface GroupedTimelineItems {
    [key: string]: TimelineItem[];
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
    private timelineItems = reactive<GroupedTimelineItems>({
        'today': [
            {
                id: uuidv4(),
                title: '完成项目规划',
                content: '讨论并确定项目范围、目标和里程碑',
                priority: '高',
                time: '09:00',
                date: '2025-04-26',
                dateColor: 'var(--md-sys-color-primary)',
                color: 'primary',
                icon: 'mdi-file-document',
                isCompleted: true,
                listId: this.LIST_IDS.WORK,
                tags: ['today', 'work', 'high-priority']
            },
            {
                id: uuidv4(),
                title: '团队会议',
                content: '周会：讨论本周工作进展和问题',
                priority: '中',
                time: '14:00',
                date: '2025-04-26',
                dateColor: 'var(--md-sys-color-secondary)',
                color: 'secondary',
                icon: 'mdi-account-group',
                isCompleted: false,
                listId: this.LIST_IDS.MEETINGS,
                tags: ['today', 'meeting']
            }
        ],
        'tomorrow': [
            {
                id: uuidv4(),
                title: '提交代码审查',
                content: '提交新功能的代码审查请求',
                priority: '中',
                time: '10:30',
                date: '2025-04-27',
                dateColor: 'var(--md-sys-color-success)',
                color: 'success',
                icon: 'mdi-code-tags',
                isCompleted: false,
                listId: this.LIST_IDS.WORK,
                tags: ['tomorrow', 'code-review']
            },
            {
                id: uuidv4(),
                title: 'UI设计评审',
                content: '评审新界面设计和用户体验改进',
                priority: '中',
                time: '15:00',
                date: '2025-04-27',
                dateColor: 'var(--md-sys-color-info)',
                color: 'info',
                icon: 'mdi-palette',
                isCompleted: false,
                listId: this.LIST_IDS.DESIGN,
                tags: ['tomorrow', 'design']
            }
        ],
        'next-week': [
            {
                id: uuidv4(),
                title: '项目进度汇报',
                content: '向管理层汇报项目进展情况',
                priority: '高',
                time: '11:00',
                date: '2025-04-28',
                dateColor: 'var(--md-sys-color-warning)',
                color: 'warning',
                icon: 'mdi-chart-timeline',
                isCompleted: false,
                listId: this.LIST_IDS.MEETINGS,
                tags: ['next-week', 'report']
            },
            {
                id: uuidv4(),
                title: '产品发布准备',
                content: '准备产品发布材料和营销内容',
                priority: '低',
                time: '09:30',
                date: '2025-04-30',
                dateColor: 'var(--md-sys-color-error)',
                color: 'error',
                icon: 'mdi-rocket-launch',
                isCompleted: false,
                listId: this.LIST_IDS.MARKETING,
                tags: ['next-week', 'release']
            }
        ]
    });

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

    // 获取今天的事项 - 直接返回对应分组的数组
    getTodayItems(): TimelineItem[] {
        return this.timelineItems['today'] || [];
    }

    // 获取明天的事项
    getTomorrowItems(): TimelineItem[] {
        return this.timelineItems['tomorrow'] || [];
    }

    // 获取下周的事项
    getNextWeekItems(): TimelineItem[] {
        return this.timelineItems['next-week'] || [];
    }

    // 获取指定组的事项
    getItemsByGroup(dateGroup: string): TimelineItem[] {
        return this.timelineItems[dateGroup] || [];
    }

    // 将timeline数据格式转换为EventCard所需的格式
    formatCardData(item: TimelineItem, dateGroup: string): EventCardData {
        // 确保tags包含dateGroup
        const tags = [...(item.tags || [])];
        if (!tags.includes(dateGroup)) {
            tags.push(dateGroup);
        }
        // if (!tags.includes(item.listId) && item.listId) {
        //     tags.push(item.listId);
        // }

        return {
            ...item,
            tags
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
    updateItem(updatedData: EventCardData, dateGroup: string): void {
        const items = this.timelineItems[dateGroup];
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
    addItem(item: Omit<TimelineItem, 'id'>, dateGroup: string): string {
        if (!this.timelineItems[dateGroup]) {
            this.timelineItems[dateGroup] = [];
        }
        
        // 生成唯一UUID
        const newId = uuidv4();
        
        const newItem: TimelineItem = { ...item, id: newId };
        this.timelineItems[dateGroup].push(newItem);
        return newId;
    }
    
    // 删除项目
    deleteItem(id: string, dateGroup: string): boolean {
        const items = this.timelineItems[dateGroup];
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
        const fromItems = this.timelineItems[fromGroup];
        if (!fromItems) return false;
        
        const index = fromItems.findIndex(item => item.id === id);
        if (index === -1) return false;
        
        // 确保目标组存在
        if (!this.timelineItems[toGroup]) {
            this.timelineItems[toGroup] = [];
        }
        
        // 复制项目并从源组删除
        const item = { ...fromItems[index] };
        fromItems.splice(index, 1);
        
        // 更新标签，添加新分组标签并移除旧分组标签
        const tags = item.tags.filter(tag => tag !== fromGroup);
        if (!tags.includes(toGroup)) {
            tags.push(toGroup);
        }
        item.tags = tags;
        
        // 添加到目标组
        this.timelineItems[toGroup].push(item);
        return true;
    }
    
    // 根据列表ID筛选项目
    getItemsByList(listId: string): TimelineItem[] {
        return Object.values(this.timelineItems)
            .flat()
            .filter(item => item.listId === listId);
    }
    
    // 获取所有的列表ID
    getListIds(): string[] {
        const allListIds = new Set<string>();
        
        Object.values(this.timelineItems)
            .flat()
            .forEach(item => {
                if (item.listId) {
                    allListIds.add(item.listId);
                }
            });
            
        return Array.from(allListIds);
    }
    
    // 按优先级排序项目
    sortItemsByPriority(items: TimelineItem[]): TimelineItem[] {
        const priorityWeight = {
            '高': 3,
            '中': 2,
            '低': 1
        };
        
        return [...items].sort((a, b) => 
            priorityWeight[b.priority] - priorityWeight[a.priority]
        );
    }
}

// 创建单例实例
const timelineService = new TimelineDataService();
export default timelineService;