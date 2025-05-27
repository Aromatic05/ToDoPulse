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
          <li
            v-for="event in currentEvents"
            :key="event.id"
            @click="handleSidebarEventClick(event)"
            class="event-list-item"
          >
            <span class="event-time">{{ event.startStr }}</span>
            <span class="event-title">{{ event.title }}</span>
          </li>
        </ul>
      </div>
    </div>

    <div class="calendar-main" :class="{ 'mobile-calendar': isMobile }">
      <FullCalendar
        ref="fullCalendar"
        class="calendar"
        :options="calendarOptions"
      >
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

    <!-- 添加新建事件模态框 -->
    <AddCardModal
      v-model="showAddCardModal"
      :initial-date="newEventDate"
      @confirm="handleNewEvent"
    />

    <!-- 添加日历事件上下文菜单 -->
    <CalendarContextMenu
      v-model:show="showContextMenu"
      :activator-element="contextMenuActivator"
      :target-event="contextMenuEvent"
      @toggle-completion="handleToggleCompletion"
      @delete="handleEventDelete"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, onUnmounted, computed, defineAsyncComponent } from "vue";
import FullCalendar from "@fullcalendar/vue3";
import dayGridPlugin from "@fullcalendar/daygrid";
import timeGridPlugin from "@fullcalendar/timegrid";
import interactionPlugin from "@fullcalendar/interaction";
import type {
  EventApi,
  CalendarOptions,
  DateSelectArg,
  EventClickArg,
} from "@fullcalendar/core";
import { invoke } from "@tauri-apps/api/core";
import type { FEvent } from "src-tauri/bindings/FEvent";
import { timestampToDate } from "@/services/DateTimeService";
import AddCardModal from "@/components/Modals/AddCardModal.vue";
import CalendarContextMenu from "@/components/Menus/CalendarContextMenu.vue";
import { useEventStore } from "@/stores/eventStore";
import { useListStore } from "@/stores/listStore";

const CardContentModal = defineAsyncComponent(() =>
  import("@/components/Modals/CardContentModal.vue")
);

// 将日期范围转换为日期字符串数组
const iter_calendar = (start: Date, end: Date) => {
  const dates: string[] = [];
  const currentDate = new Date(start);
  while (currentDate <= end) {
    const year = currentDate.getFullYear();
    const month = String(currentDate.getMonth() + 1).padStart(2, "0");
    const day = String(currentDate.getDate()).padStart(2, "0");
    const formattedDate = `${year}-${month}-${day}`;
    dates.push(formattedDate);
    currentDate.setDate(currentDate.getDate() + 1);
  }
  return dates;
};

