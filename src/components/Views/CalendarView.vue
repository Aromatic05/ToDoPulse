<template>
    <div class="calendar-container">
        <div class="calendar-sidebar">
            <div class="calendar-sidebar-section">
                <h2>操作说明</h2>
                <ul>
                    <li>点击日期可以创建新事件</li>
                    <li>可以拖拽和调整事件</li>
                    <li>点击事件可以删除</li>
                </ul>
            </div>
            <div class="calendar-sidebar-section">
                <label class="weekend-toggle">
                    <input type="checkbox" :checked="calendarOptions.weekends"
                        @change="calendarOptions.weekends = !calendarOptions.weekends" />
                    显示周末
                </label>
            </div>
            <div class="calendar-sidebar-section">
                <h2>所有事件 ({{ currentEvents.length }})</h2>
                <ul class="event-list">
                    <li v-for="event in currentEvents" :key="event.id">
                        <span class="event-time">{{ event.startStr }}</span>
                        <span class="event-title">{{ event.title }}</span>
                    </li>
                </ul>
            </div>
        </div>
        <div class="calendar-main">
            <FullCalendar class="calendar" :options="calendarOptions">
                <template v-slot:eventContent="arg">
                    <div class="event-content">
                        <span class="event-time">{{ arg.timeText }}</span>
                        <span class="event-title">{{ arg.event.title }}</span>
                    </div>
                </template>
            </FullCalendar>
        </div>
    </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue';
import FullCalendar from '@fullcalendar/vue3';
import dayGridPlugin from '@fullcalendar/daygrid';
import timeGridPlugin from '@fullcalendar/timegrid';
import interactionPlugin from '@fullcalendar/interaction';
import { EventApi, CalendarOptions } from '@fullcalendar/core'; // 添加这行导入
// import { useEventStore } from '@/stores';

export default defineComponent({
    name: 'CalendarView',
    components: {
        FullCalendar
    },
    setup() {
        // 使用eventStore
        // const eventStore = useEventStore();
        
        // 日历事件 - 明确类型
        type CalendarEvent = {
            id: string;
            title: string;
            start: Date | string;
            end?: Date | string;
            allDay?: boolean;
            startStr?: string;
            endStr?: string;
            [key: string]: any;
        };
        
        const currentEvents = ref<CalendarEvent[]>([]);
        
        // 在组件挂载时加载所有事件
        onMounted(async () => {
            // 此处可以从eventStore获取事件数据并转换为日历格式
            await loadCalendarEvents();
        });
        
        // 加载日历事件
        const loadCalendarEvents = async () => {
            // 从eventStore获取所有事件
            // 这里模拟从store获取事件的逻辑
            // 实际项目中应该使用store的方法获取真实数据
            // 例如: await eventStore.fetchAllEvents();
        };
        
        // 处理日期选择
        // 添加类型定义
        interface DateSelectArg {
            start: Date;
            end: Date;
            startStr: string;
            endStr: string;
            allDay: boolean;
            view: {
                calendar: any;
            };
        }
        
        interface EventClickArg {
            event: {
                id: string;
                title: string;
                remove: () => void;
            };
        }
        
        const handleDateSelect = (selectInfo: DateSelectArg) => {
            let title = prompt('请输入事件标题');
            let calendarApi = selectInfo.view.calendar;

            calendarApi.unselect();

            if (title) {
                const newEvent = {
                    id: Date.now().toString(),
                    title,
                    start: selectInfo.startStr,
                    end: selectInfo.endStr,
                    allDay: selectInfo.allDay
                };
                
                // 添加事件到日历
                calendarApi.addEvent(newEvent);
                
                // 同时可以将事件保存到store
                // 例如: eventStore.addEvent(newEvent);
            }
        };
        
        // 处理事件点击
        const handleEventClick = (clickInfo: EventClickArg) => {
            if (confirm(`确定要删除事件"${clickInfo.event.title}"吗？`)) {
                clickInfo.event.remove();
                
                // 同时可以从store中删除事件
                // 例如: eventStore.deleteEvent(clickInfo.event.id);
            }
        };
        
        // 处理事件集合变化
        const handleEvents = (events: EventApi[]) => {
            // 将EventApi类型的数组转换为CalendarEvent类型的数组
            currentEvents.value = events.map(event => ({
                id: event.id,
                title: event.title,
                start: event.start || new Date(),
                end: event.end || undefined, // 将null转换为undefined
                allDay: event.allDay,
                startStr: event.startStr,
                endStr: event.endStr
            }));
        };
        
        // 日历配置
        const calendarOptions: CalendarOptions = {
            plugins: [dayGridPlugin, timeGridPlugin, interactionPlugin],
            headerToolbar: {
                left: 'prev,next today',
                center: 'title',
                right: 'dayGridMonth,timeGridWeek,timeGridDay'
            },
            initialView: 'dayGridMonth',
            editable: true,
            selectable: true,
            selectMirror: true,
            dayMaxEvents: true,
            weekends: true,
            select: handleDateSelect,
            eventClick: handleEventClick,
            eventsSet: handleEvents,
            height: 'auto',
            contentHeight: 'auto',
            aspectRatio: 2.2,
            locale: 'zh-cn',
            buttonText: {
                today: '今天',
                month: '月',
                week: '周',
                day: '日'
            }
        };
        
        return {
            calendarOptions,
            currentEvents
        };
    }
});
</script>

