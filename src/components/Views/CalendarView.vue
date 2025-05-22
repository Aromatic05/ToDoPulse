<template>
  <div class="calendar-container">
    <!-- 新增移动端返回按钮 -->
    <div v-if="isMobile" class="mobile-back-button">
      <button @click="goBack">
        <span class="back-icon">&#8592;</span> 返回列表
      </button>
    </div>
    
    <div class="calendar-sidebar" >
      <div class="calendar-sidebar-section">
      </div>
      <div class="calendar-sidebar-section">
        <h2>所有事件 ({{ currentEvents.length }})</h2>
        <ul  class="event-list">
          <li v-for="event in currentEvents" :key="event.id">
            <span class="event-time">{{ event.startStr }}</span>
            <span class="event-title">{{ event.title }}</span>
          </li>
        </ul>
      </div>
    </div>
    <div class="calendar-main">
      <FullCalendar ref="fullCalendar" class="calendar" :options="calendarOptions">
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
import { defineComponent, ref, onMounted, onUnmounted } from 'vue';
import FullCalendar from '@fullcalendar/vue3';
import dayGridPlugin from '@fullcalendar/daygrid';
import timeGridPlugin from '@fullcalendar/timegrid';
import interactionPlugin from '@fullcalendar/interaction';
import type { EventApi, CalendarOptions, DateSelectArg, EventClickArg } from '@fullcalendar/core'; 
import { invoke } from '@tauri-apps/api/core';
import type { FEvent } from 'src-tauri/bindings/FEvent';
import { timestampToDate } from '@/services/DateTimeService';
// import { useEventStore } from '@/stores';

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
    FullCalendar
  },
  setup() {
    // 使用eventStore
    // const eventStore = useEventStore();
    const fullCalendar = ref<InstanceType<typeof FullCalendar> | null>(null);

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
        detail: { route: 'lists' }  // 默认返回到时间线页面
        }));
    };

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
          console.log("日历API状态:", fullCalendar.value.getApi());
          
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
      console.log("当前日期范围:", range);
      if (!range) {
        console.error("无法获取当前日期范围");
        return;
      }

      const dates = iter_calendar(range.start, range.end);
      console.log(`正在获取 ${dates.length} 天的事件数据...`);
      
      const events: FEvent[] = [];
      for (const date of dates) {
        try {
          // filter_events 返回一个数组，不是单个事件
          const response:FEvent[] = await invoke('filter_events', { filter:date });
          // 将该日期的所有事件添加到总事件数组中
          events.push(...response);
        } catch (error) {
          console.error("Error fetching events:", error);
        }
      }
      console.log("获取到的事件:", events);
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
              allDay: true, // 考虑是否需要根据时间设置为false
              backgroundColor: event.color !== "default" ? event.color : undefined,
              borderColor: event.color !== "default" ? event.color : undefined,
              textColor: '#fff', // 可以根据背景颜色自动调整
              classNames: event.finished ? ['event-completed'] : [],
              extendedProps: {
                listid: event.listid,
                tags: event.tag,
                finished: event.finished,
                priority: event.priority,
                icon: event.icon
              }
            });
            
            // 打印调试信息，帮助诊断事件是否正确添加
            console.log(`已添加事件: ${event.title}, 开始: ${startDate?.toISOString() || '无'}, 结束: ${endDate?.toISOString() || '无'}`);
          } else {
            console.warn(`事件 ${event.title} (ID: ${event.id}) 没有有效的日期，无法显示在日历上`);
          }
        }
      }
    };

    // 处理日期选择
    // 使用从 @fullcalendar/core 导入的类型

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
      currentEvents.value = events;
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
      currentEvents,
      fullCalendar,
      isMobile,
      goBack
    };
  }
});
</script>

<style scoped>
@media (max-width: 767px) {
  .calendar-sidebar-section:nth-child(2) {
    display: none;
  }
}

.calendar-container {
  display: flex;
  /* 移除固定高度，允许内容自然流动 */
  min-height: calc(100vh - 64px);
  /* 改为最小高度 */
  width: 100%;
  font-family: var(--md-sys-typescale-body-large-font);
  font-size: var(--md-sys-typescale-body-large-size);
  color: var(--md-sys-color-on-surface);
  overflow: visible;
  /* 允许内容溢出 */
  flex-direction: column;
  /* 在小屏幕上垂直堆叠 */
}

/* 在大屏幕上保持水平布局 */
@media (min-width: 768px) {
  .calendar-container {
    flex-direction: row;
  }
}

.calendar-sidebar {
  width: 100%;
  /* 在小屏幕上占满宽度 */
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
  height: auto;
  /* 从 100% 改为 auto，让内容决定高度 */
  min-height: 600px;
  /* 确保有最小高度 */
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
</style>