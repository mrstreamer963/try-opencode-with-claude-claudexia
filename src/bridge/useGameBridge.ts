import { ref, shallowRef, onUnmounted, type Ref, type ShallowRef } from 'vue'
import { GameBridge } from './GameBridge'
import type { WorldSnapshot, BuildingType } from '../types/events'

const bridge = new GameBridge()

export function useGameBridge(): {
  snapshot: ShallowRef<WorldSnapshot | null>
  isReady: Ref<boolean>
  placeBuilding: (type: BuildingType, x: number, y: number) => void
  setSpeed: (speed: number) => void
  pause: () => void
  resume: () => void
} {
  const snapshot = shallowRef<WorldSnapshot | null>(null)
  const isReady = ref(false)

  const unsubscribe = bridge.onEvent((event) => {
    switch (event.type) {
      case 'Ready':
        isReady.value = true
        break
      case 'WorldSnapshot':
        snapshot.value = event.snapshot
        break
      case 'Error':
        console.error('[Game]', event.message)
        break
    }
  })

  onUnmounted(() => {
    unsubscribe()
  })

  return {
    snapshot,
    isReady,
    placeBuilding: (type, x, y) => bridge.placeBuilding(type, x, y),
    setSpeed: (speed) => bridge.setSpeed(speed),
    pause: () => bridge.pause(),
    resume: () => bridge.resume(),
  }
}

export { bridge }
