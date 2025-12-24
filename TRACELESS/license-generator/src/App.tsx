import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import './App.css'

interface LicenseOutput {
  license_key: string
  tier: string
  machine_id: string
  activation_date: string
  expiration_date: string
  days_valid: number
}

function App() {
  const [tier, setTier] = useState('monthly')
  const [machineId, setMachineId] = useState('')
  const [activationDate, setActivationDate] = useState('')
  const [count, setCount] = useState(1)
  const [licenses, setLicenses] = useState<LicenseOutput[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [copied, setCopied] = useState<number | null>(null)
  const [saving, setSaving] = useState(false)

  // Disable right-click context menu
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault()
    }
    document.addEventListener('contextmenu', handleContextMenu)
    return () => {
      document.removeEventListener('contextmenu', handleContextMenu)
    }
  }, [])

  const handleGenerate = async () => {
    if (!machineId.trim()) {
      setError('请输入机器ID')
      return
    }
    if (machineId.length < 8) {
      setError('机器ID至少需要8个字符')
      return
    }

    setLoading(true)
    setError('')

    try {
      const result = await invoke<LicenseOutput[]>('generate_license', {
        tier,
        machineId: machineId.trim(),
        activationDate: activationDate || null,
        count,
      })
      setLicenses(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const copyToClipboard = async (text: string, index: number) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(index)
      setTimeout(() => setCopied(null), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }

  const copyAllAsJson = async () => {
    try {
      await navigator.clipboard.writeText(JSON.stringify(licenses, null, 2))
      setCopied(-1)
      setTimeout(() => setCopied(null), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }

  const saveToFile = async () => {
    if (licenses.length === 0) return

    setSaving(true)
    try {
      const filePath = await save({
        defaultPath: `licenses_${new Date().toISOString().split('T')[0]}.json`,
        filters: [
          { name: 'JSON', extensions: ['json'] },
          { name: 'Text', extensions: ['txt'] },
        ],
      })

      if (filePath) {
        const content = JSON.stringify(licenses, null, 2)
        await writeTextFile(filePath, content)
        setCopied(-2)
        setTimeout(() => setCopied(null), 2000)
      }
    } catch (err) {
      console.error('Failed to save:', err)
      setError('保存失败: ' + String(err))
    } finally {
      setSaving(false)
    }
  }

  const getTierLabel = (t: string) => {
    switch (t) {
      case 'monthly': return '月度版'
      case 'quarterly': return '季度版'
      case 'yearly': return '年度版'
      default: return t
    }
  }

  const getTierColor = (t: string) => {
    switch (t) {
      case 'monthly': return '#3b82f6'
      case 'quarterly': return '#8b5cf6'
      case 'yearly': return '#f59e0b'
      default: return '#6b7280'
    }
  }

  return (
    <div className="container">
      {/* Draggable Title Bar */}
      <div className="titlebar" data-tauri-drag-region>
        <div className="titlebar-title" data-tauri-drag-region>License Key Generator</div>
      </div>

      <header className="header">
        <div className="logo">
          <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            <path d="M9 12l2 2 4-4" />
          </svg>
        </div>
        <h1>许可证密钥生成器</h1>
        <p className="subtitle">License Key Generator for Traceless</p>
      </header>

      <div className="form-section">
        <div className="form-group">
          <label>许可证类型</label>
          <div className="tier-buttons">
            {['monthly', 'quarterly', 'yearly'].map((t) => (
              <button
                key={t}
                className={`tier-btn ${tier === t ? 'active' : ''}`}
                onClick={() => setTier(t)}
                style={{ '--tier-color': getTierColor(t) } as React.CSSProperties}
              >
                <span className="tier-name">{getTierLabel(t)}</span>
                <span className="tier-days">
                  {t === 'monthly' ? '30天' : t === 'quarterly' ? '90天' : '365天'}
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label>机器ID <span className="required">*</span></label>
          <input
            type="text"
            value={machineId}
            onChange={(e) => setMachineId(e.target.value.toUpperCase())}
            placeholder="输入目标设备的机器ID (至少8字符)"
            className="input"
          />
          <p className="hint">机器ID可在应用的许可证激活对话框中找到</p>
        </div>

        <div className="form-row">
          <div className="form-group">
            <label>激活日期 (可选)</label>
            <input
              type="date"
              value={activationDate}
              onChange={(e) => setActivationDate(e.target.value)}
              className="input"
            />
          </div>
          <div className="form-group">
            <label>生成数量</label>
            <input
              type="number"
              min="1"
              max="100"
              value={count}
              onChange={(e) => setCount(Math.min(100, Math.max(1, parseInt(e.target.value) || 1)))}
              className="input"
            />
          </div>
        </div>

        {error && <div className="error-message">{error}</div>}

        <button
          className="generate-btn"
          onClick={handleGenerate}
          disabled={loading}
        >
          {loading ? (
            <span className="loading-spinner"></span>
          ) : (
            <>
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
              </svg>
              生成许可证
            </>
          )}
        </button>
      </div>

      {licenses.length > 0 && (
        <div className="results-section">
          <div className="results-header">
            <h2>生成结果 ({licenses.length})</h2>
            <div className="results-actions">
              <button className="action-btn" onClick={copyAllAsJson}>
                {copied === -1 ? (
                  <>
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M20 6L9 17l-5-5" />
                    </svg>
                    已复制
                  </>
                ) : (
                  <>
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
                      <rect x="9" y="9" width="13" height="13" rx="2" />
                      <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
                    </svg>
                    复制全部
                  </>
                )}
              </button>
              <button className="action-btn save-btn" onClick={saveToFile} disabled={saving}>
                {copied === -2 ? (
                  <>
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M20 6L9 17l-5-5" />
                    </svg>
                    已保存
                  </>
                ) : saving ? (
                  <span className="loading-spinner small"></span>
                ) : (
                  <>
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                      <polyline points="7 10 12 15 17 10" />
                      <line x1="12" y1="15" x2="12" y2="3" />
                    </svg>
                    保存到文件
                  </>
                )}
              </button>
            </div>
          </div>

          <div className="licenses-list">
            {licenses.map((license, index) => (
              <div key={index} className="license-card">
                <div
                  className="license-key-row clickable"
                  onClick={() => copyToClipboard(license.license_key, index)}
                  title="点击复制"
                >
                  <code className="license-key">{license.license_key}</code>
                  <span className="copy-indicator">
                    {copied === index ? (
                      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#22c55e" strokeWidth="2">
                        <path d="M20 6L9 17l-5-5" />
                      </svg>
                    ) : (
                      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2">
                        <rect x="9" y="9" width="13" height="13" rx="2" />
                        <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
                      </svg>
                    )}
                  </span>
                </div>
                <div className="license-info">
                  <span className="tier-badge" style={{ background: getTierColor(license.tier) }}>
                    {getTierLabel(license.tier)}
                  </span>
                  <span className="info-item">
                    <strong>激活:</strong> {license.activation_date}
                  </span>
                  <span className="info-item">
                    <strong>到期:</strong> {license.expiration_date}
                  </span>
                  <span className="info-item">
                    <strong>有效期:</strong> {license.days_valid}天
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <footer className="footer">
        <p>仅供授权使用 | For authorized use only</p>
      </footer>
    </div>
  )
}

export default App
