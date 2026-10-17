<script setup lang="ts">
const props = defineProps<{
  label: string;
  modelValue: number;
  displayValue: string;
  fillPercent: number;
  min?: number;
  max?: number;
  step?: number;
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: number): void;
  (event: "change"): void;
}>();

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", Number(target.value));
};
</script>

<template>
  <label class="font-setting-field font-slider-field">
    <span class="font-slider-head">
      <span>{{ label }}</span>
      <strong>{{ displayValue }}</strong>
    </span>
    <span class="font-slider-control" :style="{ '--ocr-confidence-percent': `${fillPercent}%` }">
      <span class="font-slider-track" aria-hidden="true">
        <span class="font-slider-fill"></span>
      </span>
      <input class="font-confidence-slider" :aria-label="label" :aria-valuetext="displayValue"
        :value="modelValue" :min="min ?? 1" :max="max ?? 100" :step="step ?? 1" type="range"
        @input="handleInput" @change="emit('change')" />
    </span>
  </label>
</template>
