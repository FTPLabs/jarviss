import { writable, get } from "svelte/store"
  import { invoke } from "@tauri-apps/api/core"
  import { getCurrentWindow } from "@tauri-apps/api/window"

  // ### IPC STORES ###

  export type JarvisState = "disconnected" | "idle" | "listening" | "processing"

  export const jarvisState = writable<JarvisState>("disconnected")
  export const ipcConnected = writable(false)
  export const lastRecognizedText = writable("")
  export const lastExecutedCommand = writable("")
  export const lastError = writable("")

  // ### CONNECTION ###

  const IPC_PORT_DEFAULT = 9712
  const RECONNECT_DELAY = 5000

  let ws: WebSocket | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let manualDisconnect = false
  let enabled = false

  export function enableIpc() {
      enabled = true
      manualDisconnect = false
      connectIpc()
  }

  export function disableIpc() {
      enabled = false
      disconnectIpc()
  }

  export function connectIpc(port: number = IPC_PORT_DEFAULT) {
      if (ws?.readyState === WebSocket.OPEN) return

      // Respect JARVIS_IPC_PORT env var when running in dev via Tauri
      const effectivePort = (typeof window !== "undefined" && (window as any).__JARVIS_IPC_PORT)
          ? parseInt((window as any).__JARVIS_IPC_PORT, 10)
          : port

      ws = new WebSocket(`ws://127.0.0.1:${effectivePort}`)

      ws.onopen = () => {
          ipcConnected.set(true)
          jarvisState.set("idle")
          console.log("[IPC] connected on port", effectivePort)
      }

      ws.onclose = () => {
          ipcConnected.set(false)
          jarvisState.set("disconnected")
          console.log("[IPC] disconnected")
          if (enabled && !manualDisconnect) {
              scheduleReconnect()
          }
      }

      ws.onerror = (err) => {
          console.error("[IPC] error:", err)
      }

      ws.onmessage = (event) => {
          try {
              const msg = JSON.parse(event.data)
              handleEvent(msg)
          } catch (e) {
              console.error("[IPC] failed to parse message:", e)
          }
      }
  }

  function scheduleReconnect() {
      if (reconnectTimer || manualDisconnect || !enabled) return

      console.log(`[IPC] Will retry in ${RECONNECT_DELAY / 1000}s...`)
      reconnectTimer = setTimeout(() => {
          reconnectTimer = null
          connectIpc()
      }, RECONNECT_DELAY)
  }

  export function disconnectIpc() {
      manualDisconnect = true
      if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null }
      if (ws) { ws.close(); ws = null }
      ipcConnected.set(false)
      jarvisState.set("disconnected")
  }

  // ### EVENT HANDLING ###

  function handleEvent(data: any) {
      console.log("[IPC] Event", data.event, data)

      switch (data.event) {
          case "wake_word_detected":
          case "listening":
              jarvisState.set("listening")
              break

          case "speech_recognized":
              lastRecognizedText.set(data.text || "")
              jarvisState.set("processing")
              break

          case "command_executed":
              lastExecutedCommand.set(data.id || "")
              break

          case "idle":
              jarvisState.set("idle")
              break

          case "error":
              lastError.set(data.message || "Unknown error")
              break

          case "started":
              jarvisState.set("idle")
              break

          case "stopping":
              jarvisState.set("disconnected")
              break

          case "pong":
              // connection heartbeat confirmed
              break

          case "reveal_window":
              revealWindow()
              break
      }
  }

  // ### ACTIONS ###

  export function sendAction(action: string, payload: Record<string, any> = {}) {
      if (ws?.readyState !== WebSocket.OPEN) {
          return false
      }
      ws.send(JSON.stringify({ action, ...payload }))
      return true
  }

  export function stopJarvisApp() {
      return sendAction("stop")
  }

  export function reloadCommands() {
      return sendAction("reload_commands")
  }

  export function sendIpcMessage(message: object): Promise<void> {
      return new Promise((resolve, reject) => {
          if (!ws || ws.readyState !== WebSocket.OPEN) {
              reject(new Error("IPC not connected"))
              return
          }
          try {
              ws.send(JSON.stringify(message))
              resolve()
          } catch (err) {
              reject(err)
          }
      })
  }

  export function sendTextCommand(text: string): boolean {
      return sendAction("text_command", { text })
  }

  async function revealWindow() {
      try {
          const window = getCurrentWindow()
          await window.show()
          await window.unminimize()
          await window.setFocus()
      } catch (e) {
          console.error("[IPC] Failed to reveal window:", e)
      }
  }

  export function pingJarvis() {
      return sendAction("ping")
  }
  