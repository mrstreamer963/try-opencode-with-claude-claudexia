<template>
  <div class="colonist-panel" v-if="colonist">
    <div class="panel-header">
      <span class="colonist-name">{{ colonist.name }}</span>
      <button class="close-btn" @click="$emit('close')">✕</button>
    </div>
    <div class="panel-body">
      <div class="stat-row">
        <span class="stat-label">Task</span>
        <span class="stat-value task-badge" :class="taskClass">{{ taskLabel }}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Position</span>
        <span class="stat-value">({{ colonist.x }}, {{ colonist.y }})</span>
      </div>
      <div class="need-bar">
        <span class="need-label">Food</span>
        <div class="bar-track">
          <div class="bar-fill food" :style="{ width: `${colonist.food * 100}%` }"></div>
        </div>
        <span class="need-pct">{{ Math.round(colonist.food * 100) }}%</span>
      </div>
      <div class="need-bar">
        <span class="need-label">Sleep</span>
        <div class="bar-track">
          <div class="bar-fill sleep" :style="{ width: `${colonist.sleep * 100}%` }"></div>
        </div>
        <span class="need-pct">{{ Math.round(colonist.sleep * 100) }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type PropType } from 'vue'
import type { ColonistState } from '../types/events'
import { ColonistTask } from '../types/events'

const props = defineProps({
  colonist: {
    type: Object as PropType<ColonistState | null>,
    default: null,
  },
})

defineEmits<{
  (e: 'close'): void
}>()

const taskLabel = computed(() => {
  if (!props.colonist) return ''
  const labels: Record<ColonistTask, string> = {
    [ColonistTask.Idle]: 'Idle',
    [ColonistTask.MovingToFood]: 'Going to eat',
    [ColonistTask.Eating]: 'Eating',
    [ColonistTask.MovingToBed]: 'Going to bed',
    [ColonistTask.Sleeping]: 'Sleeping',
    [ColonistTask.Wandering]: 'Wandering',
  }
  return labels[props.colonist.task] || props.colonist.task
})

const taskClass = computed(() => {
  if (!props.colonist) return ''
  return `task-${props.colonist.task.toLowerCase()}`
})
</script>

<style scoped>
.colonist-panel {
  width: 240px;
  background: rgba(26, 26, 46, 0.95);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(12px);
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.colonist-name {
  font-size: 16px;
  font-weight: 700;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #999;
  font-size: 14px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.panel-body {
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  color: #9e9e9e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  color: #e0e0e0;
  font-size: 13px;
}

.task-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
}

.task-idle { background: rgba(158, 158, 158, 0.3); }
.task-movingtofood, .task-eating { background: rgba(233, 30, 99, 0.3); color: #FF80AB; }
.task-movingtobed, .task-sleeping { background: rgba(124, 77, 255, 0.3); color: #B388FF; }
.task-wandering { background: rgba(255, 235, 59, 0.3); color: #FFF176; }

.need-bar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.need-label {
  color: #9e9e9e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  width: 40px;
}

.bar-track {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.2s ease;
}

.bar-fill.food {
  background: linear-gradient(90deg, #E91E63, #FF5722);
}

.bar-fill.sleep {
  background: linear-gradient(90deg, #7C4DFF, #448AFF);
}

.need-pct {
  color: #bbb;
  font-size: 11px;
  width: 32px;
  text-align: right;
}
</style>
