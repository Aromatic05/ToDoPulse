<template>
  <v-card class="header">
    <!-- 标题区域 -->
    <div class="header_title ml-4">
      <!-- 列表相关视图显示 Breadcrumbs -->
      <Breadcrumbs v-if="showBreadcrumbs" :currentPageTitle="viewTitle" />

      <!-- 在其他视图中显示英文标题 -->
      <h2 v-else class="text-h6">{{ viewTitle }}</h2>
    </div>

    <v-spacer></v-spacer>

    <div ref="searchRef" style="width: 320px" class="search_ip mr-2 search-container">
      <v-text-field v-model="searchQuery" rounded variant="outlined" density="compact" label="Search"
        prepend-inner-icon="mdi-magnify" hide-details clearable bg-color="var(--md-sys-color-surface-container)"
        color="var(--md-sys-color-primary)" @update:model-value="handleSearch" @click:clear="clearSearch"
        @focus="showResults = true" @blur="handleBlur"></v-text-field>
    </div>
    <!-- 搜索结果列表 -->
    <Teleport to="body">
      <div v-if="showResults && filteredEvents.length > 0" ref="resultsRef" class="search-results google-style">
        <div class="google-search-results">
          <div v-for="event in filteredEvents" :key="event.id" class="search-result-item"
            @click="handleEventClick(event)" @mouseenter="highlightedIndex = filteredEvents.indexOf(event)" :class="{
              highlighted: highlightedIndex === filteredEvents.indexOf(event),
            }">
            <div class="search-item-icon">
              <v-icon :color="event.color" size="small">{{
                event.icon || "mdi-calendar-check"
                }}</v-icon>
            </div>
            <div class="search-item-content">
              <div class="search-item-title" v-html="highlightMatchText(event.title, searchQuery)"></div>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <div class="tool_btns">
      <ThemePicker />

      <v-btn density="comfortable" variant="text" icon="mdi-cog" @click="toggleSettings" :color="isSettingsActive
          ? 'var(--md-sys-color-primary)'
          : 'var(--md-sys-color-on-surface-variant)'
        ">
      </v-btn>
    </div>

    <!-- 添加卡片详情弹窗 -->
    <CardContentModal v-model="showCardModal" :card-data="selectedEvent" @confirm="handleEventUpdate" />
  </v-card>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  nextTick,
  watch,
  Teleport,
} from "vue";
import { listen } from "@tauri-apps/api/event";
import Breadcrumbs from "@/components/Breadcrumbs.vue";
import ThemePicker from "@/components/ThemePicker.vue";
import CardContentModal from "@/components/Modals/CardContentModal.vue";
import { useEventStore } from "@/stores/eventStore";
import { debounce } from "../utils/helpers";
import type { FEvent } from "src-tauri/bindings/FEvent";

// 定义视图名称的类型
type ViewName = "timeline" | "calendar" | "tags" | "settings" | "default";

const props = defineProps({
  currentView: {
    type: String,
    default: "timeline" as ViewName,
  },
});

const emit = defineEmits(["toggle-settings"]);
const isSettingsActive = ref(false);

// 视图名称映射到英文标题
const viewTitles: Record<ViewName, string> = {
  timeline: "Timeline",
  calendar: "Calendar",
  tags: "Tags",
  settings: "Settings",
  default: "Dashboard",
};

// 计算当前应显示的标题
const viewTitle = computed(() => {
  if (props.currentView.startsWith("list/")) {
    return "List Details";
  }
  // 使用类型断言来确保 TypeScript 理解这是一个有效的键
  return viewTitles[props.currentView as ViewName] || viewTitles.default;
});

// 判断是否显示面包屑
const showBreadcrumbs = computed(() => {
  return props.currentView.startsWith('list/') || 
         props.currentView.startsWith('list-') || 
         props.currentView.startsWith('list-item/');
});

function toggleSettings() {
  isSettingsActive.value = !isSettingsActive.value;
  emit("toggle-settings");
}

// 搜索相关逻辑
const searchQuery = ref("");
const eventStore = useEventStore();
const showResults = ref(false);
const highlightedIndex = ref(-1);
const filteredEvents = ref<FEvent[]>([]);

// 搜索处理函数
const handleSearch = debounce(async (query: string) => {
  if (query?.trim()) {
    // 清空之前的结果，避免累积
    filteredEvents.value = [];
    const result = await eventStore.searchEvents(query);
    filteredEvents.value = result;
    showResults.value = true;
  } else {
    clearSearch();
  }
}, 300);

