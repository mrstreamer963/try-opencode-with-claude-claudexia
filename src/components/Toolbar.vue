<template>
  <div class="toolbar">
    <div class="toolbar-label">Build</div>
    <button v-for="b in buildings" :key="b.type" class="toolbar-btn" :class="{ active: selected === b.type }"
      @click="toggle(b.type)" :title="b.label">
      <span class="btn-icon" :style="{ background: b.color }"></span>
      <span class="btn-text">{{ b.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { BuildingType } from '../types/events'

const props = defineProps<{
  selected: BuildingType | null
}>()

const emit = defineEmits<{
  (e: 'select', type: BuildingType | null): void
}>()

const buildings = [
  { type: BuildingType.Wall, label: 'Wall', color: '#616161' },
  { type: BuildingType.Bed, label: 'Bed', color: '#8D6E63' },
  { type: BuildingType.BerryBush, label: 'Berry', color: '#E91E63' },
]

function toggle(type: BuildingType) {
  emit('select', props.selected === type ? null : type)
}
</script>

<style scoped>
.toolbar {
  display: flex;
  gap: 6px;
  align-items: center;
  padding: 8px 12px;
  background: rgba(26, 26, 46, 0.9);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
}

.toolbar-label {
  color: #9e9e9e;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-right: 4px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #e0e0e0;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s ease;
}

.toolbar-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.toolbar-btn.active {
  background: rgba(100, 181, 246, 0.3);
  border-color: #64B5F6;
  color: #64B5F6;
}

.btn-icon {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  flex-shrink: 0;
}

.btn-text {
  font-weight: 500;
}
</style>
