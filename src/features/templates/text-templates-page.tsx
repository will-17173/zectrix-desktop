import { useState } from "react";

type Device = { deviceId: string; alias: string; board: string };

export type TextTemplateRecord = {
  id: number;
  title: string;
  content: string;
};

type Props = {
  templates: TextTemplateRecord[];
  devices: Device[];
  onCreateTemplate: (input: { title: string; content: string }) => Promise<TextTemplateRecord>;
  onPushTemplate: (templateId: number, deviceId: string) => Promise<void>;
};

export function TextTemplatesPage({ templates: initialTemplates, devices, onCreateTemplate, onPushTemplate }: Props) {
  const [templates, setTemplates] = useState<TextTemplateRecord[]>(initialTemplates);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const created = await onCreateTemplate({ title, content });
    setTemplates((prev) => [...prev, created]);
    setTitle("");
    setContent("");
  }

  async function handlePush(templateId: number) {
    const deviceId = devices[0]?.deviceId;
    if (!deviceId) return;
    await onPushTemplate(templateId, deviceId);
  }

  return (
    <section className="p-4">
      <form onSubmit={handleSubmit} className="space-y-4 max-w-md">
        <div className="space-y-2">
          <label htmlFor="template-title" className="block text-sm font-medium">模板标题</label>
          <input
            id="template-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
          />
        </div>
        <div className="space-y-2">
          <label htmlFor="template-content" className="block text-sm font-medium">模板正文</label>
          <textarea
            id="template-content"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600"
          />
        </div>
        <button
          type="submit"
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          保存模板
        </button>
      </form>

      <ul className="mt-6 space-y-2">
        {templates.map((t) => (
          <li key={t.id} className="flex items-center gap-3 p-2 border border-gray-200 rounded-md dark:border-gray-700">
            <span>{t.title}</span>
            <button
              type="button"
              onClick={() => handlePush(t.id)}
              className="ml-auto px-3 py-1 text-sm bg-gray-200 rounded hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
            >
              推送模板
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}