// 清除搜索框
function clearSearch() {
  searchQuery.value = "";
  eventStore.clearSearchResults();
  filteredEvents.value = []; // 确保本地的结果数组也被清空
  showResults.value = false;
}

// 添加卡片详情弹窗相关状态
const showCardModal = ref(false);
const selectedEvent = ref<FEvent>({
  id: "",
  title: "",
  ddl: "",
  listid: "",
  tag: [],
  create: "",
  finished: false,
  priority: "Low",
  icon: "",
  color: ""
});

// 处理搜索结果项点击
function handleEventClick(event: FEvent) {
  // 设置选中的事件，并显示卡片详情弹窗
  selectedEvent.value = event;
  showCardModal.value = true;
  showResults.value = false;
}

// 处理事件更新
function handleEventUpdate(updatedEvent: FEvent) {
  // 如果需要，可以在这里更新事件状态或刷新数据
  eventStore.updateEvent(updatedEvent);
}

// 处理搜索框失焦
function handleBlur() {
  // 使用延时，以便用户可以点击搜索结果
  setTimeout(() => {
    showResults.value = false;
  }, 200);
}

// 高亮搜索文本中匹配的部分
function highlightMatchText(text: string, query: string): string {
  if (!query.trim()) return text;

  const regex = new RegExp(`(${query.trim()})`, "gi");
  return text.replace(regex, '<span class="highlight">$1</span>');
}

// 手动计算位置
const searchRef = ref<HTMLElement | null>(null);
const resultsRef = ref<HTMLElement | null>(null);

function updateResultsPosition() {
  if (searchRef.value && resultsRef.value) {
    const rect = searchRef.value.getBoundingClientRect();
    resultsRef.value.style.position = "fixed";
    resultsRef.value.style.top = `${rect.bottom}px`;
    resultsRef.value.style.left = `${rect.left}px`;
    resultsRef.value.style.width = `${rect.width}px`;
  }
}

watch(showResults, (newVal) => {
  if (newVal) {
    nextTick(() => {
      updateResultsPosition();
    });
  }
});

// 监听来自托盘菜单的设置切换事件
onMounted(async () => {
  window.addEventListener("resize", () => {
    if (showResults.value) {
      updateResultsPosition();
    }
  });
  const unlisten = await listen("toggle-settings", () => {
    toggleSettings();
  });

  onUnmounted(() => {
    unlisten();
    window.removeEventListener("resize", updateResultsPosition);
  });
});
</script>

<style>
.search-results {
  position: fixed;
  /* 改为固定定位 */
  width: 320px;
  /* 与搜索框宽度一致 */
  max-height: 300px;
  overflow-y: auto;
  z-index: 9999;
  border-radius: 8px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}
</style>

<style scoped>
.header_title {
  display: flex;
  align-items: center;
  min-width: 150px;
}

.header_title h2 {
  margin: 0;
  color: var(--md-sys-color-on-surface);
  font-weight: 600;
}

.search-container {
  position: relative;
}

/* Google风格搜索结果 */
.google-style {
  background-color: var(--md-sys-color-surface);
  border: 1px solid var(--md-sys-color-outline);
  padding: 8px 0;
}

.google-search-results {
  width: 100%;
}

.search-result-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.search-result-item:hover,
.search-result-item.highlighted {
  background-color: var(--md-sys-color-surface-variant);
}

.search-item-icon {
  margin-right: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.search-item-content {
  flex: 1;
}

.search-item-title {
  font-size: 0.9rem;
  color: var(--md-sys-color-on-surface);
  margin-bottom: 4px;
}

.search-item-title .highlight {
  font-weight: bold;
  color: var(--md-sys-color-primary);
}

.search-item-detail {
  display: flex;
  align-items: center;
  font-size: 0.75rem;
  color: var(--md-sys-color-on-surface-variant);
  gap: 8px;
}

.list-name {
  font-style: italic;
}

.priority-badge {
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 2px;
}

.priority-high {
  background-color: rgba(244, 67, 54, 0.1);
  color: #f44336;
}

.priority-medium {
  background-color: rgba(255, 152, 0, 0.1);
  color: #ff9800;
}

.priority-low {
  background-color: rgba(76, 175, 80, 0.1);
  color: #4caf50;
}

.due-date {
  font-size: 0.7rem;
}
</style>
