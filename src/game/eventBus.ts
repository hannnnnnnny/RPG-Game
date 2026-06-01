type Handler<T = unknown> = (payload: T) => void;

class GameEventBus {
  private handlers = new Map<string, Set<Handler>>();

  on<T>(event: string, handler: Handler<T>) {
    const handlers = this.handlers.get(event) ?? new Set();
    handlers.add(handler as Handler);
    this.handlers.set(event, handlers);
    return () => this.off(event, handler);
  }

  off<T>(event: string, handler: Handler<T>) {
    this.handlers.get(event)?.delete(handler as Handler);
  }

  emit<T>(event: string, payload: T) {
    this.handlers.get(event)?.forEach((handler) => handler(payload));
  }
}

export const gameEvents = new GameEventBus();

