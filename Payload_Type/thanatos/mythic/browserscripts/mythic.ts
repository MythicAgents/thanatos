export type TaskStatus = "success" | "error";

export type Task = {
  task_id: string;
  completed: boolean;
  status: TaskStatus;
}

export interface PlaintextResponse {
  plaintext: string;
}

export interface TableResponse<T extends TableRowEntry> {
  table: Array<TableEntry<T>>;
}

export interface TableEntry<T extends TableRowEntry> {
  title: string;
  headers: Array<TableHeader>;
  rows: Array<T>;
}

export type TableEntryType = "string" | "size" | "button";

export interface TableHeader {
  plaintext: string;
  type: TableEntryType;
  width?: number;
  fillWidth?: boolean;
  disableSort?: boolean;
  cellStyle?: object;
}

export interface TableRowEntry {
  rowStyle?: object;
}

export type TableValue = {
  button?: ButtonTableValue;
  plaintext?: PlaintextTableValue;
}

export interface ButtonTableValue extends BaseTableValue {
}

export interface PlaintextTableValue extends BaseTableValue {
  plaintext: string;
}

interface BaseTableValue {
  cellStyle?: object,
  copyIcon?: boolean,
}
