import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api'
import { appWindow } from '@tauri-apps/api/window'
import { 
  Clipboard, 
  Search, 
  Pin, 
  Settings, 
  Menu, 
  Link, 
  FileText, 
  Image, 
  Terminal, 
  Copy, 
  Star, 
  Trash2, 
  Check,
  Minimize,
  Maximize2,
  X
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

function App() {
  const [items, setItems] = useState<ClipboardItem[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedItem, setSelectedItem] = useState<ClipboardItem | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadClipboardHistory(true)

    // Auto-refresh every 2 seconds
    const interval = setInterval(() => {
      loadClipboardHistory(false)
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  const loadClipboardHistory = async (isInitial: boolean = false) => {
    try {
      if (isInitial) {
        setLoading(true)
      }
      setError(null)

      const history = await invoke<ClipboardItem[]>('get_clipboard_history')

      // Only update if data actually changed (compare by ID and hash)
      const currentIds = items.map(item => `${item.id}-${item.hash}`).join(',')
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
      await invoke('toggle_pin_item', { id: Number(item.id) })
      loadClipboardHistory(false)
    } catch (e) {
      console.error('Failed to toggle pin:', e)
    }
  }

  const toggleFavorite = async (item: ClipboardItem) => {
    try {
      await invoke('toggle_favorite_item', { id: Number(item.id) })
      loadClipboardHistory(false)
    } catch (e) {
      console.error('Failed to toggle favorite:', e)
    }
  }

  const getIconForType = (contentType: string) => {
    const type = contentType.toLowerCase()
    if (type.includes('url') || type.includes('http')) return Link
    if (type.includes('image')) return Image
    if (type.includes('code') || type.includes('json')) return Terminal
    return FileText
  }

  const getTypeBadge = (contentType: string) => {
    const type = contentType.toLowerCase()
    if (type.includes('url') || type.includes('http')) return 'URL'
    if (type.includes('image')) return 'IMAGE'
    if (type.includes('code') || type.includes('json')) return 'CODE'
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

  const filteredItems = items.filter(item =>
    item.content.toLowerCase().includes(searchQuery.toLowerCase())
  )

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
          <Search size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} />
          <Pin size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} />
          <Settings size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} />
          <Menu size={20} color="#C8CDD4" style={{ cursor: 'pointer' }} />
        </div>
      </div>

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
            type="text"
            placeholder="Search clipboard..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
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
            {filteredItems.map((item) => {
              const IconComponent = getIconForType(item.content_type)
              return (
                <div
                  key={item.id}
                  style={{
                    height: '80px',
                    background: 'linear-gradient(180deg, #262B31 0%, #21252B 100%)',
                    border: '1px solid #343A44',
                    borderRadius: '10px',
                    padding: '14px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '14px',
                    boxShadow: '0 4px 8px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.04), inset 0 -1px 0 rgba(0,0,0,0.30)',
                    transition: 'transform 120ms ease-out, box-shadow 120ms ease-out',
                    cursor: 'pointer'
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.transform = 'translateY(-1px)'
                    e.currentTarget.style.borderColor = '#5A6471'
                    e.currentTarget.style.boxShadow = '0 6px 12px rgba(0,0,0,0.28)'
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.transform = 'translateY(0)'
                    e.currentTarget.style.borderColor = '#343A44'
                    e.currentTarget.style.boxShadow = '0 4px 8px rgba(0,0,0,0.22)'
                  }}
                  onClick={() => setSelectedItem(item)}
                >
                  {/* Left Icon Area */}
                  <div style={{
                    width: '48px',
                    height: '48px',
                    backgroundColor: '#232831',
                    border: '1px solid #3A404B',
                    borderRadius: '10px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0
                  }}>
                    <IconComponent size={20} color="#8EA2FF" />
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
                    <div style={{
                      display: 'inline-block',
                      height: '20px',
                      padding: '0 8px',
                      backgroundColor: '#29314A',
                      border: '1px solid #465273',
                      borderRadius: '4px',
                      fontSize: '10px',
                      fontWeight: '600',
                      color: '#8EA2FF',
                      textTransform: 'uppercase',
                      lineHeight: '20px'
                    }}>
                      {getTypeBadge(item.content_type)}
                    </div>
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
                        color="#6D7FFF" 
                        style={{ cursor: 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          copyToClipboard(item.content)
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
                        }}
                      />
                      <Trash2 
                        size={16} 
                        color="#FF7A72" 
                        style={{ cursor: 'pointer' }}
                        onClick={(e) => {
                          e.stopPropagation()
                          deleteItem(item.id)
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
      </div>

      {/* Bottom Status Bar */}
      <div style={{
        height: '48px',
        backgroundColor: '#1A1E24',
        borderTop: '1px solid #2D333D',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 32px',
        flexShrink: 0
      }}>
        <div style={{ display: 'flex', gap: '24px' }}>
          <span style={{ fontSize: '13px', fontWeight: '500', color: '#A7B0BC' }}>
            {filteredItems.length} items
          </span>
          <span style={{ fontSize: '13px', fontWeight: '500', color: '#A7B0BC' }}>
            {items.filter(i => i.pinned).length} pinned
          </span>
          <span style={{ fontSize: '13px', fontWeight: '500', color: '#A7B0BC' }}>
            {items.filter(i => i.favorite).length} favorites
          </span>
          <span style={{ fontSize: '13px', fontWeight: '500', color: '#A7B0BC' }}>
            {items.filter(i => i.content_type.includes('image')).length} images
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <Check size={14} color="#47C267" />
          <span style={{ fontSize: '13px', fontWeight: '500', color: '#47C267' }}>
            All synced
          </span>
        </div>
      </div>

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
                onClick={() => setSelectedItem(null)}
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
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default App
