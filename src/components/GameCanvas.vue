<template>
  <div ref="canvasContainer" class="game-canvas" @wheel="onWheel" @pointerdown="onPointerDown"
    @pointermove="onPointerMove" @pointerup="onPointerUp" @contextmenu.prevent>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, type PropType } from 'vue'
import { Application, Container, Graphics } from 'pixi.js'
import type { WorldSnapshot, ColonistState, BuildingState, BuildingType } from '../types/events'
import { TileType } from '../types/events'

const TILE_SIZE = 16

const props = defineProps({
  snapshot: {
    type: Object as PropType<WorldSnapshot | null>,
    default: null,
  },
  placementMode: {
    type: [String, null] as PropType<BuildingType | null>,
    default: null,
  },
})

const emit = defineEmits<{
  (e: 'tileClick', x: number, y: number): void
  (e: 'colonistClick', colonist: ColonistState): void
  (e: 'emptyClick'): void
}>()

const canvasContainer = ref<HTMLDivElement | null>(null)

let app: Application | null = null
let terrainLayer: Container
let buildingLayer: Container
let entityLayer: Container
let blueprintLayer: Container

// Camera state
let cameraX = 0
let cameraY = 0
let cameraScale = 1.5
let isDragging = false
let dragStartX = 0
let dragStartY = 0
let dragStartCamX = 0
let dragStartCamY = 0

// Track hover position for blueprint
let hoverTileX = -1
let hoverTileY = -1

// Tile colors
const TILE_COLORS: Record<TileType, number> = {
  [TileType.Water]: 0x2196F3,
  [TileType.Sand]: 0xF5D68D,
  [TileType.Grass]: 0x66BB6A,
}

// Building colors
const BUILDING_COLORS: Record<string, number> = {
  Wall: 0x616161,
  Bed: 0x8D6E63,
  BerryBush: 0xE91E63,
}

const COLONIST_COLORS = [0xFFEB3B, 0xFF9800, 0x9C27B0, 0x00BCD4, 0xCDDC39, 0xFF5722, 0x3F51B5, 0x009688]

onMounted(async () => {
  if (!canvasContainer.value) return

  app = new Application()
  await app.init({
    background: 0x1A1A2E,
    resizeTo: canvasContainer.value,
    antialias: false,
    resolution: window.devicePixelRatio || 1,
    autoDensity: true,
  })

  canvasContainer.value.appendChild(app.canvas as HTMLCanvasElement)

  // Create layers
  terrainLayer = new Container()
  buildingLayer = new Container()
  entityLayer = new Container()
  blueprintLayer = new Container()

  const worldContainer = new Container()
  worldContainer.addChild(terrainLayer, buildingLayer, blueprintLayer, entityLayer)
  app.stage.addChild(worldContainer)

  // Center camera
  cameraX = -(25 * TILE_SIZE * cameraScale - (canvasContainer.value?.clientWidth || 800) / 2)
  cameraY = -(25 * TILE_SIZE * cameraScale - (canvasContainer.value?.clientHeight || 600) / 2)
  updateCamera()

  // Start render loop
  app.ticker.add(() => {
    render()
  })
})

onUnmounted(() => {
  if (app) {
    app.destroy(true)
    app = null
  }
})

function updateCamera() {
  if (!app) return
  const world = app.stage.children[0]
  if (world) {
    world.x = cameraX
    world.y = cameraY
    world.scale.set(cameraScale)
  }
}

// ── Camera Controls ──
function onWheel(e: WheelEvent) {
  e.preventDefault()
  const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1
  const oldScale = cameraScale
  cameraScale = Math.max(0.5, Math.min(5, cameraScale * zoomFactor))

  // Zoom toward mouse position
  if (canvasContainer.value) {
    const rect = canvasContainer.value.getBoundingClientRect()
    const mouseX = e.clientX - rect.left
    const mouseY = e.clientY - rect.top

    cameraX = mouseX - (mouseX - cameraX) * (cameraScale / oldScale)
    cameraY = mouseY - (mouseY - cameraY) * (cameraScale / oldScale)
  }

  updateCamera()
}

function onPointerDown(e: PointerEvent) {
  if (e.button === 0 || e.button === 2) {
    isDragging = true
    dragStartX = e.clientX
    dragStartY = e.clientY
    dragStartCamX = cameraX
    dragStartCamY = cameraY
  }
}

function onPointerMove(e: PointerEvent) {
  // Update hover tile
  if (canvasContainer.value) {
    const rect = canvasContainer.value.getBoundingClientRect()
    const worldX = (e.clientX - rect.left - cameraX) / cameraScale
    const worldY = (e.clientY - rect.top - cameraY) / cameraScale
    hoverTileX = Math.floor(worldX / TILE_SIZE)
    hoverTileY = Math.floor(worldY / TILE_SIZE)
  }

  if (isDragging) {
    cameraX = dragStartCamX + (e.clientX - dragStartX)
    cameraY = dragStartCamY + (e.clientY - dragStartY)
    updateCamera()
  }
}

