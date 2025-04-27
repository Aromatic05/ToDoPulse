# ToDoLists 项目文档

## 1. 项目概述

ToDoLists - 守护时光，传递温度的数字伙伴

ToDoLists 是一个基于 Tauri 框架的跨平台桌面/移动端应用，不仅仅是一款普通的待办事项管理工具，更是一位贴心的数字伙伴。项目着眼于现代人忙碌生活中容易被忽视的细节，用技术守护每一份珍贵的生活瞬间与人际温情。

在快节奏的社会中，我们常常被繁杂事务淹没，忽略了对家人的陪伴、对朋友的关心以及对自我的照顾。ToDoLists 不只提醒您完成工作任务，更注重提醒您生活中的温暖时刻：家人生日、朋友纪念日、亲情互动时间等，帮助用户在效率与情感之间找到平衡。

通过智能 AI 助手，应用不仅提供任务管理建议，还会根据用户的情绪状态和生活习惯，提供温馨提示和情感支持。结合知识库功能，用户可以记录和分享生活经验与感悟，形成一个充满温度的数字社区。

项目采用现代化的前端技术和 Rust 后端，提供高效、安全的用户体验，让科技与人文关怀完美融合，用代码守护每一份善意，让生活充满温度。

## 2. 技术栈

前端

- Vue 3 框架
- Vite 构建工具
- 组件库：Material Design Icons
- 日历组件：FullCalendar

后端

- Tauri 2.x (Rust)
- Rust 依赖：serde, serde_json, async-std 等

## 3.项目结构

``` plaintext
.
├── src/                  # 前端源码
│   ├── components/       # Vue组件
│   │   ├── Cards/        # 卡片相关组件
│   │   ├── Lists/        # 列表相关组件
│   │   ├── Modals/       # 模态框组件
│   │   └── Views/        # 视图组件
│   ├── services/         # 前端服务
│   │   ├── CardDataService.js    # 卡片数据服务
│   │   ├── ListDataService.js    # 列表数据服务
│   │   └── KnowledgeBaseService.js # 知识库服务
│   └── styles/           # 样式文件
├── src-tauri/            # Tauri/Rust后端
│   ├── src/              # Rust源代码
│   │   ├── lib.rs        # 主库文件
│   │   ├── dirs.rs       # 目录相关功能
│   │   ├── ipc.rs        # 前后端通信
│   │   ├── storage.rs    # 数据存储
│   │   ├── aigc.rs       # AI生成内容相关
│   │   └── config.rs     # 配置管理
│   └── Cargo.toml        # Rust依赖配置
└── package.json          # 前端依赖配置
```

## 4. 功能模块

- **待办事项管理**：用户可以创建、编辑和删除待办事项，支持分类和优先级设置。
- **日程安排**：用户可以创建和管理日程，支持日历视图和列表视图切换。
- **知识库管理**：用户可以创建和管理知识库条目，支持搜索和分类。
- **AI助手**：集成了AI助手功能，用户可以通过输入关键词获取相关信息和建议。
- **数据存储**：使用SQLite数据库存储用户数据，支持数据备份和恢复。
- **跨平台支持**：支持Windows、macOS和Linux等多个操作系统。
- **主题切换**：用户可以选择不同的主题风格，提升用户体验。


## 5. 配置文件

配置采用toml文件。以下是一个配置文件示例。
```toml
[Theme]
color = 'blue'
[Info]
switch = true
time = [
  "12:00",
  "13:00",
]
[Model]
switch = false
name = "deepseek-v3"
tokens = " "
prompt = "你是一个优秀的助手"
```

## 6.开发与构建指南

开发环境设置
1. 安装依赖

``` bash
npm install
```

2. 启动开发服务器

``` bash
npm run dev
```

3. 启动 Tauri 后端

``` bash
npm run tauri dev
```

4. 打包应用

``` bash
npm run tauri build
```

