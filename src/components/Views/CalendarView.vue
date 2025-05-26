<template>
  <div class="calendar-container">    
    <!-- 侧边栏 - 只在非移动端显示 -->
    <div v-if="!isMobile" class="calendar-sidebar">
      <div class="calendar-sidebar-section">
        <h2>日历视图</h2>
        <div class="calendar-view-controls">
          <!-- 日历控制元素 -->
        </div>
      </div>
      <div class="calendar-sidebar-section">
        <h2>所有事件 ({{ currentEvents.length }})</h2>
        <ul class="event-list">
          <li v-for="event in currentEvents" :key="event.id" 
              @click="handleSidebarEventClick(event)"
              class="event-list-item">
            <span class="event-time">{{ event.startStr }}</span>
            <span class="event-title">{{ event.title }}</span>
          </li>
        </ul>
      </div>
    </div>
    
    <div class="calendar-main" :class="{ 'mobile-calendar': isMobile }">
      <FullCalendar ref="fullCalendar" class="calendar" :options="calendarOptions">
        <template v-slot:eventContent="arg">
          <div class="event-content">
            <span class="event-time">{{ arg.timeText }}</span>
            <span class="event-title">{{ arg.event.title }}</span>
          </div>
        </template>
      </FullCalendar>
    </div>
    
    <!-- 添加事件详情模态框 -->
    <CardContentModal
      v-if="showEventModal && selectedEvent"
      v-model="showEventModal"
      :card-data="selectedEvent"
      @confirm="handleEventUpdate"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, onUnmounted, computed } from 'vue';
import FullCalendar from '@fullcalendar/vue3';
import dayGridPlugin from '@fullcalendar/daygrid';
import timeGridPlugin from '@fullcalendar/timegrid';
import interactionPlugin from '@fullcalendar/interaction';
import type { EventApi, CalendarOptions, DateSelectArg, EventClickArg } from '@fullcalendar/core'; 
import { invoke } from '@tauri-apps/api/core';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { timestampToDate } from '@/services/DateTimeService';
import CardContentModal from '@/components/Modals/CardContentModal.vue';
import { useEventStore } from '@/stores';

// 将日期范围转换为日期字符串数组
const iter_calendar=(start:Date, end:Date) => {
  const dates:string[] = [];
  const currentDate = new Date(start);
  while (currentDate <= end) {
    const year = currentDate.getFullYear();
    const month = String(currentDate.getMonth() + 1).padStart(2, '0');
    const day = String(currentDate.getDate()).padStart(2, '0');
    const formattedDate = `${year}-${month}-${day}`;
    dates.push(formattedDate);
    currentDate.setDate(currentDate.getDate() + 1);
  }
  return dates;
}

