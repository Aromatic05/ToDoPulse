import { reactive } from 'vue';

// 定义类型接口 - 移除了dateGroup字段
export interface TimelineItem {
    id: number;
    title: string;
    description: string;
    time: string;
    color: string;
    icon: string;
    isCompleted: boolean;
}

export interface EventCardData {
    id: number;
    title: string;
    content: string;
    date: string;
    dateColor: string;
    isCompleted: boolean;
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

    // 所有时间线项目数据 - 改为按组分类的二维数组
    private timelineItems = reactive<GroupedTimelineItems>({
        'today': [
            {
                id: 1,
                title: '完成项目规划',
                description: '讨论并确定项目范围、目标和里程碑',
                time: '今天 09:00',
                color: 'primary',
                icon: 'mdi-file-document',
                isCompleted: true
            },
            {
                id: 2,
                title: '团队会议',
                description: '周会：讨论本周工作进展和问题',
                time: '今天 14:00',
                color: 'secondary',
                icon: 'mdi-account-group',
                isCompleted: false
            }
        ],
        'tomorrow': [
            {
                id: 3,
                title: '提交代码审查',
                description: '提交新功能的代码审查请求',
                time: '明天 10:30',
                color: 'success',
                icon: 'mdi-code-tags',
                isCompleted: false
            },
            {
                id: 4,
                title: 'UI设计评审',
                description: '评审新界面设计和用户体验改进',
                time: '明天 15:00',
                color: 'info',
                icon: 'mdi-palette',
                isCompleted: false
            }
        ],
        'next-week': [
            {
                id: 5,
                title: '项目进度汇报',
                description: '向管理层汇报项目进展情况',
                time: '下周一 11:00',
                color: 'warning',
                icon: 'mdi-chart-timeline',
                isCompleted: false
            },
            {
                id: 6,
                title: '产品发布准备',
                description: '准备产品发布材料和营销内容',
                time: '下周三 09:30',
                color: 'error',
                icon: 'mdi-rocket-launch',
                isCompleted: false
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
        return {
            id: item.id,
            title: item.title,
            content: item.description,
            date: item.time,
            dateColor: this.getColorVariable(item.color),
            isCompleted: item.isCompleted || false,
            tags: [dateGroup] // 现在需要从外部传入dateGroup
        }
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
            // 从EventCard格式转回timeline格式
            items[index] = {
                ...items[index],
                title: updatedData.title,
                description: updatedData.content,
                isCompleted: updatedData.isCompleted
            };
        }
    }
    
    // 添加新项目
    addItem(item: Omit<TimelineItem, 'id'>, dateGroup: string): number {
        if (!this.timelineItems[dateGroup]) {
            this.timelineItems[dateGroup] = [];
        }
        
        // 查找所有组中的最大ID
        const allIds = Object.values(this.timelineItems)
            .flat()
            .map(item => item.id);
        const newId = Math.max(0, ...allIds) + 1;
        
        const newItem = { ...item, id: newId };
        this.timelineItems[dateGroup].push(newItem);
        return newId;
    }
    
    // 删除项目
    deleteItem(id: number, dateGroup: string): boolean {
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
    moveItem(id: number, fromGroup: string, toGroup: string): boolean {
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
        
        // 添加到目标组
        this.timelineItems[toGroup].push(item);
        return true;
    }
}

// 创建单例实例
const timelineService = new TimelineDataService();
export default timelineService;