export interface FrameScheduler { request(callback: FrameRequestCallback): number; cancel(id: number): void }

const browserScheduler: FrameScheduler = {
  request: (callback) => requestAnimationFrame(callback),
  cancel: (id) => cancelAnimationFrame(id),
}

/** Batches every stream arriving during one browser frame into one render. */
export function coalesce<T>(deliver: (items: readonly T[]) => void, scheduler = browserScheduler) {
  let frame: number | undefined
  let queued: T[] = []
  return {
    push(item: T) {
      queued.push(item)
      if (frame !== undefined) return
      frame = scheduler.request(() => {
        const items = queued
        queued = []
        frame = undefined
        deliver(items)
      })
    },
    stop() {
      if (frame !== undefined) scheduler.cancel(frame)
      frame = undefined
      queued = []
    },
  }
}
