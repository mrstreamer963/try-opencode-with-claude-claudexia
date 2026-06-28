import initWasm, { init, tick, push_event, drain_events } from '../wasm/game_core'
import type { IncomingEvent, OutgoingEvent } from '../types/events'

let gameLoopInterval: ReturnType<typeof setInterval> | null = null
let speedMultiplier = 1
let isPaused = false
const TICK_INTERVAL = 50 // 20 ticks/sec base rate
const BASE_DT = TICK_INTERVAL / 1000 // 0.05 seconds

async function start() {
  try {
    // Initialize WASM module
    await initWasm()

    // Initialize game engine and get initial snapshot
    const initialEventsJson = init()
    const initialEvents: OutgoingEvent[] = JSON.parse(initialEventsJson)

    // Post Ready event
    self.postMessage({ type: 'Ready' } satisfies OutgoingEvent)

    // Post initial events (WorldSnapshot)
    for (const event of initialEvents) {
      self.postMessage(event)
    }

    // Start game loop
    startGameLoop()
  } catch (error) {
    self.postMessage({
      type: 'Error',
      message: `WASM initialization failed: ${error}`,
    } satisfies OutgoingEvent)
  }
}

function startGameLoop() {
  if (gameLoopInterval !== null) return

  gameLoopInterval = setInterval(() => {
    const dt = BASE_DT * speedMultiplier
    tick(dt)

    const eventsJson = drain_events()
    const events: OutgoingEvent[] = JSON.parse(eventsJson)

    for (const event of events) {
      self.postMessage(event)
    }
  }, TICK_INTERVAL)
}

function stopGameLoop() {
  if (gameLoopInterval !== null) {
    clearInterval(gameLoopInterval)
    gameLoopInterval = null
  }
}

// Handle incoming messages from main thread
self.onmessage = (e: MessageEvent<IncomingEvent>) => {
  const event = e.data

  switch (event.type) {
    case 'Pause':
      isPaused = true
      stopGameLoop()
      break

    case 'Resume':
      isPaused = false
      startGameLoop()
      break

    case 'SetSpeed':
      speedMultiplier = event.speed
      break

    case 'PlaceBuilding':
      // Forward to WASM
      push_event(JSON.stringify(event))
      break
  }
}

// Start the worker
start()
