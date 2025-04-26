//这个文件定义了与 Rust 代码中 TypeScript 数据接口

export interface EventMetadata {
  uuid: string;
  timestamp: number;
  tag: number | null;
  list: number | null;
}

export enum EventType {
  Instant = "Instant",
  Duration = "Duration"
}

export interface DurationTime {
  start: number;
  end: number;
}

export type TaskTime = 
  | { Deadline: number }
  | { Duration: DurationTime };

export interface Event {
  metadata: EventMetadata;
  title: string;
  content: string;
  event_type: EventType;
  task_time: TaskTime;
  finished: boolean;
}

export enum TagColor {
  Primary = "Primary",
  Secondary = "Secondary",
  Sucess = "Sucess",
  Info = "Info",
  Warning = "Warning",
  Error = "Error"
}

export interface Tag {
  name: string;
  color: TagColor;
}

export interface List {
  name: string;
}