import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api'
import { open as openDialog } from '@tauri-apps/api/dialog'
import { appWindow } from '@tauri-apps/api/window'
import {
  Clipboard,
  Search,
  Pin,
  Settings,
  Menu,
  FileText,
  Image,
  Terminal,
  Copy,
  Star,
  Trash2,
  Check,
  Minimize,
  Maximize2,
  X,
  Plug,
  RefreshCw,
  Wifi,
  WifiOff,
  Upload,
  Download,
  Package
} from 'lucide-react'
import './App.css'

interface ClipboardItem {
  id: number
  content_type: string
  content: string
  hash: string
  created_at: string
  accessed_at: string | null
  pinned: boolean
  favorite: boolean
}

interface Tag {
  id: number
  name: string
  color: string | null
}

interface PluginInfo {
  name: string
  path: string
  enabled: boolean
}

interface SyncConfig {
  server_url: string
  api_token: string | null
  enabled: boolean
  last_sync_at: string | null
}

function App() {
  const [items, setItems] = useState<ClipboardItem[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedItem, setSelectedItem] = useState<ClipboardItem | null>(null)
  const [selectedIndex, setSelectedIndex] = useState<number>(-1)
  const [showSettings, setShowSettings] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const itemsRef = useRef(items)
  const searchTimeoutRef = useRef<number | null>(null)
  const searchInputRef = useRef<HTMLInputElement>(null)

  // Settings state
  const [settings, setSettings] = useState({
    encryptionEnabled: false,
    autoLockMinutes: 5,
    maxItems: 10000,
    retentionDays: 90,
    refreshInterval: 2,
    showNotifications: true
  })
  const [launchAtLogin, setLaunchAtLogin] = useState(false)

  // Password management state
  const [vaultLocked, setVaultLocked] = useState(false)
  const [masterPassword, setMasterPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [unlockPassword, setUnlockPassword] = useState('')
  const [hasPassword, setHasPassword] = useState(false)
  const [unlockError, setUnlockError] = useState<string | null>(null)
  const [passwordError, setPasswordError] = useState<string | null>(null)
  const lastActivityRef = useRef(Date.now())
  const vaultLockedRef = useRef(false)

  // Tag state
  const [allTags, setAllTags] = useState<Tag[]>([])
  const [itemTags, setItemTags] = useState<Record<number, Tag[]>>({})
  const [activeTagFilter, setActiveTagFilter] = useState<number | null>(null)
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; item: ClipboardItem } | null>(null)
  const [tagInput, setTagInput] = useState('')
  const [showTagInput, setShowTagInput] = useState(false)

  // Plugin state
  const [showPlugins, setShowPlugins] = useState(false)
  const [plugins, setPlugins] = useState<PluginInfo[]>([])
  const [pluginLoading, setPluginLoading] = useState(false)
  const [pluginError, setPluginError] = useState<string | null>(null)

  // Sync state
  const [showSync, setShowSync] = useState(false)
  const [syncConfig, setSyncConfig] = useState<SyncConfig>({
    server_url: '',
    api_token: null,
    enabled: false,
    last_sync_at: null,
  })
  const [syncLoading, setSyncLoading] = useState(false)
  const [syncStatus, setSyncStatus] = useState<string | null>(null)
  const [syncError, setSyncError] = useState<string | null>(null)

  const handleSettingsChange = (key: string, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }))
    lastActivityRef.current = Date.now()
  }

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<any>('get_settings')
      setSettings({
        encryptionEnabled: loadedSettings.encryption_enabled,
        autoLockMinutes: loadedSettings.auto_lock_minutes,
        maxItems: loadedSettings.max_items,
        retentionDays: loadedSettings.retention_days,
        refreshInterval: loadedSettings.refresh_interval,
        showNotifications: loadedSettings.show_notifications
      })

      // Check both whether a password exists and whether vault is currently locked
      const [pwExists, isLocked, loginEnabled] = await Promise.all([
        invoke<boolean>('has_master_password'),
        invoke<boolean>('check_vault_status'),
        invoke<boolean>('get_launch_at_login'),
      ])
      setHasPassword(pwExists)
      setVaultLocked(isLocked)
      vaultLockedRef.current = isLocked
      setLaunchAtLogin(loginEnabled)
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }

  const saveSettings = async () => {
    try {
      console.log('Saving settings:', settings)
      const result = await invoke('save_settings', {
        settings: {
          encryption_enabled: settings.encryptionEnabled,
          auto_lock_minutes: settings.autoLockMinutes,
          max_items: settings.maxItems,
          retention_days: settings.retentionDays,
          refresh_interval: settings.refreshInterval,
          show_notifications: settings.showNotifications
        }
      })
      console.log('Save result:', result)
      setShowSettings(false)
    } catch (e) {
      console.error('Failed to save settings:', e)
      alert('Failed to save settings: ' + e)
    }
  }

  const handleSetMasterPassword = async () => {
    if (masterPassword !== confirmPassword) {
      setPasswordError('Passwords do not match')
      return
    }
    if (masterPassword.length < 8) {
      setPasswordError('Password must be at least 8 characters')
      return
    }
    setPasswordError(null)

    try {
      await invoke('set_master_password', { password: masterPassword })
      setMasterPassword('')
      setConfirmPassword('')
      setHasPassword(true)
      setVaultLocked(true)
      vaultLockedRef.current = true
    } catch (e) {
      setPasswordError(String(e).replace('Daemon error: ', ''))
    }
  }

  const handleUnlockVault = async () => {
    setUnlockError(null)
    try {
      await invoke('unlock_vault', { password: unlockPassword })
      setUnlockPassword('')
      setVaultLocked(false)
      vaultLockedRef.current = false
      lastActivityRef.current = Date.now()
      loadClipboardHistory(false)
    } catch (e) {
      setUnlockError(String(e).replace('Daemon error: ', ''))
    }
  }

  const handleLockVault = async () => {
    try {
      await invoke('lock_vault')
      setVaultLocked(true)
      vaultLockedRef.current = true
      // Reload to show encrypted placeholders
      loadClipboardHistory(false)
    } catch (e) {
      console.error('Failed to lock vault:', e)
    }
  }

  const updateActivity = () => {
    lastActivityRef.current = Date.now()
  }

  // ── Tag helpers ──────────────────────────────────────────────────────────
  const loadTags = async () => {
    try {
      const tags = await invoke<Tag[]>('list_tags')
      setAllTags(tags)
    } catch (e) {
      console.error('Failed to load tags:', e)
    }
  }

  const loadItemTags = async (itemId: number) => {
    try {
      const tags = await invoke<Tag[]>('get_item_tags', { itemId })
      setItemTags(prev => ({ ...prev, [itemId]: tags }))
    } catch (e) {
      console.error('Failed to load item tags:', e)
    }
  }

  const loadAllItemTags = async (itemList: ClipboardItem[]) => {
    // Batch load tags for visible items
    const results = await Promise.allSettled(
      itemList.map(item =>
        invoke<Tag[]>('get_item_tags', { itemId: item.id })
          .then(tags => ({ id: item.id, tags }))
      )
    )
    const map: Record<number, Tag[]> = {}
    results.forEach(r => {
      if (r.status === 'fulfilled') map[r.value.id] = r.value.tags
    })
    setItemTags(map)
  }

  const handleAddTag = async (item: ClipboardItem, tagName: string) => {
    const name = tagName.trim()
    if (!name) return
    // Pick a color based on name hash
    const colors = ['#6D7FFF', '#47C267', '#E5C76B', '#FF7A72', '#C47AFF', '#5BBFFF', '#FF9F5A']
    const color = colors[name.charCodeAt(0) % colors.length]
    try {
      await invoke('add_tag_to_item', { itemId: item.id, tagName: name, color })
      await Promise.all([loadItemTags(item.id), loadTags()])
    } catch (e) {
      console.error('Failed to add tag:', e)
    }
  }

  const handleRemoveTag = async (itemId: number, tagId: number) => {
    try {
      await invoke('remove_tag_from_item', { itemId, tagId })
      await loadItemTags(itemId)
    } catch (e) {
      console.error('Failed to remove tag:', e)
    }
  }

  const handleDeleteTag = async (tagId: number) => {
    try {
      await invoke('delete_tag', { id: tagId })
      if (activeTagFilter === tagId) setActiveTagFilter(null)
      await loadTags()
      // Refresh item tags since some may have been removed
      setItemTags({})
    } catch (e) {
      console.error('Failed to delete tag:', e)
    }
  }

  // ── Plugin helpers ───────────────────────────────────────────────────────
  const loadPlugins = async () => {
    setPluginLoading(true)
    setPluginError(null)
    try {
      const list = await invoke<PluginInfo[]>('list_plugins')
      setPlugins(list)
    } catch (e) {
      setPluginError(String(e))
    } finally {
      setPluginLoading(false)
    }
  }

  const handleLoadPlugin = async () => {
    setPluginError(null)
    try {
      const selected = await openDialog({
        title: 'Select a WASM plugin',
        filters: [{ name: 'WASM Plugin', extensions: ['wasm'] }],
        multiple: false,
        directory: false,
      })
      if (!selected || Array.isArray(selected)) return
      const name = await invoke<string>('load_plugin', { path: selected })
      setSyncStatus(null)
      setPluginError(null)
      await loadPlugins()
      setPluginError(null)
      console.log('Loaded plugin:', name)
    } catch (e) {
      setPluginError(String(e))
    }
  }

  const handleUnloadPlugin = async (name: string) => {
    setPluginError(null)
    try {
      await invoke('unload_plugin', { name })
      await loadPlugins()
    } catch (e) {
      setPluginError(String(e))
    }
  }

  // ── Sync helpers ─────────────────────────────────────────────────────────
  const loadSyncConfig = async () => {
    setSyncLoading(true)
    setSyncError(null)
    try {
      const config = await invoke<SyncConfig>('get_sync_config')
      setSyncConfig(config)
    } catch (e) {
      setSyncError(String(e))
    } finally {
      setSyncLoading(false)
    }
  }

  const handleSaveSyncConfig = async () => {
    setSyncLoading(true)
    setSyncError(null)
    setSyncStatus(null)
    try {
      await invoke('set_sync_config', {
        serverUrl: syncConfig.server_url,
        apiToken: syncConfig.api_token || null,
        enabled: syncConfig.enabled,
      })
      setSyncStatus('Sync settings saved.')
    } catch (e) {
      setSyncError(String(e))
    } finally {
      setSyncLoading(false)
    }
  }

  const handleSyncNow = async () => {
    setSyncLoading(true)
    setSyncError(null)
    setSyncStatus(null)
    try {
      const result = await invoke<string>('sync_now')
      setSyncStatus(`Sync complete — ${result}`)
      await loadSyncConfig() // refresh last_sync_at
    } catch (e) {
      setSyncError(String(e))
    } finally {
      setSyncLoading(false)
    }
  }

  // Update ref whenever items changes
  useEffect(() => {
    itemsRef.current = items
    if (items.length > 0) {
      loadAllItemTags(items)
    }
  }, [items])

  // Run once on mount
  useEffect(() => {
    loadClipboardHistory(true)
    loadSettings()
    loadTags()
  }, [])

  // Auto-refresh interval — recreates only when interval setting changes
  useEffect(() => {
    const interval = setInterval(() => {
      if (!searchQuery && activeTagFilter === null) {
        loadClipboardHistory(false)
      }
    }, settings.refreshInterval * 1000)
    return () => clearInterval(interval)
  }, [searchQuery, settings.refreshInterval, activeTagFilter])

  // When tag filter changes, reload items
  useEffect(() => {
    if (activeTagFilter !== null) {
      invoke<ClipboardItem[]>('list_items_by_tag', { tagId: activeTagFilter })
        .then(setItems)
        .catch(console.error)
    } else if (!searchQuery.trim()) {
      loadClipboardHistory(false)
    }
  }, [activeTagFilter])

  // Auto-lock timer — recreates when encryption/lock settings change
  useEffect(() => {
    const autoLockInterval = setInterval(() => {
      if (settings.encryptionEnabled && !vaultLockedRef.current && settings.autoLockMinutes > 0) {
        const inactiveTime = Date.now() - lastActivityRef.current
        const lockThreshold = settings.autoLockMinutes * 60 * 1000
        if (inactiveTime > lockThreshold) {
          handleLockVault()
        }
      }
    }, 10_000)
    return () => clearInterval(autoLockInterval)
  }, [settings.encryptionEnabled, settings.autoLockMinutes])

  // Debounced search
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current)
    }

    if (searchQuery.trim()) {
      searchTimeoutRef.current = setTimeout(() => {
        performSearch(searchQuery)
      }, 300)
    } else {
      // Load full history when search is empty
      loadClipboardHistory(false)
    }

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current)
      }
    }
  }, [searchQuery])

  // Reset selected index when items change
  useEffect(() => {
    setSelectedIndex(-1)
  }, [items])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't handle if typing in search input
      if (document.activeElement === searchInputRef.current) {
        if (e.key === 'Escape') {
          searchInputRef.current?.blur()
          setSearchQuery('')
        }
        return
      }

      // Cmd/Ctrl+K to focus search
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        searchInputRef.current?.focus()
        return
      }

      // Escape to close modal or clear selection
      if (e.key === 'Escape') {
        if (selectedItem) {
          setSelectedItem(null)
        } else {
          setSelectedIndex(-1)
        }
        return
      }

      // Navigation
      if (e.key === 'ArrowDown' || e.key === 'j') {
        e.preventDefault()
        setSelectedIndex(prev => {
          const maxIndex = items.length - 1
          return prev < maxIndex ? prev + 1 : prev
        })
      } else if (e.key === 'ArrowUp' || e.key === 'k') {
        e.preventDefault()
        setSelectedIndex(prev => prev > 0 ? prev - 1 : -1)
      } else if (e.key === 'Enter' && selectedIndex >= 0) {
        e.preventDefault()
        const item = items[selectedIndex]
        if (item) {
          copyToClipboard(item.content)
        }
      } else if ((e.metaKey || e.ctrlKey) && e.key === 'c' && selectedIndex >= 0) {
        e.preventDefault()
        const item = items[selectedIndex]
        if (item) {
          copyToClipboard(item.content)
        }
      } else if (e.key === 'p' && selectedIndex >= 0) {
        e.preventDefault()
        const item = items[selectedIndex]
        if (item) {
          togglePin(item)
        }
      } else if (e.key === 'f' && selectedIndex >= 0) {
        e.preventDefault()
        const item = items[selectedIndex]
        if (item) {
          toggleFavorite(item)
        }
      } else if (e.key === 'd' && selectedIndex >= 0) {
        e.preventDefault()
        const item = items[selectedIndex]
        if (item) {
          deleteItem(item.id)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [items, selectedIndex, selectedItem])

  // Close context menu on click outside
  useEffect(() => {
    const close = () => setContextMenu(null)
    window.addEventListener('click', close)
    return () => window.removeEventListener('click', close)
  }, [])

  const loadClipboardHistory = async (isInitial: boolean = false) => {
    try {
      if (isInitial) {
        setLoading(true)
      }
      setError(null)

      const history = await invoke<ClipboardItem[]>('get_clipboard_history')

      // Only update if data actually changed (compare by ID and hash)
      const currentIds = itemsRef.current.map(item => `${item.id}-${item.hash}`).join(',')
      const newIds = history.map(item => `${item.id}-${item.hash}`).join(',')

      if (currentIds !== newIds) {
        setItems(history)
      }
    } catch (e) {
      setError(e as string)
      console.error('Failed to load clipboard history:', e)
    } finally {
      if (isInitial) {
        setLoading(false)
      }
    }
  }

  const performSearch = async (query: string) => {
    try {
      setLoading(true)
      setError(null)

      const results = await invoke<ClipboardItem[]>('search_clipboard_items', { query })
      setItems(results)
    } catch (e) {
      setError(e as string)
      console.error('Failed to search clipboard items:', e)
    } finally {
      setLoading(false)
    }
  }

  const copyToClipboard = async (content: string) => {
    try {
      await invoke('set_clipboard_content', { content })
    } catch (e) {
      console.error('Failed to copy to clipboard:', e)
    }
  }

  const deleteItem = async (id: number) => {
    try {
      await invoke('delete_clipboard_item', { id })
      loadClipboardHistory()
    } catch (e) {
      console.error('Failed to delete item:', e)
    }
  }

  const togglePin = async (item: ClipboardItem) => {
    try {
      await invoke('toggle_pin_item', { id: String(item.id) })
      loadClipboardHistory(false)
    } catch (e) {
      console.error('Failed to toggle pin:', e)
    }
  }

  const toggleFavorite = async (item: ClipboardItem) => {
    try {
      await invoke('toggle_favorite_item', { id: String(item.id) })
      loadClipboardHistory(false)
    } catch (e) {
      console.error('Failed to toggle favorite:', e)
    }
  }

  const getIconForType = (contentType: string) => {
    const type = contentType.toLowerCase()
    if (type === 'encrypted') return Pin  // lock icon substitute
    if (type.includes('image')) return Image
    if (type.includes('code') || type.includes('json') || type.includes('html')) return Terminal
    return FileText
  }

  const getTypeBadge = (contentType: string, content: string) => {
    const type = contentType.toLowerCase()
    if (type === 'encrypted') return 'ENCRYPTED'
    if (type.includes('image')) return 'IMAGE'
    if (type.includes('code') || type.includes('json') || type.includes('html')) return 'CODE'
    const trimmed = content.trim()
    if (trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('ftp://')) return 'URL'
    return 'TEXT'
  }

  const getRelativeTime = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffSecs = Math.floor(diffMs / 1000)
    const diffMins = Math.floor(diffSecs / 60)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffSecs < 60) return `${diffSecs} sec ago`
    if (diffMins < 60) return `${diffMins} min ago`
    if (diffHours < 24) return `${diffHours} hr ago`
    return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`
  }

  const filteredItems = items

  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      backgroundColor: '#171A1F',
      fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      color: '#F3F5F8',
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden'
    }}>
      {/* Title Bar */}
      <div 
        className="title-bar"
        style={{
          height: '48px',
          background: 'linear-gradient(180deg, #32363D 0%, #272B31 100%)',
          borderBottom: '1px solid #1C1F24',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          position: 'relative',
          flexShrink: 0,
          boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.06)'
        }}
      >
        {/* Window Controls */}
        <div style={{
          position: 'absolute',
          left: '12px',
          display: 'flex',
          gap: '12px'
        }}>
          <button
            onClick={() => appWindow.minimize()}
            className="window-control"
            style={{
              width: '15px',
              height: '15px',
              borderRadius: '50%',
              backgroundColor: '#FEBC2E',
              border: 'none',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 1px 2px rgba(0,0,0,0.3)'
            }}
          >
            <Minimize size={10} color="#1a1a1a" />
          </button>
          <button
            onClick={() => appWindow.toggleMaximize()}
            className="window-control"
            style={{
              width: '15px',
              height: '15px',
              borderRadius: '50%',
              backgroundColor: '#28C840',
              border: 'none',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 1px 2px rgba(0,0,0,0.3)'
            }}
          >
            <Maximize2 size={10} color="#1a1a1a" />
          </button>
          <button
            onClick={() => appWindow.close()}
            className="window-control"
            style={{
              width: '15px',
              height: '15px',
              borderRadius: '50%',
              backgroundColor: '#FF5F57',
              border: 'none',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 1px 2px rgba(0,0,0,0.3)'
            }}
          >
            <X size={10} color="#1a1a1a" />
          </button>
        </div>

        {/* Title */}
        <h1 style={{
          fontSize: '18px',
          fontWeight: '600',
          color: '#ECECEC',
          margin: 0
        }}>
          OpenPaste
        </h1>
      </div>

      {/* Hero Header */}
      <div style={{
        height: '100px',
        padding: '24px 32px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        flexShrink: 0
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <Clipboard size={36} color="#F3F4F6" />
          <div>
            <h2 style={{
              fontSize: '32px',
              fontWeight: '700',
              color: '#F3F4F6',
              margin: '0 0 4px 0',
              lineHeight: 1
            }}>
              OpenPaste
            </h2>
            <p style={{
              fontSize: '14px',
              fontWeight: '400',
              color: '#A6AEB8',
              margin: 0
            }}>
              Clipboard history at your fingertips.
            </p>
          </div>
        </div>

        {/* Toolbar Icons */}
        <div style={{ display: 'flex', gap: '20px' }}>
          <Search size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} onClick={() => {
            searchInputRef.current?.focus()
            updateActivity()
          }} />
          <Pin size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} onClick={() => updateActivity()} />
          <span title="Plugins" style={{ display: 'flex', cursor: 'pointer' }} onClick={() => {
            setShowPlugins(true)
            loadPlugins()
            updateActivity()
          }}>
            <Plug size={20} color="#C8CDD4" />
          </span>
          <span title="Sync" style={{ display: 'flex', cursor: 'pointer' }} onClick={() => {
            setShowSync(true)
            loadSyncConfig()
            updateActivity()
          }}>
            <Wifi size={20} color="#C8CDD4" />
          </span>
          <Settings size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} onClick={() => {
            setShowSettings(true)
            updateActivity()
          }} />
          <Menu size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} onClick={() => updateActivity()} />
        </div>
      </div>

      {/* Search + List + Tag Sidebar */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>

        {/* Tag Sidebar */}
        {allTags.length > 0 && (
          <div style={{
            width: '160px',
            flexShrink: 0,
            borderRight: '1px solid #2D333D',
            backgroundColor: '#171A1F',
            overflowY: 'auto',
            padding: '12px 8px',
            display: 'flex',
            flexDirection: 'column',
            gap: '4px'
          }}>
            <div style={{ fontSize: '11px', fontWeight: '600', color: '#7D8793', textTransform: 'uppercase', padding: '0 8px 8px' }}>
              Tags
            </div>
            <div
              onClick={() => setActiveTagFilter(null)}
              style={{
                padding: '6px 10px',
                borderRadius: '6px',
                fontSize: '13px',
                fontWeight: '500',
                cursor: 'pointer',
                backgroundColor: activeTagFilter === null ? '#262B31' : 'transparent',
                color: activeTagFilter === null ? '#F3F5F8' : '#A7B0BC',
              }}
            >
              All items
            </div>
            {allTags.map(tag => (
              <div
                key={tag.id}
                style={{ display: 'flex', alignItems: 'center', gap: '4px', borderRadius: '6px',
                         backgroundColor: activeTagFilter === tag.id ? '#262B31' : 'transparent' }}
              >
                <div
                  onClick={() => setActiveTagFilter(activeTagFilter === tag.id ? null : tag.id)}
                  style={{
                    flex: 1,
                    padding: '6px 10px',
                    borderRadius: '6px',
                    fontSize: '13px',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    color: activeTagFilter === tag.id ? '#F3F5F8' : '#A7B0BC',
                  }}
                >
                  <span style={{
                    width: '8px', height: '8px', borderRadius: '50%', flexShrink: 0,
                    backgroundColor: tag.color ?? '#6D7FFF'
                  }} />
                  <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {tag.name}
                  </span>
                </div>
                <button
                  onClick={(e) => { e.stopPropagation(); handleDeleteTag(tag.id) }}
                  style={{
                    background: 'none', border: 'none', cursor: 'pointer',
                    color: '#5A6471', padding: '2px 6px', fontSize: '14px', lineHeight: 1,
                    borderRadius: '4px',
                  }}
                  title="Delete tag"
                >×</button>
              </div>
            ))}
          </div>
        )}

        {/* Right pane: Search + List */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>

          {/* Search Section */}
          <div style={{ padding: '0 32px 20px 32px', flexShrink: 0 }}>
            <div style={{
              height: '48px',
              backgroundColor: '#1F242B',
              border: '1px solid #3A414C',
              borderRadius: '12px',
              display: 'flex',
              alignItems: 'center',
              padding: '0 16px',
              boxShadow: '0 4px 12px rgba(0,0,0,0.25), inset 0 1px 0 rgba(255,255,255,0.03)',
              position: 'relative'
            }}>
              <Search size={18} color="#8E97A4" />
              <input
                ref={searchInputRef}
                type="text"
                placeholder="Search clipboard..."
                value={searchQuery}
                onChange={(e) => {
                  setSearchQuery(e.target.value)
                  updateActivity()
                }}
                style={{
                  flex: 1,
                  marginLeft: '12px',
                  fontSize: '14px',
                  color: '#F3F5F8',
                  background: 'transparent',
                  border: 'none',
                  outline: 'none',
                  fontFamily: 'Inter, sans-serif'
                }}
              />
              <div style={{
                backgroundColor: '#282D35',
                border: '1px solid #454C56',
                borderRadius: '6px',
                padding: '6px 10px',
                fontSize: '12px',
                fontWeight: '500',
                color: '#A7B0BC'
              }}>
                ⌘K
              </div>
            </div>
          </div>

          {/* Clipboard List */}
          <div style={{ 
            flex: 1,
            overflowY: 'auto',
            padding: '0 32px',
            overflowX: 'hidden'
          }}>
        {loading && (
          <div style={{ textAlign: 'center', color: '#7D8793', padding: '32px 0' }}>
            Loading clipboard history...
          </div>
        )}

        {error && (
          <div style={{ textAlign: 'center', color: '#E05D5D', padding: '32px 0' }}>
            Error: {error}
          </div>
        )}

        {!loading && !error && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {filteredItems.map((item, index) => {
              const IconComponent = getIconForType(item.content_type)
              const isSelected = index === selectedIndex
              return (
                <div
                  key={item.id}
                  style={{
                    minHeight: '80px',
                    background: isSelected
                      ? 'linear-gradient(180deg, #3A414C 0%, #2F363F 100%)'
                      : 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
                    border: isSelected ? '2px solid #6D7FFF' : '1px solid #343A44',
                    borderRadius: '10px',
                    padding: '14px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '14px',
                    boxShadow: isSelected
                      ? '0 6px 12px rgba(0,0,0,0.28), inset 0 1px 0 rgba(255,255,255,0.06)'
                      : '0 4px 8px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04), inset 0 -1px 0 rgba(0,0,0,0.30)',
                    transition: 'transform 120ms ease-out, box-shadow 120ms ease-out, border-color 120ms ease-out',
                    cursor: 'pointer'
                  }}
                  onMouseEnter={(e) => {
                    setSelectedIndex(index)
                    e.currentTarget.style.transform = 'translateY(-1px)'
                    e.currentTarget.style.borderColor = '#5A6471'
                    e.currentTarget.style.boxShadow = '0 6px 12px rgba(0,0,0,0.28)'
                  }}
                  onMouseLeave={(e) => {
                    setSelectedIndex(-1)
                    e.currentTarget.style.transform = 'translateY(0)'
                    e.currentTarget.style.borderColor = '#343A44'
                    e.currentTarget.style.boxShadow = '0 4px 8px rgba(0,0,0,0.22)'
                  }}
                  onClick={() => {
                    if (item.content_type !== 'encrypted') {
                      setSelectedItem(item)
                    }
                    updateActivity()
                  }}
                  onContextMenu={(e) => {
                    e.preventDefault()
                    e.stopPropagation()
                    setContextMenu({ x: e.clientX, y: e.clientY, item })
                  }}
                >
                  {/* Left Icon Area */}
                  <div style={{
                    width: '48px',
                    height: '48px',
                    backgroundColor: item.content_type === 'encrypted' ? '#2A1F1F' : '#232831',
                    border: `1px solid ${item.content_type === 'encrypted' ? '#5A3030' : '#3A404B'}`,
                    borderRadius: '10px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0
                  }}>
                    {item.content_type === 'encrypted'
                      ? <span style={{ fontSize: '20px' }}>🔒</span>
                      : <IconComponent size={20} color="#8EA2FF" />
                    }
                  </div>

                  {/* Content Area */}
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{
                      fontSize: '12px',
                      fontWeight: '500',
                      color: '#9AA3AF',
                      marginBottom: '4px'
                    }}>
                      {new Date(item.created_at).toLocaleString()}
                    </div>
                    {item.content_type === 'encrypted' ? (
                      <div style={{
                        fontSize: '14px',
                        fontWeight: '500',
                        color: '#7D8793',
                        fontStyle: 'italic',
                        marginBottom: '6px'
                      }}>
                        🔒 Encrypted — unlock vault to view
                      </div>
                    ) : item.content_type === 'image' ? (
                      <img
                        src={`data:image/png;base64,${item.content}`}
                        alt="Clipboard image"
                        style={{
                          maxHeight: '36px',
                          maxWidth: '120px',
                          objectFit: 'cover',
                          borderRadius: '4px',
                          marginBottom: '6px',
                          display: 'block'
                        }}
                      />
                    ) : (
                      <div style={{
                        fontSize: '14px',
                        fontWeight: '600',
                        color: '#F3F5F8',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        marginBottom: '6px'
                      }}>
                        {item.content}
                      </div>
                    )}
                    <div style={{
                      display: 'inline-block',
                      height: '20px',
                      padding: '0 8px',
                      backgroundColor: item.content_type === 'encrypted' ? '#2A1A1A' : '#29314A',
                      border: `1px solid ${item.content_type === 'encrypted' ? '#5A2020' : '#465273'}`,
                      borderRadius: '4px',
                      fontSize: '10px',
                      fontWeight: '600',
                      color: item.content_type === 'encrypted' ? '#AA5555' : '#8EA2FF',
                      textTransform: 'uppercase',
                      lineHeight: '20px'
                    }}>
                      {getTypeBadge(item.content_type, item.content)}
                    </div>
                    {/* Tag pills */}
                    {(itemTags[item.id] ?? []).length > 0 && (
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px', marginTop: '5px' }}>
                        {(itemTags[item.id] ?? []).map(tag => (
                          <span
                            key={tag.id}
                            style={{
                              display: 'inline-flex',
                              alignItems: 'center',
                              gap: '3px',
                              padding: '2px 7px',
                              borderRadius: '10px',
                              fontSize: '11px',
                              fontWeight: '500',
                              backgroundColor: `${tag.color ?? '#6D7FFF'}22`,
                              border: `1px solid ${tag.color ?? '#6D7FFF'}55`,
                              color: tag.color ?? '#6D7FFF',
                              cursor: 'default',
                            }}
                            onClick={(e) => { e.stopPropagation(); setActiveTagFilter(tag.id) }}
                          >
                            {tag.name}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>

                  {/* Vertical Separator */}
                  <div style={{
                    width: '1px',
                    height: '60px',
                    backgroundColor: '#343B45',
                    flexShrink: 0
                  }} />

                  {/* Right Side - Actions */}
                  <div style={{
                    width: '140px',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    gap: '6px',
                    flexShrink: 0
                  }}>
                    <div style={{
                      fontSize: '12px',
                      fontWeight: '500',
                      color: '#A8B0BA'
                    }}>
                      {getRelativeTime(item.created_at)}
                    </div>
                    <div style={{ display: 'flex', gap: '16px' }}>
                      <Copy 
                        size={16} 
                        color={item.content_type === 'encrypted' ? '#3A414C' : '#6D7FFF'}
                        style={{ cursor: item.content_type === 'encrypted' ? 'not-allowed' : 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          if (item.content_type !== 'encrypted') {
                            copyToClipboard(item.content)
                            updateActivity()
                          }
                        }}
                      />
                      <Pin 
                        size={16} 
                        color={item.pinned ? "#6D7FFF" : "#A7B0BC"} 
                        fill={item.pinned ? "#6D7FFF" : "none"}
                        style={{ cursor: 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          togglePin(item)
                          updateActivity()
                        }}
                      />
                      <Star 
                        size={16} 
                        color={item.favorite ? "#E5C76B" : "#A7B0BC"} 
                        fill={item.favorite ? "#E5C76B" : "none"}
                        style={{ cursor: 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          toggleFavorite(item)
                          updateActivity()
                        }}
                      />
                      <Trash2 
                        size={16} 
                        color="#FF7A72" 
                        style={{ cursor: 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          deleteItem(item.id)
                          updateActivity()
                        }}
                      />
                    </div>
                  </div>
                </div>
              )
            })}

            {filteredItems.length === 0 && (
              <div style={{ textAlign: 'center', color: '#7D8793', padding: '32px 0' }}>
                No clipboard items found
              </div>
            )}
          </div>
        )}
          </div> {/* end Clipboard List */}

        </div> {/* end right pane */}
      </div> {/* end Search + List + Tag Sidebar */}

      {/* Bottom Status Bar */}
      <div style={{
        height: '36px',
        backgroundColor: '#1A1E24',
        borderTop: '1px solid #2D333D',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 20px',
        flexShrink: 0,
        gap: '12px',
      }}>
        {/* Left — stats */}
        <div style={{ display: 'flex', gap: '16px', flexShrink: 0 }}>
          <span style={{ fontSize: '12px', color: '#7D8793' }}>
            <span style={{ color: '#A7B0BC', fontWeight: '600' }}>{filteredItems.length}</span> items
          </span>
          <span style={{ fontSize: '12px', color: '#7D8793' }}>
            <span style={{ color: '#A7B0BC', fontWeight: '600' }}>{items.filter(i => i.pinned).length}</span> pinned
          </span>
          <span style={{ fontSize: '12px', color: '#7D8793' }}>
            <span style={{ color: '#A7B0BC', fontWeight: '600' }}>{items.filter(i => i.favorite).length}</span> fav
          </span>
          <span style={{ fontSize: '12px', color: '#7D8793' }}>
            <span style={{ color: '#A7B0BC', fontWeight: '600' }}>{items.filter(i => i.content_type === 'image').length}</span> img
          </span>
        </div>

        {/* Right — vault badge + synced + shortcuts hint */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexShrink: 0 }}>
          {settings.encryptionEnabled && hasPassword && (
            <div
              onClick={vaultLocked ? () => { setShowSettings(true); updateActivity() } : handleLockVault}
              style={{
                display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer',
                padding: '2px 7px', borderRadius: '5px',
                backgroundColor: vaultLocked ? 'rgba(255,122,114,0.12)' : 'rgba(71,194,103,0.12)',
                border: `1px solid ${vaultLocked ? 'rgba(255,122,114,0.3)' : 'rgba(71,194,103,0.3)'}`,
              }}
              title={vaultLocked ? 'Vault locked — click to unlock' : 'Vault unlocked — click to lock'}
            >
              <span style={{ fontSize: '11px' }}>{vaultLocked ? '🔒' : '🔓'}</span>
              <span style={{ fontSize: '11px', fontWeight: '500', color: vaultLocked ? '#FF7A72' : '#47C267' }}>
                {vaultLocked ? 'Locked' : 'Unlocked'}
              </span>
            </div>
          )}

          <div style={{ display: 'flex', alignItems: 'center', gap: '5px' }}>
            <Check size={12} color="#47C267" />
            <span style={{ fontSize: '12px', color: '#47C267' }}>Synced</span>
          </div>

          {/* Shortcuts tooltip trigger */}
          <div style={{ position: 'relative' }} className="shortcuts-hint">
            <div style={{
              width: '18px', height: '18px', borderRadius: '50%',
              border: '1px solid #3A414C', display: 'flex', alignItems: 'center',
              justifyContent: 'center', cursor: 'default', fontSize: '11px',
              color: '#7D8793', userSelect: 'none',
            }}>?</div>
            {/* Tooltip */}
            <div className="shortcuts-tooltip" style={{
              position: 'absolute', bottom: '26px', right: 0,
              backgroundColor: '#262B31', border: '1px solid #3A414C',
              borderRadius: '8px', padding: '10px 14px',
              display: 'none', flexDirection: 'column', gap: '5px',
              whiteSpace: 'nowrap', zIndex: 500,
              boxShadow: '0 4px 16px rgba(0,0,0,0.4)',
              minWidth: '200px',
            }}>
              {[
                ['⌘⇧V', 'Show / hide window'],
                ['⌘⇧C', 'Quick paste latest'],
                ['⌘K', 'Focus search'],
                ['↑ ↓', 'Navigate'],
                ['Enter', 'Copy selected'],
                ['P', 'Pin selected'],
                ['F', 'Favorite selected'],
                ['D', 'Delete selected'],
              ].map(([key, desc]) => (
                <div key={key} style={{ display: 'flex', justifyContent: 'space-between', gap: '20px' }}>
                  <span style={{ fontSize: '12px', color: '#8EA2FF', fontFamily: 'monospace' }}>{key}</span>
                  <span style={{ fontSize: '12px', color: '#A7B0BC' }}>{desc}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <div
          style={{
            position: 'fixed',
            top: contextMenu.y,
            left: contextMenu.x,
            zIndex: 2000,
            backgroundColor: '#262B31',
            border: '1px solid #3A414C',
            borderRadius: '8px',
            boxShadow: '0 8px 24px rgba(0,0,0,0.4)',
            minWidth: '200px',
            padding: '6px',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Current tags */}
          {(itemTags[contextMenu.item.id] ?? []).length > 0 && (
            <>
              <div style={{ padding: '4px 10px', fontSize: '11px', color: '#7D8793', textTransform: 'uppercase' }}>
                Tags
              </div>
              {(itemTags[contextMenu.item.id] ?? []).map(tag => (
                <div
                  key={tag.id}
                  style={{
                    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                    padding: '6px 10px', borderRadius: '6px', gap: '8px'
                  }}
                >
                  <span style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '13px', color: '#F3F5F8' }}>
                    <span style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: tag.color ?? '#6D7FFF', flexShrink: 0 }} />
                    {tag.name}
                  </span>
                  <button
                    onClick={() => { handleRemoveTag(contextMenu.item.id, tag.id); setContextMenu(null) }}
                    style={{ background: 'none', border: 'none', color: '#7D8793', cursor: 'pointer', fontSize: '14px' }}
                  >×</button>
                </div>
              ))}
              <div style={{ height: '1px', backgroundColor: '#343A44', margin: '4px 0' }} />
            </>
          )}
          {/* Add tag row */}
          {showTagInput ? (
            <div style={{ padding: '6px 10px', display: 'flex', gap: '6px' }}>
              <input
                autoFocus
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleAddTag(contextMenu.item, tagInput)
                    setTagInput('')
                    setShowTagInput(false)
                    setContextMenu(null)
                  }
                  if (e.key === 'Escape') {
                    setShowTagInput(false)
                    setTagInput('')
                  }
                }}
                placeholder="Tag name…"
                style={{
                  flex: 1, padding: '5px 8px', backgroundColor: '#1F242B',
                  border: '1px solid #3A414C', borderRadius: '5px',
                  color: '#F3F5F8', fontSize: '13px', outline: 'none'
                }}
              />
              <button
                onClick={() => {
                  handleAddTag(contextMenu.item, tagInput)
                  setTagInput('')
                  setShowTagInput(false)
                  setContextMenu(null)
                }}
                style={{
                  padding: '5px 10px', backgroundColor: '#6D7FFF', border: 'none',
                  borderRadius: '5px', color: '#fff', fontSize: '13px', cursor: 'pointer'
                }}
              >Add</button>
            </div>
          ) : (
            <div
              onClick={() => setShowTagInput(true)}
              style={{
                padding: '8px 10px', borderRadius: '6px', fontSize: '13px',
                color: '#A7B0BC', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '8px'
              }}
            >
              <span style={{ fontSize: '16px', lineHeight: 1 }}>+</span> Add tag
            </div>
          )}
          {/* Other actions */}
          <div style={{ height: '1px', backgroundColor: '#343A44', margin: '4px 0' }} />
          <div
            onClick={() => { copyToClipboard(contextMenu.item.content); setContextMenu(null) }}
            style={{
              padding: '8px 10px', borderRadius: '6px', fontSize: '13px',
              color: contextMenu.item.content_type === 'encrypted' ? '#5A6471' : '#A7B0BC',
              cursor: contextMenu.item.content_type === 'encrypted' ? 'not-allowed' : 'pointer'
            }}
          >
            Copy to clipboard
          </div>
          <div
            onClick={() => { togglePin(contextMenu.item); setContextMenu(null) }}
            style={{ padding: '8px 10px', borderRadius: '6px', fontSize: '13px', color: '#A7B0BC', cursor: 'pointer' }}
          >
            {contextMenu.item.pinned ? 'Unpin' : 'Pin'}
          </div>
          <div
            onClick={() => { toggleFavorite(contextMenu.item); setContextMenu(null) }}
            style={{ padding: '8px 10px', borderRadius: '6px', fontSize: '13px', color: '#A7B0BC', cursor: 'pointer' }}
          >
            {contextMenu.item.favorite ? 'Unfavorite' : 'Favorite'}
          </div>
          <div
            onClick={() => { deleteItem(contextMenu.item.id); setContextMenu(null) }}
            style={{ padding: '8px 10px', borderRadius: '6px', fontSize: '13px', color: '#FF7A72', cursor: 'pointer' }}
          >
            Delete
          </div>
        </div>
      )}

      {/* Settings Modal */}
      {showSettings && (
        <div style={{
          position: 'fixed',
          inset: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '1rem',
          zIndex: 1000
        }}>
          <div style={{
            background: 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
            border: '1px solid #343A44',
            borderRadius: '14px',
            padding: '24px',
            maxWidth: '500px',
            width: '100%',
            maxHeight: '80vh',
            overflow: 'auto',
            boxShadow: '0 8px 16px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04)'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h3 style={{ fontSize: '20px', fontWeight: '600', color: '#F3F5F8', margin: 0 }}>
                Settings
              </h3>
              <button
                onClick={() => {
                  setShowSettings(false)
                  updateActivity()
                }}
                style={{
                  backgroundColor: 'transparent',
                  border: 'none',
                  color: '#A7B0BC',
                  cursor: 'pointer',
                  fontSize: '24px',
                  padding: '4px'
                }}
              >
                ✕
              </button>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              {/* Encryption Section */}
              <div>
                <h4 style={{ fontSize: '16px', fontWeight: '600', color: '#F3F5F8', margin: '0 0 12px 0' }}>
                  Encryption
                </h4>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Enable encryption</span>
                    <div
                      onClick={() => handleSettingsChange('encryptionEnabled', !settings.encryptionEnabled)}
                      style={{
                        width: '44px',
                        height: '24px',
                        backgroundColor: settings.encryptionEnabled ? '#6D7FFF' : '#3A414C',
                        borderRadius: '12px',
                        position: 'relative',
                        cursor: 'pointer',
                        transition: 'background-color 0.2s'
                      }}
                    >
                      <div style={{
                        width: '20px',
                        height: '20px',
                        backgroundColor: '#F3F5F8',
                        borderRadius: '50%',
                        position: 'absolute',
                        top: '2px',
                        left: settings.encryptionEnabled ? '22px' : '2px',
                        transition: 'left 0.2s',
                        pointerEvents: 'none'
                      }} />
                    </div>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Auto-lock after (min)</span>
                    <input
                      type="number"
                      value={settings.autoLockMinutes}
                      onChange={(e) => handleSettingsChange('autoLockMinutes', parseInt(e.target.value) || 0)}
                      style={{
                        width: '60px',
                        padding: '6px 8px',
                        backgroundColor: '#1F242B',
                        border: '1px solid #3A414C',
                        borderRadius: '6px',
                        color: '#F3F5F8',
                        fontSize: '14px'
                      }}
                    />
                  </div>
                </div>
              </div>

              {/* Master Password Section */}
              <div>
                <h4 style={{ fontSize: '16px', fontWeight: '600', color: '#F3F5F8', margin: '0 0 12px 0' }}>
                  Master Password
                </h4>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  {!hasPassword ? (
                    /* No password configured — show set form */
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                      <span style={{ fontSize: '13px', color: '#7D8793' }}>
                        Set a master password to enable AES-256-GCM encryption.
                      </span>
                      <input
                        type="password"
                        placeholder="New password (min 8 chars)"
                        value={masterPassword}
                        onChange={(e) => { setMasterPassword(e.target.value); setPasswordError(null) }}
                        style={{
                          padding: '8px 12px',
                          backgroundColor: '#1F242B',
                          border: '1px solid #3A414C',
                          borderRadius: '6px',
                          color: '#F3F5F8',
                          fontSize: '14px'
                        }}
                      />
                      <input
                        type="password"
                        placeholder="Confirm password"
                        value={confirmPassword}
                        onChange={(e) => { setConfirmPassword(e.target.value); setPasswordError(null) }}
                        onKeyDown={(e) => { if (e.key === 'Enter') handleSetMasterPassword() }}
                        style={{
                          padding: '8px 12px',
                          backgroundColor: '#1F242B',
                          border: '1px solid #3A414C',
                          borderRadius: '6px',
                          color: '#F3F5F8',
                          fontSize: '14px'
                        }}
                      />
                      {passwordError && (
                        <span style={{ fontSize: '13px', color: '#FF7A72' }}>{passwordError}</span>
                      )}
                      <button
                        onClick={handleSetMasterPassword}
                        style={{
                          padding: '8px 16px',
                          backgroundColor: '#6D7FFF',
                          border: 'none',
                          borderRadius: '6px',
                          color: '#F3F5F8',
                          fontSize: '14px',
                          fontWeight: '500',
                          cursor: 'pointer',
                          alignSelf: 'flex-start'
                        }}
                      >
                        Set Password
                      </button>
                    </div>
                  ) : vaultLocked ? (
                    /* Password set, vault locked — show unlock form */
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span style={{ fontSize: '14px' }}>🔒</span>
                        <span style={{ fontSize: '14px', color: '#FF7A72', fontWeight: '500' }}>Vault is locked</span>
                      </div>
                      <input
                        type="password"
                        placeholder="Enter password to unlock"
                        value={unlockPassword}
                        onChange={(e) => { setUnlockPassword(e.target.value); setUnlockError(null) }}
                        onKeyDown={(e) => { if (e.key === 'Enter') handleUnlockVault() }}
                        style={{
                          padding: '8px 12px',
                          backgroundColor: '#1F242B',
                          border: `1px solid ${unlockError ? '#FF7A72' : '#3A414C'}`,
                          borderRadius: '6px',
                          color: '#F3F5F8',
                          fontSize: '14px'
                        }}
                      />
                      {unlockError && (
                        <span style={{ fontSize: '13px', color: '#FF7A72' }}>{unlockError}</span>
                      )}
                      <button
                        onClick={handleUnlockVault}
                        style={{
                          padding: '8px 16px',
                          backgroundColor: '#6D7FFF',
                          border: 'none',
                          borderRadius: '6px',
                          color: '#F3F5F8',
                          fontSize: '14px',
                          fontWeight: '500',
                          cursor: 'pointer',
                          alignSelf: 'flex-start'
                        }}
                      >
                        Unlock Vault
                      </button>
                    </div>
                  ) : (
                    /* Password set, vault unlocked — show lock button */
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span style={{ fontSize: '14px' }}>🔓</span>
                        <span style={{ fontSize: '14px', color: '#47C267', fontWeight: '500' }}>Vault is unlocked</span>
                      </div>
                      <span style={{ fontSize: '13px', color: '#7D8793' }}>
                        Clipboard data is being encrypted. Auto-lock after {settings.autoLockMinutes} min of inactivity.
                      </span>
                      <button
                        onClick={() => { handleLockVault(); setShowSettings(false) }}
                        style={{
                          padding: '8px 16px',
                          backgroundColor: '#3A414C',
                          border: '1px solid #FF7A72',
                          borderRadius: '6px',
                          color: '#FF7A72',
                          fontSize: '14px',
                          fontWeight: '500',
                          cursor: 'pointer',
                          alignSelf: 'flex-start'
                        }}
                      >
                        Lock Vault Now
                      </button>
                    </div>
                  )}

                </div>
              </div>

              {/* Retention Section */}
              <div>
                <h4 style={{ fontSize: '16px', fontWeight: '600', color: '#F3F5F8', margin: '0 0 12px 0' }}>
                  Data Retention
                </h4>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Max items</span>
                    <input
                      type="number"
                      value={settings.maxItems}
                      onChange={(e) => handleSettingsChange('maxItems', parseInt(e.target.value) || 0)}
                      style={{
                        width: '80px',
                        padding: '6px 8px',
                        backgroundColor: '#1F242B',
                        border: '1px solid #3A414C',
                        borderRadius: '6px',
                        color: '#F3F5F8',
                        fontSize: '14px'
                      }}
                    />
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Retention days</span>
                    <input
                      type="number"
                      value={settings.retentionDays}
                      onChange={(e) => handleSettingsChange('retentionDays', parseInt(e.target.value) || 0)}
                      style={{
                        width: '60px',
                        padding: '6px 8px',
                        backgroundColor: '#1F242B',
                        border: '1px solid #3A414C',
                        borderRadius: '6px',
                        color: '#F3F5F8',
                        fontSize: '14px'
                      }}
                    />
                  </div>
                </div>
              </div>

              {/* General Section */}
              <div>
                <h4 style={{ fontSize: '16px', fontWeight: '600', color: '#F3F5F8', margin: '0 0 12px 0' }}>
                  General
                </h4>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Auto-refresh interval (sec)</span>
                    <input
                      type="number"
                      value={settings.refreshInterval}
                      onChange={(e) => handleSettingsChange('refreshInterval', parseInt(e.target.value) || 0)}
                      style={{
                        width: '60px',
                        padding: '6px 8px',
                        backgroundColor: '#1F242B',
                        border: '1px solid #3A414C',
                        borderRadius: '6px',
                        color: '#F3F5F8',
                        fontSize: '14px'
                      }}
                    />
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Show notifications</span>
                    <div
                      onClick={() => handleSettingsChange('showNotifications', !settings.showNotifications)}
                      style={{
                        width: '44px',
                        height: '24px',
                        backgroundColor: settings.showNotifications ? '#6D7FFF' : '#3A414C',
                        borderRadius: '12px',
                        position: 'relative',
                        cursor: 'pointer',
                        transition: 'background-color 0.2s'
                      }}
                    >
                      <div style={{
                        width: '20px',
                        height: '20px',
                        backgroundColor: '#F3F5F8',
                        borderRadius: '50%',
                        position: 'absolute',
                        top: '2px',
                        left: settings.showNotifications ? '22px' : '2px',
                        transition: 'left 0.2s',
                        pointerEvents: 'none'
                      }} />
                    </div>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Launch at login</span>
                    <div
                      onClick={async () => {
                        const next = !launchAtLogin
                        try {
                          await invoke('set_launch_at_login', { enabled: next })
                          setLaunchAtLogin(next)
                        } catch (e) {
                          console.error('Failed to set launch at login:', e)
                        }
                      }}
                      style={{
                        width: '44px',
                        height: '24px',
                        backgroundColor: launchAtLogin ? '#6D7FFF' : '#3A414C',
                        borderRadius: '12px',
                        position: 'relative',
                        cursor: 'pointer',
                        transition: 'background-color 0.2s'
                      }}
                    >
                      <div style={{
                        width: '20px',
                        height: '20px',
                        backgroundColor: '#F3F5F8',
                        borderRadius: '50%',
                        position: 'absolute',
                        top: '2px',
                        left: launchAtLogin ? '22px' : '2px',
                        transition: 'left 0.2s',
                        pointerEvents: 'none'
                      }} />
                    </div>
                  </div>
                </div>
              </div>

              <div style={{
                display: 'flex',
                gap: '12px',
                marginTop: '8px',
                paddingTop: '16px',
                borderTop: '1px solid #343A44'
              }}>
                <button
                  onClick={() => {
                    saveSettings()
                    updateActivity()
                  }}
                  style={{
                    flex: 1,
                    padding: '10px 16px',
                    backgroundColor: '#6D7FFF',
                    border: 'none',
                    borderRadius: '8px',
                    color: '#F3F5F8',
                    fontSize: '14px',
                    fontWeight: '500',
                    cursor: 'pointer'
                  }}
                >
                  Save Changes
                </button>
                <button
                  onClick={() => {
                    setShowSettings(false)
                    updateActivity()
                  }}
                  style={{
                    flex: 1,
                    padding: '10px 16px',
                    backgroundColor: '#3A414C',
                    border: '1px solid #4A5260',
                    borderRadius: '8px',
                    color: '#F3F5F8',
                    fontSize: '14px',
                    fontWeight: '500',
                    cursor: 'pointer'
                  }}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Selected Item Detail Modal */}
      {selectedItem && (
        <div style={{
          position: 'fixed',
          inset: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '1rem',
          zIndex: 1000
        }}>
          <div style={{
            background: 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
            border: '1px solid #343A44',
            borderRadius: '14px',
            padding: '24px',
            maxWidth: '600px',
            width: '100%',
            maxHeight: '80vh',
            overflow: 'auto',
            boxShadow: '0 8px 16px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04)'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
              <h3 style={{ fontSize: '20px', fontWeight: '600', color: '#F3F5F8', margin: 0 }}>
                Clipboard Item
              </h3>
              <button
                onClick={() => {
                  setSelectedItem(null)
                  updateActivity()
                }}
                style={{
                  backgroundColor: 'transparent',
                  border: 'none',
                  color: '#A7B0BC',
                  cursor: 'pointer',
                  fontSize: '24px',
                  padding: '4px'
                }}
              >
                ✕
              </button>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <div>
                <span style={{ color: '#9AA3AF', fontSize: '14px' }}>Type:</span>{' '}
                <span style={{ color: '#F3F5F8', fontSize: '14px' }}>{selectedItem.content_type}</span>
              </div>
              <div>
                <span style={{ color: '#9AA3AF', fontSize: '14px' }}>Created:</span>{' '}
                <span style={{ color: '#F3F5F8', fontSize: '14px' }}>{new Date(selectedItem.created_at).toLocaleString()}</span>
              </div>
              <div>
                <span style={{ color: '#9AA3AF', fontSize: '14px' }}>Content:</span>
                {selectedItem.content_type === 'image' ? (
                  <div style={{ marginTop: '8px' }}>
                    <img
                      src={`data:image/png;base64,${selectedItem.content}`}
                      alt="Clipboard image"
                      style={{
                        maxWidth: '100%',
                        borderRadius: '8px',
                        border: '1px solid #343A44'
                      }}
                    />
                  </div>
                ) : (
                  <pre style={{
                    marginTop: '8px',
                    padding: '16px',
                    background: 'linear-gradient(180deg, #1F242B 0%, #1A1E24 100%)',
                    border: '1px solid #343A44',
                    borderRadius: '8px',
                    overflow: 'auto',
                    color: '#F3F5F8',
                    fontSize: '14px',
                    fontFamily: 'Inter, monospace',
                    whiteSpace: 'pre-wrap',
                    wordWrap: 'break-word'
                  }}>
                    {selectedItem.content}
                  </pre>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── Plugins Modal ──────────────────────────────────────────────────── */}
      {showPlugins && (
        <div style={{
          position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.7)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          padding: '1rem', zIndex: 1000,
        }}>
          <div style={{
            background: 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
            border: '1px solid #343A44', borderRadius: '14px', padding: '24px',
            maxWidth: '520px', width: '100%', maxHeight: '80vh', overflow: 'auto',
            boxShadow: '0 8px 16px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04)',
          }}>
            {/* Header */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <Package size={20} color="#8EA2FF" />
                <h3 style={{ fontSize: '20px', fontWeight: '600', color: '#F3F5F8', margin: 0 }}>
                  Plugins
                </h3>
              </div>
              <button onClick={() => setShowPlugins(false)} style={{
                background: 'none', border: 'none', color: '#A7B0BC', cursor: 'pointer', fontSize: '24px', padding: '4px',
              }}>✕</button>
            </div>

            <p style={{ fontSize: '13px', color: '#7D8793', margin: '0 0 16px 0', lineHeight: 1.5 }}>
              Plugins are WebAssembly modules that transform clipboard text. Drop a{' '}
              <code style={{ color: '#8EA2FF', backgroundColor: '#1F242B', padding: '1px 5px', borderRadius: '3px' }}>.wasm</code>{' '}
              file in <code style={{ color: '#8EA2FF', backgroundColor: '#1F242B', padding: '1px 5px', borderRadius: '3px' }}>~/.local/share/openpaste/plugins/</code>{' '}
              or load one manually below.
            </p>

            {/* Error banner */}
            {pluginError && (
              <div style={{
                padding: '10px 14px', backgroundColor: 'rgba(255,122,114,0.1)',
                border: '1px solid rgba(255,122,114,0.3)', borderRadius: '8px',
                color: '#FF7A72', fontSize: '13px', marginBottom: '16px',
              }}>
                {pluginError}
              </div>
            )}

            {/* Load button */}
            <button
              onClick={handleLoadPlugin}
              style={{
                display: 'flex', alignItems: 'center', gap: '8px',
                padding: '9px 16px', backgroundColor: '#6D7FFF',
                border: 'none', borderRadius: '8px',
                color: '#F3F5F8', fontSize: '14px', fontWeight: '500',
                cursor: 'pointer', marginBottom: '20px',
              }}
            >
              <Upload size={15} />
              Load Plugin (.wasm)
            </button>

            {/* Plugin list */}
            {pluginLoading ? (
              <div style={{ color: '#7D8793', fontSize: '13px', padding: '8px 0' }}>Loading…</div>
            ) : plugins.length === 0 ? (
              <div style={{
                textAlign: 'center', padding: '32px 0', color: '#5A6471', fontSize: '13px',
              }}>
                <Package size={32} color="#3A414C" style={{ marginBottom: '10px', display: 'block', margin: '0 auto 10px' }} />
                No plugins loaded. Plugins are auto-loaded from the plugins directory on daemon start.
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {plugins.map(plugin => (
                  <div key={plugin.name} style={{
                    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                    padding: '12px 14px',
                    background: 'linear-gradient(180deg, #1F242B 0%, #1A1E24 100%)',
                    border: '1px solid #343A44', borderRadius: '8px',
                    gap: '12px',
                  }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', minWidth: 0 }}>
                      <Plug size={16} color={plugin.enabled ? '#8EA2FF' : '#5A6471'} style={{ flexShrink: 0 }} />
                      <div style={{ minWidth: 0 }}>
                        <div style={{ fontSize: '14px', fontWeight: '600', color: '#F3F5F8' }}>
                          {plugin.name}
                        </div>
                        <div style={{
                          fontSize: '11px', color: '#5A6471',
                          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
                          maxWidth: '280px',
                        }}>
                          {plugin.path}
                        </div>
                      </div>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexShrink: 0 }}>
                      <span style={{
                        fontSize: '11px', fontWeight: '600', padding: '2px 8px', borderRadius: '4px',
                        backgroundColor: plugin.enabled ? 'rgba(71,194,103,0.12)' : 'rgba(90,100,113,0.2)',
                        border: `1px solid ${plugin.enabled ? 'rgba(71,194,103,0.3)' : 'rgba(90,100,113,0.3)'}`,
                        color: plugin.enabled ? '#47C267' : '#5A6471',
                      }}>
                        {plugin.enabled ? 'Active' : 'Disabled'}
                      </span>
                      <button
                        onClick={() => handleUnloadPlugin(plugin.name)}
                        style={{
                          background: 'none', border: '1px solid rgba(255,122,114,0.3)',
                          borderRadius: '6px', color: '#FF7A72', fontSize: '12px',
                          padding: '4px 10px', cursor: 'pointer',
                        }}
                        title="Unload plugin"
                      >
                        Unload
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Refresh button */}
            <div style={{ marginTop: '16px', borderTop: '1px solid #343A44', paddingTop: '16px' }}>
              <button
                onClick={loadPlugins}
                style={{
                  display: 'flex', alignItems: 'center', gap: '6px',
                  padding: '7px 14px', backgroundColor: '#3A414C',
                  border: '1px solid #4A5260', borderRadius: '7px',
                  color: '#A7B0BC', fontSize: '13px', cursor: 'pointer',
                }}
              >
                <RefreshCw size={13} />
                Refresh
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ── Sync Modal ──────────────────────────────────────────────────────── */}
      {showSync && (
        <div style={{
          position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.7)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          padding: '1rem', zIndex: 1000,
        }}>
          <div style={{
            background: 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
            border: '1px solid #343A44', borderRadius: '14px', padding: '24px',
            maxWidth: '480px', width: '100%',
            boxShadow: '0 8px 16px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04)',
          }}>
            {/* Header */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                {syncConfig.enabled
                  ? <Wifi size={20} color="#47C267" />
                  : <WifiOff size={20} color="#5A6471" />
                }
                <h3 style={{ fontSize: '20px', fontWeight: '600', color: '#F3F5F8', margin: 0 }}>
                  Sync
                </h3>
              </div>
              <button onClick={() => setShowSync(false)} style={{
                background: 'none', border: 'none', color: '#A7B0BC', cursor: 'pointer', fontSize: '24px', padding: '4px',
              }}>✕</button>
            </div>

            <p style={{ fontSize: '13px', color: '#7D8793', margin: '0 0 20px 0', lineHeight: 1.5 }}>
              Sync your clipboard history across devices via a self-hosted relay server
              (or any server running the OpenPaste HTTP API).
            </p>

            {/* Status banners */}
            {syncError && (
              <div style={{
                padding: '10px 14px', backgroundColor: 'rgba(255,122,114,0.1)',
                border: '1px solid rgba(255,122,114,0.3)', borderRadius: '8px',
                color: '#FF7A72', fontSize: '13px', marginBottom: '14px',
              }}>
                {syncError}
              </div>
            )}
            {syncStatus && (
              <div style={{
                padding: '10px 14px', backgroundColor: 'rgba(71,194,103,0.1)',
                border: '1px solid rgba(71,194,103,0.3)', borderRadius: '8px',
                color: '#47C267', fontSize: '13px', marginBottom: '14px',
                display: 'flex', alignItems: 'center', gap: '8px',
              }}>
                <Check size={14} />
                {syncStatus}
              </div>
            )}

            {/* Form */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>

              {/* Enable toggle */}
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '14px', color: '#A6AEB8' }}>Enable sync</span>
                <div
                  onClick={() => setSyncConfig(c => ({ ...c, enabled: !c.enabled }))}
                  style={{
                    width: '44px', height: '24px',
                    backgroundColor: syncConfig.enabled ? '#6D7FFF' : '#3A414C',
                    borderRadius: '12px', position: 'relative', cursor: 'pointer',
                    transition: 'background-color 0.2s',
                  }}
                >
                  <div style={{
                    width: '20px', height: '20px', backgroundColor: '#F3F5F8',
                    borderRadius: '50%', position: 'absolute', top: '2px',
                    left: syncConfig.enabled ? '22px' : '2px',
                    transition: 'left 0.2s', pointerEvents: 'none',
                  }} />
                </div>
              </div>

              {/* Server URL */}
              <div>
                <label style={{ fontSize: '13px', color: '#7D8793', display: 'block', marginBottom: '6px' }}>
                  Server URL
                </label>
                <input
                  type="text"
                  placeholder="https://sync.example.com"
                  value={syncConfig.server_url}
                  onChange={e => setSyncConfig(c => ({ ...c, server_url: e.target.value }))}
                  style={{
                    width: '100%', padding: '9px 12px', boxSizing: 'border-box',
                    backgroundColor: '#1F242B', border: '1px solid #3A414C',
                    borderRadius: '7px', color: '#F3F5F8', fontSize: '14px', outline: 'none',
                  }}
                />
              </div>

              {/* API token */}
              <div>
                <label style={{ fontSize: '13px', color: '#7D8793', display: 'block', marginBottom: '6px' }}>
                  API Token <span style={{ color: '#5A6471' }}>(optional)</span>
                </label>
                <input
                  type="password"
                  placeholder="Bearer token for authenticated servers"
                  value={syncConfig.api_token ?? ''}
                  onChange={e => setSyncConfig(c => ({ ...c, api_token: e.target.value || null }))}
                  style={{
                    width: '100%', padding: '9px 12px', boxSizing: 'border-box',
                    backgroundColor: '#1F242B', border: '1px solid #3A414C',
                    borderRadius: '7px', color: '#F3F5F8', fontSize: '14px', outline: 'none',
                  }}
                />
              </div>

              {/* Last sync */}
              {syncConfig.last_sync_at && (
                <div style={{ fontSize: '12px', color: '#5A6471', display: 'flex', alignItems: 'center', gap: '5px' }}>
                  <Check size={12} color="#47C267" />
                  Last synced: {new Date(syncConfig.last_sync_at).toLocaleString()}
                </div>
              )}

              {/* Buttons */}
              <div style={{ display: 'flex', gap: '10px', paddingTop: '8px', borderTop: '1px solid #343A44' }}>
                <button
                  onClick={handleSaveSyncConfig}
                  disabled={syncLoading}
                  style={{
                    flex: 1, padding: '10px 16px', backgroundColor: '#6D7FFF',
                    border: 'none', borderRadius: '8px', color: '#F3F5F8',
                    fontSize: '14px', fontWeight: '500', cursor: syncLoading ? 'not-allowed' : 'pointer',
                    opacity: syncLoading ? 0.6 : 1,
                  }}
                >
                  Save Settings
                </button>
                <button
                  onClick={handleSyncNow}
                  disabled={syncLoading || !syncConfig.server_url.trim()}
                  style={{
                    display: 'flex', alignItems: 'center', gap: '7px',
                    padding: '10px 16px', backgroundColor: '#3A414C',
                    border: '1px solid #4A5260', borderRadius: '8px', color: '#F3F5F8',
                    fontSize: '14px', fontWeight: '500',
                    cursor: (syncLoading || !syncConfig.server_url.trim()) ? 'not-allowed' : 'pointer',
                    opacity: (syncLoading || !syncConfig.server_url.trim()) ? 0.5 : 1,
                  }}
                >
                  {syncLoading
                    ? <RefreshCw size={14} style={{ animation: 'spin 1s linear infinite' }} />
                    : <Download size={14} />
                  }
                  Sync Now
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default App