<style scoped>
.calendar-container {
    display: flex;
    /* 移除固定高度，允许内容自然流动 */
    min-height: calc(100vh - 64px); /* 改为最小高度 */
    width: 100%;
    font-family: var(--md-sys-typescale-body-large-font);
    font-size: var(--md-sys-typescale-body-large-size);
    color: var(--md-sys-color-on-surface);
    overflow: visible; /* 允许内容溢出 */
    flex-direction: column; /* 在小屏幕上垂直堆叠 */
}

/* 在大屏幕上保持水平布局 */
@media (min-width: 768px) {
    .calendar-container {
        flex-direction: row;
    }
}

.calendar-sidebar {
    width: 100%; /* 在小屏幕上占满宽度 */
    padding: 1rem;
    background: var(--md-sys-color-surface-container);
    border-right: none;
    border-bottom: 1px solid var(--md-sys-color-outline);
    /* 移除滚动属性 */
    overflow: visible;
    height: auto;
}

@media (min-width: 768px) {
    .calendar-sidebar {
        width: 280px; 
        border-right: 1px solid var(--md-sys-color-outline);
        border-bottom: none;
        height: auto;
    }
}

.calendar-sidebar-section {
    margin-bottom: 2rem;
}

.calendar-sidebar-section h2 {
    font-size: var(--md-sys-typescale-title-medium-size);
    margin-bottom: 1rem;
    color: var(--md-sys-color-on-surface);
}

.calendar-sidebar-section ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.calendar-sidebar-section li {
    margin-bottom: 0.5rem;
    padding: 0.5rem;
    background: var(--md-sys-color-surface-container-low);
    border-radius: 4px;
}

.weekend-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
}

.event-list li {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.event-time {
    font-size: 0.875rem;
    color: var(--md-sys-color-on-surface-variant);
}

.event-title {
    font-weight: 500;
}

.calendar-main {
    flex: 1;
    padding: 1rem;
    /* 移除滚动属性 */
    overflow: visible;
    height: auto;
}

.calendar {
    height: auto; /* 从 100% 改为 auto，让内容决定高度 */
    min-height: 600px; /* 确保有最小高度 */
    background: var(--md-sys-color-surface);
    border-radius: 8px;
    box-shadow: var(--md-sys-elevation-1);
}

.event-content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.25rem;
}

:deep(.fc) {
    --fc-border-color: var(--md-sys-color-outline);
    --fc-button-bg-color: var(--md-sys-color-primary);
    --fc-button-border-color: var(--md-sys-color-primary);
    --fc-button-text-color: var(--md-sys-color-on-primary);
    --fc-button-active-bg-color: var(--md-sys-color-primary-container);
    --fc-button-active-border-color: var (--md-sys-color-primary-container);
    --fc-button-active-text-color: var(--md-sys-color-on-primary-container);
    --fc-button-hover-bg-color: var(--md-sys-color-primary-container);
    --fc-button-hover-border-color: var(--md-sys-color-primary-container);
    --fc-button-hover-text-color: var(--md-sys-color-on-primary-container);
    --fc-event-bg-color: var(--md-sys-color-primary-container);
    --fc-event-border-color: var(--md-sys-color-primary-container);
    --fc-event-text-color: var(--md-sys-color-on-primary-container);
    --fc-today-bg-color: var(--md-sys-color-surface-container-high);
}
</style>