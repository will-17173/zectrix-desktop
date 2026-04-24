import type { ImageLoopTask, DeviceRecord } from "@/lib/tauri";
import { ImageLoopTaskCard } from "./image-loop-task-card";

type Props = {
  tasks: ImageLoopTask[];
  devices: DeviceRecord[];
  onStart: (taskId: number) => Promise<void>;
  onStop: (taskId: number) => Promise<void>;
  onEdit: (task: ImageLoopTask) => void;
  onDelete: (taskId: number) => Promise<void>;
};

export function ImageLoopTaskList({
  tasks,
  devices,
  onStart,
  onStop,
  onEdit,
  onDelete,
}: Props) {
  if (tasks.length === 0) {
    return (
      <p className="text-sm text-gray-500 py-4">
        暂无循环相册任务，点击上方"新建任务"创建。
      </p>
    );
  }

  return (
    <ul className="space-y-3">
      {tasks.map((task) => (
        <li key={task.id}>
          <ImageLoopTaskCard
            task={task}
            devices={devices}
            onStart={onStart}
            onStop={onStop}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        </li>
      ))}
    </ul>
  );
}
