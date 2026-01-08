import { writable } from "svelte/store";

export interface Tab {
  id: string;
  title: string;
  type: "query" | "diagram"; // table_data можно добавить позже
  connectionId: number;
  content?: string; // Для SQL запроса или schema_name для новой диаграммы
  diagramId?: number; // ID сохраненной диаграммы (если есть)
  diagramData?: string; // JSON сохраненной диаграммы (для инициализации)
  isDirty?: boolean;
}

let tabs = $state<Tab[]>([]);
let activeTabId = $state<string | null>(null);

export const tabsState = {
  get tabs() {
    return tabs;
  },
  get activeTabId() {
    return activeTabId;
  },

  addTab: (tab: Omit<Tab, "id">) => {
    const id = crypto.randomUUID();
    const newTab = { ...tab, id };
    tabs = [...tabs, newTab];
    activeTabId = id;
  },

  closeTab: (id: string) => {
    tabs = tabs.filter((t) => t.id !== id);
    if (activeTabId === id) {
      activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : null;
    }
  },

  setActive: (id: string) => {
    activeTabId = id;
  },

  updateTabContent: (id: string, content: string) => {
    const tab = tabs.find((t) => t.id === id);
    if (tab) {
      tab.content = content;
      tab.isDirty = true;
    }
  },

  markAsSaved: (id: string, newTitle?: string, newId?: number) => {
    const tab = tabs.find((t) => t.id === id);
    if (tab) {
      tab.isDirty = false;
      if (newTitle) tab.title = newTitle;
      if (newId) tab.diagramId = newId;
    }
  },
};
