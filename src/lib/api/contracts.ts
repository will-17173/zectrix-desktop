export type ApiResponse<T> = {
  code: number;
  data: T;
  msg?: string;
};

export type ApiDevice = {
  deviceId: string;
  alias: string;
  board: string;
};
