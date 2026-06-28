<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import GameCanvas from './components/GameCanvas.vue'
import HUD from './components/HUD.vue'
import Toolbar from './components/Toolbar.vue'
import ColonistInfoPanel from './components/ColonistInfoPanel.vue'
import { useGameBridge } from './bridge/useGameBridge'
import type { ColonistState, BuildingType } from './types/events'

const { snapshot, isReady, placeBuilding, setSpeed, pause, resume } = useGameBridge()

const currentSpeed = ref(1)
const placementMode = ref<BuildingType | null>(null)
const selectedColonist = ref<ColonistState | null>(null)

function handleSetSpeed(speed: number) {
  currentSpeed.value = speed
  setSpeed(speed)
}

function handleTileClick(x: number, y: number) {
  if (placementMode.value) {
    placeBuilding(placementMode.value, x, y)
  }
}

function handleColonistClick(colonist: ColonistState) {
  selectedColonist.value = colonist
}

function handleEmptyClick() {
  selectedColonist.value = null
}

function handleSelectBuilding(type: BuildingType | null) {
  placementMode.value = type
}

// Keep selectedColonist synced with latest snapshot
function updateSelectedColonist() {
  if (selectedColonist.value && snapshot.value) {
    const updated = snapshot.value.colonists.find(
      c => c.id === selectedColonist.value!.id
    )
    if (updated) {
      selectedColonist.value = updated
    }
  }
}

// Escape key to cancel placement
function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    placementMode.value = null
    selectedColonist.value = null
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})

// Watch for snapshot changes to keep panel in sync
import { watch } from 'vue'
watch(snapshot, () => {
  updateSelectedColonist()
})
</script>

<template>
  <div class="app">
    <div v-if="!isReady" class="loading">
      <div class="loading-text">Initializing colony...</div>
    </div>

    <GameCanvas
      v-if="isReady"
      :snapshot="snapshot"
      :placementMode="placementMode"
      @tileClick="handleTileClick"
      @colonistClick="handleColonistClick"
      @emptyClick="handleEmptyClick"
    />

    <div class="ui-overlay top-left" v-if="isReady">
      <HUD
        :currentSpeed="currentSpeed"
        @setSpeed="handleSetSpeed"
        @pause="pause"
        @resume="resume"
      />
    </div>

    <div class="ui-overlay bottom-center" v-if="isReady">
      <Toolbar
        :selected="placementMode"
        @select="handleSelectBuilding"
      />
    </div>

    <div class="ui-overlay top-right" v-if="selectedColonist">
      <ColonistInfoPanel
        :colonist="selectedColonist"
        @close="selectedColonist = null"
      />
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.app {
  width: 100%;
  height: 100%;
  position: relative;
  background: #1A1A2E;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.loading-text {
  color: #64B5F6;
  font-size: 18px;
  font-weight: 600;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

.ui-overlay {
  position: absolute;
  z-index: 10;
  pointer-events: auto;
}

.top-left {
  top: 16px;
  left: 16px;
}

.top-right {
  top: 16px;
  right: 16px;
}

.bottom-center {
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
}
</style>