export default defineComponent({
  name: 'CalendarView',
  components: {
    FullCalendar,
    CardContentModal
  },
  setup() {
    // 使用eventStore
    const eventStore = useEventStore();
    const fullCalendar = ref<InstanceType<typeof FullCalendar> | null>(null);
    
    // 添加用于控制模态框的状态
    const showEventModal = ref(false);
    const selectedEvent = ref<FEvent | null>(null);

    const getCurrentRange = () => {
      if (fullCalendar.value) {
        const view = fullCalendar.value.getApi().view;
        return {
          start: view.currentStart,
          end: view.currentEnd
        };
      }
      return null;
    }
    const currentEvents = ref<EventApi[]>([]);

    // 添加移动设备检测
    const isMobile = ref(false);
    
    // 检测设备是否为移动设备
    const checkMobile = () => {
      isMobile.value = window.innerWidth < 768;
    };
    
    // 返回函数
    const goBack = () => {
      window.dispatchEvent(new CustomEvent('navigation', {
        detail: { route: 'lists' }
      }));
    };

    // 根据设备类型动态计算日历配置
    const dynamicCalendarConfig = computed(() => {
      // 基础配置
      const config: CalendarOptions = {
        plugins: [dayGridPlugin, timeGridPlugin, interactionPlugin],
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
        locale: 'zh-cn',
        buttonText: {
          today: '今天',
          month: '月',
          week: '周',
          day: '日'
        }
      };
      
      // 根据设备类型调整配置
      if (isMobile.value) {
        // 移动端简化配置
        config.headerToolbar = {
          left: 'prev,next',
          center: 'title',
          right: 'today'
        };
        config.initialView = 'dayGridMonth';
        config.aspectRatio = 1.3; // 更紧凑的比例
        config.dayMaxEventRows = 2; // 每天最多显示2行事件，其余折叠
        // 移除日期范围显示
        config.views = {
          dayGrid: {
            dayHeaderFormat: { weekday: 'short' } // 简化头部显示，只显示星期几
          }
        };
      } else {
        // 桌面端完整配置
        config.headerToolbar = {
          left: 'prev,next today',
          center: 'title',
          right: 'dayGridMonth,timeGridWeek,timeGridDay'
        };
        config.initialView = 'dayGridMonth';
        config.aspectRatio = 2.2;
      }
      
      return config;
    });

    // 在组件挂载时加载所有事件
    onMounted(() => {
      console.log("日历组件挂载中...");
      // 初始检测设备类型
      checkMobile();
      
      // 监听窗口大小变化
      window.addEventListener('resize', checkMobile);
      
      // 等待日历组件初始化完成
      setTimeout(() => {
        if (fullCalendar.value) {
          console.log("日历初始化完成，设置事件监听");
          
          // 监听日期范围变化事件
          fullCalendar.value.getApi().on('datesSet', async (arg) => {
            console.log("日期范围变化:", arg.start, arg.end);
            await loadCalendarEvents();
          });
          
          // 初始加载一次
          loadCalendarEvents();
        } else {
          console.warn("日历组件未能正确初始化");
        }
      }, 100); // 给予组件一点时间完全初始化
    });

    onUnmounted(() => {
      // 移除事件监听器
      window.removeEventListener('resize', checkMobile);
    });

    // 加载日历事件
    const loadCalendarEvents = async () => {
      const range = getCurrentRange();
      if (!range) {
        console.error("无法获取当前日期范围");
        return;
      }

      const dates = iter_calendar(range.start, range.end);
      console.log(`正在获取 ${dates.length} 天的事件数据...`);
      
      const events: FEvent[] = [];
      for (const date of dates) {
        try {
          const response:FEvent[] = await invoke('filter_events', { filter:date });
          events.push(...response);
        } catch (error) {
          console.error("Error fetching events:", error);
        }
      }
      
      if (fullCalendar.value) {
        const calendarApi = fullCalendar.value.getApi();
        // 清除现有事件以防止重复
        calendarApi.removeAllEvents();
        
        for (const event of events) {
          // 使用转换函数将时间戳字符串转换为日期对象
          const startDate: Date | undefined = timestampToDate(event.create);
          const endDate: Date | undefined = timestampToDate(event.ddl);
          
          // 只有当至少有一个日期有效时才添加事件
          if (startDate || endDate) {
            calendarApi.addEvent({
              id: event.id,
              title: event.title,
              start: startDate,
              end: endDate,
              allDay: true, 
              backgroundColor: event.color !== "default" ? event.color : undefined,
              borderColor: event.color !== "default" ? event.color : undefined,
              textColor: '#fff',
              classNames: event.finished ? ['event-completed'] : [],
              extendedProps: {
                listid: event.listid,
                tags: event.tag,
                finished: event.finished,
                priority: event.priority,
                icon: event.icon,
                originalEvent: event // 存储原始FEvent对象以便稍后使用
              }
            });
          }
        }
      }
    };

    const handleDateSelect = (selectInfo: DateSelectArg) => {
      const title = prompt('请输入事件标题');
      const calendarApi = selectInfo.view.calendar;

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
      }
    };

    // 处理事件点击 - 替换为打开模态框
    const handleEventClick = (clickInfo: EventClickArg) => {
      // 获取点击的事件的原始数据
      const originalEvent = clickInfo.event.extendedProps.originalEvent as FEvent;
      
      if (originalEvent) {
        // 设置选中的事件并显示模态框
        selectedEvent.value = { ...originalEvent };
        showEventModal.value = true;
      } else {
        console.error("无法获取事件详情");
      }
    };
    
    // 添加从侧边栏点击事件的处理函数
    const handleSidebarEventClick = (event: EventApi) => {
      const originalEvent = event.extendedProps.originalEvent as FEvent;
      
      if (originalEvent) {
        selectedEvent.value = { ...originalEvent };
        showEventModal.value = true;
      }
    };

    // 处理事件更新
    const handleEventUpdate = async (updatedEvent: FEvent) => {
      try {
        // 使用eventStore更新事件
        await eventStore.updateEvent(updatedEvent);
        
        // 重新加载日历事件以反映变更
        await loadCalendarEvents();
        
        // 关闭模态框
        showEventModal.value = false;
      } catch (error) {
        console.error("更新事件失败:", error);
      }
    };

    // 处理事件集合变化
    const handleEvents = (events: EventApi[]) => {
      currentEvents.value = events;
    };

    return {
      calendarOptions: dynamicCalendarConfig,
      currentEvents,
      fullCalendar,
      isMobile,
      goBack,
      showEventModal,
      selectedEvent,
      handleEventUpdate,
      handleSidebarEventClick
    };
  }
});
</script>

