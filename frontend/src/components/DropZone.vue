<script setup lang="ts">
defineProps<{
  isActive: boolean;
  fileCount: number;
  supportsDirectoryScan: boolean;
}>();

const emit = defineEmits<{
  (event: "pick-files"): void;
  (event: "scan-directory"): void;
  (event: "clear"): void;
  (event: "drag-state", active: boolean): void;
  (event: "drop-files", files: File[]): void;
}>();

const handleDragEnter = () => {
  emit("drag-state", true);
};

const handleDragLeave = (event: DragEvent) => {
  const nextTarget = event.relatedTarget;
  if (nextTarget instanceof Node && event.currentTarget instanceof Node) {
    if (event.currentTarget.contains(nextTarget)) {
      return;
    }
  }
  emit("drag-state", false);
};

const handleDrop = (event: DragEvent) => {
  emit("drag-state", false);
  emit("drop-files", Array.from(event.dataTransfer?.files ?? []));
};
</script>

<template>
  <section
    class="dropzone glass-medium content-animated-block"
    :class="{ active: isActive }"
    @dragenter.prevent="handleDragEnter"
    @dragleave.prevent="handleDragLeave"
    @dragover.prevent="handleDragEnter"
    @drop.prevent="handleDrop"
  >
    <div>
      <p class="eyebrow">输入源</p>
      <h2>{{ supportsDirectoryScan ? "拖入 EPUB、文件夹或直接从系统选择" : "从系统选择 EPUB" }}</h2>
      <p class="muted">
        {{ supportsDirectoryScan ? "支持单文件、多文件、拖拽文件夹或目录扫描。" : "支持从系统文件选择器添加多个 EPUB。" }}当前队列 {{ fileCount }} 个文件。
      </p>
    </div>
    <div class="dropzone-actions">
      <button class="primary-btn" type="button" @click="emit('pick-files')">
        选择文件
      </button>
      <button v-if="supportsDirectoryScan" class="secondary-btn" type="button" @click="emit('scan-directory')">
        扫描目录
      </button>
      <button class="ghost-btn task-action-btn" type="button" @click="emit('clear')">
        清空队列
      </button>
    </div>
  </section>
</template>