function onPointerUp(e: PointerEvent) {
  const wasDrag = Math.abs(e.clientX - dragStartX) > 3 || Math.abs(e.clientY - dragStartY) > 3
  isDragging = false

  if (!wasDrag && canvasContainer.value) {
    const rect = canvasContainer.value.getBoundingClientRect()
    const worldX = (e.clientX - rect.left - cameraX) / cameraScale
    const worldY = (e.clientY - rect.top - cameraY) / cameraScale
    const tileX = Math.floor(worldX / TILE_SIZE)
    const tileY = Math.floor(worldY / TILE_SIZE)

    if (props.placementMode) {
      emit('tileClick', tileX, tileY)
      return
    }

    // Check colonist hit
    const snap = props.snapshot
    if (snap) {
      const clickedColonist = snap.colonists.find(
        (c) => c.x === tileX && c.y === tileY
      )
      if (clickedColonist) {
        emit('colonistClick', clickedColonist)
      } else {
        emit('emptyClick')
      }
    }
  }
}

let lastSnapshotRef: WorldSnapshot | null = null

function render() {
  const snap = props.snapshot
  if (!snap || snap === lastSnapshotRef) return
  lastSnapshotRef = snap

  renderTerrain(snap)
  renderBuildings(snap.buildings)
  renderEntities(snap.colonists)
  renderBlueprint()
}

function renderTerrain(snap: WorldSnapshot) {
  terrainLayer.removeChildren()
  const g = new Graphics()

  for (let y = 0; y < snap.height; y++) {
    for (let x = 0; x < snap.width; x++) {
      const tile = snap.tiles[y * snap.width + x]
      const color = TILE_COLORS[tile] || 0x333333
      g.rect(x * TILE_SIZE, y * TILE_SIZE, TILE_SIZE, TILE_SIZE)
      g.fill(color)
    }
  }

  // Grid lines (subtle)
  for (let y = 0; y <= snap.height; y++) {
    g.moveTo(0, y * TILE_SIZE)
    g.lineTo(snap.width * TILE_SIZE, y * TILE_SIZE)
    g.stroke({ width: 0.5, color: 0x000000, alpha: 0.15 })
  }
  for (let x = 0; x <= snap.width; x++) {
    g.moveTo(x * TILE_SIZE, 0)
    g.lineTo(x * TILE_SIZE, snap.height * TILE_SIZE)
    g.stroke({ width: 0.5, color: 0x000000, alpha: 0.15 })
  }

  terrainLayer.addChild(g)
}

function renderBuildings(buildings: BuildingState[]) {
  buildingLayer.removeChildren()
  const g = new Graphics()

  for (const b of buildings) {
    const color = BUILDING_COLORS[b.buildingType] || 0x999999
    g.rect(b.x * TILE_SIZE + 1, b.y * TILE_SIZE + 1, TILE_SIZE - 2, TILE_SIZE - 2)
    g.fill(color)
    g.stroke({ width: 1, color: 0x000000, alpha: 0.3 })
  }

  buildingLayer.addChild(g)
}

function renderEntities(colonists: ColonistState[]) {
  entityLayer.removeChildren()
  const g = new Graphics()

  for (let i = 0; i < colonists.length; i++) {
    const c = colonists[i]
    const color = COLONIST_COLORS[i % COLONIST_COLORS.length]
    const cx = c.x * TILE_SIZE + TILE_SIZE / 2
    const cy = c.y * TILE_SIZE + TILE_SIZE / 2
    const radius = TILE_SIZE * 0.35

    // Body circle
    g.circle(cx, cy, radius)
    g.fill(color)
    g.stroke({ width: 1.5, color: 0x000000, alpha: 0.5 })

    // Small indicator for task
    if (c.task === 'Eating') {
      g.circle(cx + radius * 0.6, cy - radius * 0.6, 2)
      g.fill(0xFF4081)
    } else if (c.task === 'Sleeping') {
      // Z indicator
      g.circle(cx + radius * 0.6, cy - radius * 0.6, 2)
      g.fill(0x7C4DFF)
    }
  }

  entityLayer.addChild(g)
}

function renderBlueprint() {
  blueprintLayer.removeChildren()
  if (!props.placementMode || hoverTileX < 0 || hoverTileY < 0) return
  if (!props.snapshot) return
  if (hoverTileX >= props.snapshot.width || hoverTileY >= props.snapshot.height) return

  const g = new Graphics()
  const color = BUILDING_COLORS[props.placementMode] || 0x999999

  g.rect(
    hoverTileX * TILE_SIZE + 1,
    hoverTileY * TILE_SIZE + 1,
    TILE_SIZE - 2,
    TILE_SIZE - 2
  )
  g.fill({ color, alpha: 0.5 })
  g.stroke({ width: 1, color: 0xFFFFFF, alpha: 0.7 })

  blueprintLayer.addChild(g)
}

// Re-render blueprint on hover changes
watch(() => props.placementMode, () => {
  renderBlueprint()
})
</script>

<style scoped>
.game-canvas {
  width: 100%;
  height: 100%;
  overflow: hidden;
  cursor: grab;
}

.game-canvas:active {
  cursor: grabbing;
}
</style>
