// Connects the interface to the Rust side. Every call here maps to a command or event in
// src-tauri; there is no other way for the page to reach the system.
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

type Unsubscribe = () => void

function subscribe<T>(event: string, callback: (payload: T) => void): Unsubscribe {
  let active = true
  let stop: Unsubscribe | null = null
  void listen<T>(event, (message) => {
    if (active) callback(message.payload)
  }).then((unlisten) => {
    if (active) stop = unlisten
    else unlisten()
  })
  return () => {
    active = false
    stop?.()
  }
}

export function installTauriBridge(): void {
  window.murmur = {
    onPillState: (cb) => subscribe('pill-state', cb),
    onConfirm: (cb) => subscribe('pill-confirm', cb),
    confirm: (approved) => {
      void invoke('confirm_action', { approved })
    },
    getSettings: () => invoke('get_settings'),
    saveSettings: (settings) => invoke('save_settings', { settings }),
    getModelStatus: () => invoke('model_status'),
    downloadModel: () => invoke('download_model'),
    onModelProgress: (cb) => subscribe('model-progress', cb),
    getCustomApps: () => invoke('get_custom_apps'),
    saveCustomApps: (apps) => invoke('save_custom_apps', { apps }),
    launchCustomApp: (target) => invoke('launch_custom_app', { target }),
    getVoiceMemory: () => invoke('get_voice_memory'),
    learnVoiceCorrection: (original, corrected, context) =>
      invoke('learn_voice_correction', { original, corrected, context }),
    addVoiceMemoryEntry: ({ canonical, aliases, kind, id }) =>
      invoke('add_voice_memory_entry', { canonical, aliases, kind, id }),
    deleteVoiceMemoryEntry: (id) => invoke('delete_voice_memory_entry', { id }),
    deleteVoiceCorrection: (id) => invoke('delete_voice_correction', { id }),
    clearVoiceMemory: () => invoke('clear_voice_memory'),
    onVoiceTraining: (cb) => subscribe('voice-training', cb),
    onLearningUpdate: (cb) => subscribe('learning-update', cb),
    startVoiceTraining: () => invoke('start_voice_training'),
    stopVoiceTraining: () => invoke('stop_voice_training'),
  }
}
