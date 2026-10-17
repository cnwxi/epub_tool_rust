<script setup lang="ts">
import { computed } from "vue";

import type { TaskEvent } from "../types";

const props = defineProps<{
  logs: TaskEvent[];
  canOpenLog: boolean;
}>();

const emit = defineEmits<{
  (event: "open-log"): void;
  (event: "clear-log"): void;
}>();

const visibleLogs = computed(() => [...props.logs].reverse());
</script>

<template>
  <article class="panel panel-console glass-medium content-animated-block">
    <div class="panel-head">
      <div>
        <p class="eyebrow">过程</p>
        <h3>处理日志</h3>
      </div>
      <div class="panel-actions">
        <button v-if="canOpenLog" class="ghost-btn task-action-btn" type="button" @click="emit('open-log')">
          打开处理日志
        </button>
        <button class="ghost-btn task-action-btn" type="button" @click="emit('clear-log')">
          清空面板
        </button>
      </div>
    </div>
    <div class="log-list">
      <div v-if="logs.length === 0" class="log-empty">尚未执行任务。</div>
      <div v-for="(log, index) in visibleLogs" :key="`${log.event}-${index}`" class="log-row glass-soft"
        :class="log.level ?? log.status">
        <span class="log-event">{{ log.event }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
    </div>
  </article>
</template>
