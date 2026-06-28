<template>
  <div class="hud">
    <button class="hud-btn" :class="{ active: isPaused }" @click="togglePause" :title="isPaused ? 'Resume' : 'Pause'">
      {{ isPaused ? '▶' : '⏸' }}
    </button>
    <div class="speed-group">
      <button v-for="s in speeds" :key="s" class="hud-btn speed-btn" :class="{ active: currentSpeed === s }"
        @click="$emit('setSpeed', s)">
        {{ s }}×
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  currentSpeed: number
}>()

const emit = defineEmits<{
  (e: 'setSpeed', speed: number): void
  (e: 'pause'): void
  (e: 'resume'): void
}>()

const isPaused = ref(false)
const speeds = [1, 2, 3]

function togglePause() {
  isPaused.value = !isPaused.value
  emit(isPaused.value ? 'pause' : 'resume')
}
</script>

<style scoped>
.hud {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 12px;
  background: rgba(26, 26, 46, 0.9);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
}

.hud-btn {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #e0e0e0;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.15s ease;
}

.hud-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.hud-btn.active {
  background: rgba(100, 181, 246, 0.3);
  border-color: #64B5F6;
  color: #64B5F6;
}

.speed-group {
  display: flex;
  gap: 4px;
}

.speed-btn {
  padding: 6px 10px;
  min-width: 38px;
}
</style>