export default defineComponent({
  name: "CalendarView",
  components: {
    FullCalendar,
    CardContentModal,
    AddCardModal,
    CalendarContextMenu,
  },
  setup() {
    // 使用eventStore
    const eventStore = useEventStore();
    const fullCalendar = ref<InstanceType<typeof FullCalendar> | null>(null);

    // 新增状态管理
    const showAddCardModal = ref(false);
    const newEventDate = ref<{ start: Date; end: Date } | null>(null);

    // 添加用于控制模态框的状态
    const showEventModal = ref(false);
    const selectedEvent = ref<FEvent | null>(null);

    // 上下文菜单状态
    const showContextMenu = ref(false);
    const contextMenuActivator = ref<HTMLElement | null>(null);
    const contextMenuEvent = ref<EventApi | null>(null);
    const longPressTimer = ref<number | null>(null);
    const touchedEvent = ref<EventApi | null>(null);

    const getCurrentRange = () => {
      if (fullCalendar.value) {
        const view = fullCalendar.value.getApi().view;
        return {
          start: view.currentStart,
          end: view.currentEnd,
        };
      }
      return null;
    };
    const currentEvents = ref<EventApi[]>([]);

    // 添加移动设备检测
    const isMobile = ref(false);

    // 检测设备是否为移动设备
    const checkMobile = () => {
      isMobile.value = window.innerWidth < 768;
    };

    // 返回函数
    const goBack = () => {
      window.dispatchEvent(
        new CustomEvent("navigation", {
          detail: { route: "lists" },
        })
      );
    };

    // 日历配置
    const calendarOptions = computed((): CalendarOptions => {
      // 基础配置
      const config: CalendarOptions = {
        plugins: [dayGridPlugin, timeGridPlugin, interactionPlugin],
        editable: true,
        selectable: true,
        selectMirror: true,
        dayMaxEvents: true,
        weekends: true,
        select: handleDateSelect,
        eventClick: handleEventClick, // 直接使用handleEventClick
        eventsSet: handleEvents,
        height: "auto",
        contentHeight: "auto",
        locale: "zh-cn",
        buttonText: {
          today: "今天",
          month: "月",
          week: "周",
          day: "日",
        },
      };

      // 根据设备类型调整配置
      if (isMobile.value) {
        // 移动端简化配置
        config.headerToolbar = {
          left: "prev,next",
          center: "title",
          right: "today",
        };
        config.initialView = "dayGridMonth";
        config.aspectRatio = 1.3; // 更紧凑的比例
        config.dayMaxEventRows = 2; // 每天最多显示2行事件，其余折叠
        config.selectable = true;
        config.selectMirror = true;
        config.longPressDelay = 500; // 添加长按延迟，优化移动端体验
        // 移除日期范围显示
        config.views = {
          dayGrid: {
            dayHeaderFormat: { weekday: "short" }, // 简化头部显示，只显示星期几
            selectable: true, // 确保视图级别也启用选择
          },
        };
      } else {
        // 桌面端完整配置
        config.headerToolbar = {
          left: "prev,next today",
          center: "title",
          right: "dayGridMonth,timeGridWeek,timeGridDay",
        };
        config.initialView = "dayGridMonth";
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
      window.addEventListener("resize", checkMobile);

      // 等待日历组件初始化完成
      setTimeout(() => {
        if (fullCalendar.value) {
          console.log("日历初始化完成，设置事件监听");

          // 监听日期范围变化事件
          fullCalendar.value.getApi().on("datesSet", async (arg) => {
            console.log("日期范围变化:", arg.start, arg.end);
            await loadCalendarEvents();
          });

          // 初始加载一次
          loadCalendarEvents();
        } else {
          console.warn("日历组件未能正确初始化");
        }
      }, 100); // 给予组件一点时间完全初始化

      // 添加全局事件监听器，确保菜单在适当时关闭
      window.addEventListener("click", (e) => {
        if (showContextMenu.value) {
          const target = e.target as HTMLElement;
          if (!target.closest(".v-menu")) {
            showContextMenu.value = false;
          }
        }
      });
    });

    onUnmounted(() => {
      // 移除事件监听器
      window.removeEventListener("resize", checkMobile);
      if (longPressTimer.value) {
        clearTimeout(longPressTimer.value);
      }
    });

    // 根据优先级获取颜色
    const getPriorityColor = (priority: string): string => {
      switch (priority) {
        case "High":
          return "#e53935"; // 红色
        case "Medium":
          return "#fb8c00"; // 橙色
        case "Low":
          return "#43a047"; // 绿色
        default:
          return "#1e88e5"; // 默认蓝色
      }
    };

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
          const response: FEvent[] = await invoke("filter_events", {
            filter: date,
          });
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
              backgroundColor:
                event.color !== "default"
                  ? event.color
                  : getPriorityColor(event.priority),
              borderColor:
                event.color !== "default"
                  ? event.color
                  : getPriorityColor(event.priority),
              textColor: "#fff",
              classNames: event.finished ? ["event-completed"] : [],
              extendedProps: {
                listid: event.listid,
                tags: event.tag,
                finished: event.finished,
                priority: event.priority,
                icon: event.icon,
                originalEvent: event, // 存储原始FEvent对象以便稍后使用
              },
            });
          }
        }

        // 添加事件元素的触摸事件监听
        setTimeout(() => {
          if (fullCalendar.value) {
            const eventElements =
              fullCalendar.value.$el.querySelectorAll(".fc-event");
            eventElements.forEach((el: HTMLElement) => {
              const eventId =
                el.getAttribute("data-event-id") ||
                (el as any).fcSeg?.eventRange?.def?.publicId;
              if (eventId) {
                const event = fullCalendar
                  .value!.getApi()
                  .getEventById(eventId);
                if (event) {
                  el.addEventListener("touchstart", (e: TouchEvent) => {
                    handleEventTouchStart({ event, el } as EventClickArg, e);
                  });
                  el.addEventListener("touchend", handleEventTouchEnd);
                  el.addEventListener("touchcancel", handleEventTouchEnd);
                  el.addEventListener("touchmove", handleEventTouchMove);

                  // 添加右键菜单支持
                  el.addEventListener("contextmenu", (e: MouseEvent) => {
                    e.preventDefault();
                    contextMenuActivator.value = e.target as HTMLElement;
                    contextMenuEvent.value = event;
                    showContextMenu.value = true;
                  });
                }
              }
            });
          }
        }, 200);
      }
    };

    // 处理日期选择
    const handleDateSelect = (selectInfo: DateSelectArg) => {
      newEventDate.value = {
        start: selectInfo.start,
        end: selectInfo.end,
      };
      showAddCardModal.value = true;
      selectInfo.view.calendar.unselect();
    };

    // 处理事件点击 - 替换为打开模态框
    const handleEventClick = (clickInfo: EventClickArg) => {
      // 获取点击的事件的原始数据
      const originalEvent = clickInfo.event.extendedProps
        .originalEvent as FEvent;

      if (originalEvent) {
        // 设置选中的事件并显示模态框
        selectedEvent.value = { ...originalEvent };
        showEventModal.value = true;
      } else {
        console.error("无法获取事件详情");
      }

      // 添加长按事件监听
      const eventEl = clickInfo.el;
      eventEl.addEventListener("touchstart", (e) =>
        handleEventTouchStart(clickInfo, e as TouchEvent)
      );
      eventEl.addEventListener("touchend", handleEventTouchEnd);
      eventEl.addEventListener("touchcancel", handleEventTouchEnd);
      eventEl.addEventListener("touchmove", handleEventTouchMove);
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

    // 处理新增事件
    const handleNewEvent = async (newEvent: FEvent) => {
      try {
        if (newEventDate.value) {
          // 设置截止日期为选定日期的结束时间
          const endDate = new Date(newEventDate.value.end);
          endDate.setHours(23, 59, 59, 999);
          newEvent.ddl = endDate.getTime().toString();
          newEvent.create = Date.now().toString();
        }

        await eventStore.addEvent(
          newEvent.listid,
          newEvent.title,
          newEvent.priority,
          newEvent.ddl
        );
        await loadCalendarEvents();
        showAddCardModal.value = false;
      } catch (error) {
        console.error("创建事件失败:", error);
      }
    };

    // 获取列表数据
    const listStore = useListStore();

    // 处理事件长按开始
    const handleEventTouchStart = (info: EventClickArg, e: TouchEvent) => {
      touchedEvent.value = info.event;
      longPressTimer.value = window.setTimeout(() => {
        const touch = e.touches[0];
        const touchTarget = document.elementFromPoint(
          touch.clientX,
          touch.clientY
        ) as HTMLElement;
        contextMenuActivator.value = touchTarget;
        contextMenuEvent.value = info.event;
        showContextMenu.value = true;
        e.preventDefault(); // 阻止默认行为
      }, 800); // 长按800毫秒触发
    };

    // 处理事件长按结束
    const handleEventTouchEnd = () => {
      if (longPressTimer.value) {
        clearTimeout(longPressTimer.value);
        longPressTimer.value = null;
      }
      touchedEvent.value = null;
    };

    // 处理移动中取消长按
    const handleEventTouchMove = () => {
      handleEventTouchEnd(); // 移动时取消长按计时
    };

    // 处理事件完成状态切换
    const handleToggleCompletion = async (event: EventApi) => {
      const originalEvent = event.extendedProps.originalEvent as FEvent;
      if (originalEvent) {
        try {
          // 切换完成状态
          originalEvent.finished = !originalEvent.finished;
          await eventStore.updateEvent(originalEvent);
          await loadCalendarEvents(); // 重新加载以更新UI
        } catch (error) {
          console.error("更新事件状态失败:", error);
        }
      }
    };

    // 处理事件删除
    const handleEventDelete = async (event: EventApi) => {
      const originalEvent = event.extendedProps.originalEvent as FEvent;
      if (originalEvent) {
        try {
          await eventStore.deleteEvent(originalEvent.id, originalEvent.listid);
          await loadCalendarEvents(); // 重新加载以更新UI
        } catch (error) {
          console.error("删除事件失败:", error);
        }
      }
    };

    return {
      calendarOptions,
      currentEvents,
      fullCalendar,
      isMobile,
      goBack,
      showEventModal,
      selectedEvent,
      showAddCardModal,
      newEventDate,
      handleEventUpdate,
      handleSidebarEventClick,
      handleNewEvent,
      lists: listStore.lists,
      showContextMenu,
      contextMenuActivator,
      contextMenuEvent,
      handleToggleCompletion,
      handleEventDelete,
    };
  },
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
  padding-top: 60px;
  /* 为顶部返回按钮留出空间 */
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
