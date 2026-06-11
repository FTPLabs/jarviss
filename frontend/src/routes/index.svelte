<script lang="ts">
      import { onMount, onDestroy } from "svelte"
      import { invoke } from "@tauri-apps/api/core"

      import SearchBar from "@/components/elements/SearchBar.svelte"
      import ArcReactor from "@/components/elements/ArcReactor.svelte"
      import HDivider from "@/components/elements/HDivider.svelte"
      import Stats from "@/components/elements/Stats.svelte"
      import Footer from "@/components/Footer.svelte"
      
      import {
          isJarvisRunning,
          updateJarvisStats,
          enableIpc,
          disableIpc,
          translate,
          translations
      } from "@/stores"

      $: t = (key: string, fallback?: string) => translate($translations, key, fallback)

      let processRunning = false
      let launching = false
      let launchError = ""
      let wasRunning = false

      let pollTimer: ReturnType<typeof setInterval> | null = null

      isJarvisRunning.subscribe((value) => {
          processRunning = value
          if (value) {
              enableIpc()
              wasRunning = true
              if (pollTimer) {
                  clearInterval(pollTimer)
                  pollTimer = null
              }
              launching = false
              launchError = ""
          } else if (wasRunning) {
              disableIpc()
              wasRunning = false
          }
      })

      onMount(() => {
          updateJarvisStats()
      })

      onDestroy(() => {
          disableIpc()
          if (pollTimer) clearInterval(pollTimer)
      })

      async function runAssistant() {
          launching = true
          launchError = ""

          try {
              await invoke("run_jarvis_app")
          } catch (err) {
              const msg = String(err)
              console.error("Failed to run jarvis-app:", msg)
              launchError = msg.includes("not found")
                  ? t('btn-start-not-found', 'Файл jarvis-app.exe не найден. Переустановите приложение.')
                  : t('btn-start-failed', 'Не удалось запустить. Проверьте логи приложения.')
              launching = false
              return
          }

          // Poll stats until process appears (up to 15 sec, check every 1.5 sec)
          let attempts = 0
          const maxAttempts = 10
          pollTimer = setInterval(async () => {
              attempts++
              await updateJarvisStats()
              if (processRunning || attempts >= maxAttempts) {
                  clearInterval(pollTimer!)
                  pollTimer = null
                  launching = false
                  if (!processRunning) {
                      launchError = t('btn-start-timeout', 'Ассистент запущен, но не отвечает. Проверьте логи.')
                  }
              }
          }, 1500)
      }
  </script>

  <div class="app-container assist-page">

      <div class="search search-section">
          <HDivider />
          <SearchBar />
      </div>

      <div class="reactor-section">
          <div class="reactor-wrapper" class:dimmed={!processRunning}>
              <ArcReactor />
          </div>
          
          {#if !processRunning}
              <div class="offline-badge">
                  <span class="offline-icon">⚠</span>
                  <span class="offline-text">{t('assistant-not-running')}</span>
                  <small>{t('assistant-offline-hint')}</small>
              </div>
              <button 
                  class="start-button" 
                  on:click={runAssistant}
                  disabled={launching}
              >
                  {launching ? t('btn-starting') : t('btn-start')}
              </button>
              {#if launchError}
                  <div class="launch-error">
                      <span class="error-icon">✗</span>
                      {launchError}
                  </div>
              {/if}
          {/if}
      </div>

      <HDivider noMargin />
      <Stats />
      <Footer />
  </div>

  <style>
  .launch-error {
      margin-top: 0.75rem;
      padding: 0.6rem 1rem;
      background: rgba(220, 38, 38, 0.12);
      border: 1px solid rgba(220, 38, 38, 0.4);
      border-radius: 8px;
      color: #ef4444;
      font-size: 0.8rem;
      text-align: center;
      max-width: 320px;
  }
  .error-icon {
      margin-right: 0.4rem;
      font-weight: bold;
  }
  </style>
  