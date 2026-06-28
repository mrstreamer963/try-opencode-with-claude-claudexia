import type { IncomingEvent, OutgoingEvent, WorldSnapshot, BuildingType } from '../types/events'

export type GameBridgeCallback = (event: OutgoingEvent) => void

export class GameBridge {
  private worker: Worker
  private callbacks: Set<GameBridgeCallback> = new Set()
  private _latestSnapshot: WorldSnapshot | null = null
  private _isReady = false

  constructor() {
    this.worker = new Worker(
      new URL('../worker/gameWorker.ts', import.meta.url),
      { type: 'module' }
    )

    this.worker.onmessage = (e: MessageEvent<OutgoingEvent>) => {
      const event = e.data
      this.handleEvent(event)
    }

    this.worker.onerror = (e) => {
      console.error('[GameBridge] Worker error:', e)
      for (const cb of this.callbacks) {
        cb({ type: 'Error', message: `Worker error: ${e.message}` })
      }
    }
  }

  private handleEvent(event: OutgoingEvent) {
    switch (event.type) {
      case 'Ready':
        this._isReady = true
        break

      case 'WorldSnapshot':
        this._latestSnapshot = event.snapshot
        break
    }

    // Notify all listeners
    for (const cb of this.callbacks) {
      cb(event)
    }
  }

  get isReady() {
    return this._isReady
  }

  get latestSnapshot() {
    return this._latestSnapshot
  }

  /** Subscribe to game events */
  onEvent(callback: GameBridgeCallback): () => void {
    this.callbacks.add(callback)
    return () => this.callbacks.delete(callback)
  }

  /** Place a building */
  placeBuilding(buildingType: BuildingType, x: number, y: number) {
    this.send({ type: 'PlaceBuilding', buildingType, x, y })
  }

  /** Set game speed multiplier */
  setSpeed(speed: number) {
    this.send({ type: 'SetSpeed', speed })
  }

  /** Pause the game */
  pause() {
    this.send({ type: 'Pause' })
  }

  /** Resume the game */
  resume() {
    this.send({ type: 'Resume' })
  }

  private send(event: IncomingEvent) {
    this.worker.postMessage(event)
  }

  /** Clean up the worker */
  destroy() {
    this.worker.terminate()
    this.callbacks.clear()
  }
}