<style scoped>
.calendar-container {
  display: flex;
  min-height: calc(100vh - 64px);
  width: 100%;
  font-family: var(--md-sys-typescale-body-large-font);
  font-size: var(--md-sys-typescale-body-large-size);
  color: var(--md-sys-color-on-surface);
  overflow: visible;
  flex-direction: column;
}

/* 在大屏幕上保持水平布局 */
@media (min-width: 768px) {
  .calendar-container {
    flex-direction: row;
  }
}

.calendar-sidebar {
  width: 280px;
  padding: 1rem;
  background: var(--md-sys-color-surface-container);
  border-right: 1px solid var(--md-sys-color-outline);
  height: auto;
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
  overflow: visible;
  height: auto;
}

/* 移动端日历容器额外样式 */
.mobile-calendar {
  padding-top: 60px; /* 为顶部返回按钮留出空间 */
}

.calendar {
  height: auto;
  min-height: 500px;
  background: var(--md-sys-color-surface);
  border-radius: 8px;
  box-shadow: var(--md-sys-elevation-1);
}

/* 移动端日历调整最小高度 */
@media (max-width: 767px) {
  .calendar {
    min-height: 400px;
  }
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
  --fc-button-active-border-color: var(--md-sys-color-primary-container);
  --fc-button-active-text-color: var(--md-sys-color-on-primary-container);
  --fc-button-hover-bg-color: var(--md-sys-color-primary-container);
  --fc-button-hover-border-color: var(--md-sys-color-primary-container);
  --fc-button-hover-text-color: var(--md-sys-color-on-primary-container);
  --fc-event-bg-color: var(--md-sys-color-primary-container);
  --fc-event-border-color: var(--md-sys-color-primary-container);
  --fc-event-text-color: var(--md-sys-color-on-primary-container);
  --fc-today-bg-color: var(--md-sys-color-surface-container-high);
}

/* 移动端日历样式调整 */
@media (max-width: 767px) {
  :deep(.fc .fc-toolbar) {
    flex-wrap: wrap;
    gap: 8px;
  }
  
  :deep(.fc .fc-toolbar-title) {
    font-size: 1.2em;
  }
  
  :deep(.fc .fc-button) {
    padding: 0.25em 0.5em;
    font-size: 0.9em;
  }
  
  /* 简化日期标题 */
  :deep(.fc .fc-daygrid-day-top) {
    justify-content: center;
  }
  
  :deep(.fc-daygrid-day-number) {
    font-size: 0.9em;
    padding: 2px;
  }
}

/* 已完成事件的样式 */
:deep(.event-completed) {
  opacity: 0.7;
  text-decoration: line-through;
}

/* 移动端返回按钮样式 */
.mobile-back-button {
  position: fixed;
  top: 16px;
  left: 16px;
  z-index: 100;
}

.mobile-back-button button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background-color: var(--md-sys-color-primary);
  color: var(--md-sys-color-on-primary);
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  box-shadow: var(--md-sys-elevation-1);
}

.back-icon {
  font-size: 18px;
}

.mobile-back-button button:hover {
  background-color: var(--md-sys-color-primary-container);
  color: var(--md-sys-color-on-primary-container);
}

/* 添加事件列表项的可点击样式 */
.event-list-item {
  cursor: pointer;
  transition: background-color 0.2s;
}

.event-list-item:hover {
  background-color: var(--md-sys-color-surface-container-high);
}
</style>