<script setup lang="ts">
import { computed } from "vue";

import type { FileIssue, TaskResult } from "../types";

const props = defineProps<{
  result: TaskResult | null;
  isMobile: boolean;
}>();

const emit = defineEmits<{
  (event: "open-output", path: string): void;
}>();

const summary = computed(() => props.result?.summary ?? { total: 0, success: 0, failed: 0, skipped: 0 });
const outputs = computed(() => props.result?.outputs ?? []);
const errors = computed(() => props.result?.errors ?? []);
const skipped = computed(() => props.result?.skipped ?? []);

const displayName = (path: string): string => path.split(/[\\/]/).pop() ?? path;

interface DetailGroup {
  key: "error" | "skip";
  label: string;
  items: FileIssue[];
}

const detailGroups = computed<DetailGroup[]>(() => {
  const groups: DetailGroup[] = [
    { key: "error", label: "失败", items: errors.value },
    { key: "skip", label: "跳过", items: skipped.value },
  ];
  return groups.filter((group) => group.items.length > 0);
});
</script>

<template>
  <article class="panel panel-result glass-medium content-animated-block">
    <div class="panel-head">
      <div>
        <p class="eyebrow">结果</p>
        <h3>最近一次执行摘要</h3>
      </div>
    </div>

    <div v-if="!result" class="result-empty">还没有可展示的执行结果。</div>
    <div v-else class="result-block">
      <div class="result-metrics">
        <div class="result-metric-card total">
          <strong class="content-animated-value">{{ summary.total }}</strong>
          <span>总文件</span>
        </div>
        <div class="result-metric-card success">
          <strong class="content-animated-value">{{ summary.success }}</strong>
          <span>成功</span>
        </div>
        <div class="result-metric-card error">
          <strong class="content-animated-value">{{ summary.failed }}</strong>
          <span>失败</span>
        </div>
        <div class="result-metric-card skip">
          <strong class="content-animated-value">{{ summary.skipped }}</strong>
          <span>跳过</span>
        </div>
      </div>

      <div v-if="outputs.length > 0" class="result-detail-block">
        <div class="result-detail-head">
          <strong>成功</strong>
          <span class="content-animated-value">{{ outputs.length }} 项</span>
        </div>
        <div class="result-output-list">
          <button v-for="output in outputs" :key="output" class="result-output-row success glass-soft" type="button"
            @click="emit('open-output', output)">
            <div class="result-row-head">
              <strong>{{ displayName(output) }}</strong>
              <span class="result-status-tag success">{{ isMobile ? "导出" : "成功" }}</span>
            </div>
            <span>{{ output }}</span>
          </button>
        </div>
      </div>

      <div v-for="group in detailGroups" :key="group.key" class="result-detail-block">
        <div class="result-detail-head">
          <strong>{{ group.label }}</strong>
          <span class="content-animated-value">{{ group.items.length }} 项</span>
        </div>
        <div class="result-detail-list">
          <div v-for="item in group.items" :key="`${item.inputFile}-${item.message}`"
            class="result-detail-row glass-soft" :class="group.key">
            <div class="result-row-head">
              <strong>{{ displayName(item.inputFile) }}</strong>
              <span class="result-status-tag" :class="group.key">{{ group.label }}</span>
            </div>
            <p>{{ item.message }}</p>
            <span>{{ item.inputFile }}</span>
          </div>
        </div>
      </div>
    </div>
  </article>
</template>
