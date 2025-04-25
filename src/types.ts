// src/types.ts

// 对应 Rust 中的 EventMetadata
export interface EventMetadata {
  uuid: string;
  timestamp: number;
  tag: number | null;
  list: number | null;
}

// 对应 Rust 中的 EventType
export enum EventType {
  Instant = "Instant",
  Duration = "Duration"
}

// 对应 Rust 中的 DurationTime
export interface DurationTime {
  start: number;
  end: number;
}

// 对应 Rust 中的 TaskTime
export type TaskTime = 
  | { Deadline: number }
  | { Duration: DurationTime };

// 对应 Rust 中的 Event
export interface Event {
  metadata: EventMetadata;
  title: string;
  content: string;
  event_type: EventType;
  task_time: TaskTime;
  finished: boolean;
}

// 对应 Rust 中的 TagColor
export enum TagColor {
  Primary = "Primary",
  Secondary = "Secondary",
  Sucess = "Sucess",
  Info = "Info",
  Warning = "Warning",
  Error = "Error"
}

// 对应 Rust 中的 Tag
export interface Tag {
  name: string;
  color: TagColor;
}

// 对应 Rust 中的 List
export interface List {
  name: string;
}