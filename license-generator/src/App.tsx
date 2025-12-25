import { useState, useEffect, useCallback } from 'react'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import {
  supabase,
  isSupabaseConfigured,
  isServiceKeyConfigured,
} from './config/supabase'
import './App.css'

interface LicenseOutput {
  license_key: string
  tier: number
  tier_name: string
  created_at: string
}

interface CreateLicenseResult {
  success: boolean
  license_key?: string
  license_id?: string
  tier?: number
  error?: string
}

interface CreateBatchResult {
  success: boolean
  count?: number
  licenses?: CreateLicenseResult[]
  error?: string
}

// 许可证列表项接口
interface LicenseListItem {
  id: string
  license_key: string
  tier: number
  tier_name: string
  status: 'unused' | 'active' | 'expired' | 'revoked'
  machine_id: string | null
  os_info: string | null
  os_version: string | null
  hostname: string | null
  created_at: string
  activated_at: string | null
  expires_at: string | null
  last_verified_at: string | null
  activation_count: number
  max_activations: number
  notes: string | null
  days_remaining: number | null
}

interface LicenseListResult {
  success: boolean
  total: number
  page: number
  page_size: number
  total_pages: number
  licenses: LicenseListItem[]
}

interface LicenseStats {
  total: number
  unused: number
  active: number
  expired: number
  revoked: number
  by_tier?: Record<string, { total: number; unused: number; active: number; expired: number }>
}

type TabType = 'generate' | 'list'
type StatusFilter = 'all' | 'unused' | 'active' | 'expired' | 'revoked'

function App() {
  // Tab state
  const [activeTab, setActiveTab] = useState<TabType>('generate')

  // Generate tab state
  const [tier, setTier] = useState(1) // 1: Monthly, 2: Quarterly, 3: Yearly
  const [count, setCount] = useState(1)
  const [notes, setNotes] = useState('')
  const [maxActivations, setMaxActivations] = useState(1)
  const [licenses, setLicenses] = useState<LicenseOutput[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [copied, setCopied] = useState<number | null>(null)
  const [saving, setSaving] = useState(false)
  const [configStatus, setConfigStatus] = useState({ url: false, key: false })

  // License list tab state
  const [licenseList, setLicenseList] = useState<LicenseListItem[]>([])
  const [listLoading, setListLoading] = useState(false)
  const [listError, setListError] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [tierFilter, setTierFilter] = useState<number | null>(null)
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(1)
  const [totalCount, setTotalCount] = useState(0)
  const [stats, setStats] = useState<LicenseStats | null>(null)
  const [expandedLicense, setExpandedLicense] = useState<string | null>(null)

  // Check configuration status
  useEffect(() => {
    setConfigStatus({
      url: isSupabaseConfigured(),
      key: isServiceKeyConfigured(),
    })
  }, [])

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
    if (!configStatus.url || !configStatus.key) {
      setError('请先配置 Supabase URL 和 Service Role Key')
      return
    }

    setLoading(true)
    setError('')

    try {
      if (count === 1) {
        // 单个许可证 - 使用 Supabase 官方客户端
        const { data, error: rpcError } = await supabase.rpc('create_license', {
          p_tier: tier,
          p_notes: notes || null,
          p_max_activations: maxActivations,
        })

        if (rpcError) {
          setError(rpcError.message || '创建许可证失败')
          return
        }

        const result = data as CreateLicenseResult
        if (result.success && result.license_key) {
          setLicenses([{
            license_key: result.license_key,
            tier: result.tier || tier,
            tier_name: getTierName(result.tier || tier),
            created_at: new Date().toISOString(),
          }])
        } else {
          setError(result.error || '创建许可证失败')
        }
      } else {
        // 批量创建 - 使用 Supabase 官方客户端
        const { data, error: rpcError } = await supabase.rpc('create_licenses_batch', {
          p_tier: tier,
          p_count: count,
          p_notes: notes || null,
          p_max_activations: maxActivations,
        })

        if (rpcError) {
          setError(rpcError.message || '批量创建许可证失败')
          return
        }

        const result = data as CreateBatchResult
        if (result.success && result.licenses) {
          const newLicenses: LicenseOutput[] = result.licenses
            .filter(l => l.success && l.license_key)
            .map(l => ({
              license_key: l.license_key!,
              tier: l.tier || tier,
              tier_name: getTierName(l.tier || tier),
              created_at: new Date().toISOString(),
            }))
          setLicenses(newLicenses)
        } else {
          setError(result.error || '批量创建许可证失败')
        }
      }
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

  const copyAllKeys = async () => {
    try {
      const keys = licenses.map(l => l.license_key).join('\n')
      await navigator.clipboard.writeText(keys)
      setCopied(-3)
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

  const getTierName = (t: number) => {
    switch (t) {
      case 1: return '月度版'
      case 2: return '季度版'
      case 3: return '年度版'
      default: return '未知'
    }
  }

  const getTierLabel = (t: number) => {
    switch (t) {
      case 1: return '月度版'
      case 2: return '季度版'
      case 3: return '年度版'
      default: return '未知'
    }
  }

  const getTierDays = (t: number) => {
    switch (t) {
      case 1: return '30天'
      case 2: return '90天'
      case 3: return '365天'
      default: return ''
    }
  }

  const getTierColor = (t: number) => {
    switch (t) {
      case 1: return '#3b82f6'
      case 2: return '#8b5cf6'
      case 3: return '#f59e0b'
      default: return '#6b7280'
    }
  }

  // 获取状态颜色
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'unused': return '#6b7280'
      case 'active': return '#22c55e'
      case 'expired': return '#ef4444'
      case 'revoked': return '#f59e0b'
      default: return '#6b7280'
    }
  }

  // 获取状态显示名称
  const getStatusName = (status: string) => {
    switch (status) {
      case 'unused': return '未使用'
      case 'active': return '已激活'
      case 'expired': return '已过期'
      case 'revoked': return '已撤销'
      default: return status
    }
  }

  // 格式化日期
  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '-'
    return new Date(dateStr).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  // 获取许可证列表
  const fetchLicenseList = useCallback(async () => {
    if (!configStatus.url || !configStatus.key) return

    setListLoading(true)
    setListError('')

    try {
      const { data, error: rpcError } = await supabase.rpc('get_licenses_list', {
        p_status: statusFilter === 'all' ? null : statusFilter,
        p_tier: tierFilter,
        p_page: currentPage,
        p_page_size: 20,
      })

      if (rpcError) {
        setListError(rpcError.message || '获取许可证列表失败')
        return
      }

      const result = data as LicenseListResult
      if (result.success) {
        setLicenseList(result.licenses || [])
        setTotalPages(result.total_pages)
        setTotalCount(result.total)
      } else {
        setListError('获取许可证列表失败')
      }
    } catch (err) {
      setListError(String(err))
    } finally {
      setListLoading(false)
    }
  }, [configStatus.url, configStatus.key, statusFilter, tierFilter, currentPage])

  // 获取统计信息
  const fetchStatistics = useCallback(async () => {
    if (!configStatus.url || !configStatus.key) return

    try {
      const { data, error: rpcError } = await supabase.rpc('get_license_statistics')

      if (rpcError) {
        console.error('Failed to fetch statistics:', rpcError)
        return
      }

      const result = data as { success: boolean; statistics: LicenseStats }
      if (result.success) {
        setStats(result.statistics)
      }
    } catch (err) {
      console.error('Failed to fetch statistics:', err)
    }
  }, [configStatus.url, configStatus.key])

  // 当切换到列表标签时加载数据
  useEffect(() => {
    if (activeTab === 'list') {
      fetchLicenseList()
      fetchStatistics()
    }
  }, [activeTab, fetchLicenseList, fetchStatistics])

  // 当筛选条件变化时重新加载
  useEffect(() => {
    if (activeTab === 'list') {
      setCurrentPage(1)
    }
  }, [statusFilter, tierFilter, activeTab])

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
        <p className="subtitle">License Key Generator for Traceless (Supabase)</p>
      </header>

      {/* Configuration Status */}
      <div className="config-status">
        <div className={`status-item ${configStatus.url ? 'connected' : 'disconnected'}`}>
          <span className="status-dot"></span>
          <span>Supabase: {configStatus.url ? '已配置' : '未配置'}</span>
        </div>
        <div className={`status-item ${configStatus.key ? 'connected' : 'disconnected'}`}>
          <span className="status-dot"></span>
          <span>Service Key: {configStatus.key ? '已配置' : '未配置'}</span>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === 'generate' ? 'active' : ''}`}
          onClick={() => setActiveTab('generate')}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
          </svg>
          生成许可证
        </button>
        <button
          className={`tab-btn ${activeTab === 'list' ? 'active' : ''}`}
          onClick={() => setActiveTab('list')}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
          </svg>
          许可证列表
        </button>
      </div>

      {/* Generate Tab */}
      {activeTab === 'generate' && (
        <>
          <div className="form-section">
        <div className="form-group">
          <label>许可证类型</label>
          <div className="tier-buttons">
            {[1, 2, 3].map((t) => (
              <button
                key={t}
                className={`tier-btn ${tier === t ? 'active' : ''}`}
                onClick={() => setTier(t)}
                style={{ '--tier-color': getTierColor(t) } as React.CSSProperties}
              >
                <span className="tier-name">{getTierLabel(t)}</span>
                <span className="tier-days">{getTierDays(t)}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="form-row">
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
          <div className="form-group">
            <label>最大激活次数</label>
            <input
              type="number"
              min="1"
              max="10"
              value={maxActivations}
              onChange={(e) => setMaxActivations(Math.min(10, Math.max(1, parseInt(e.target.value) || 1)))}
              className="input"
            />
          </div>
        </div>

        <div className="form-group">
          <label>备注 (可选)</label>
          <input
            type="text"
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder="许可证备注信息"
            className="input"
          />
        </div>

        {error && <div className="error-message">{error}</div>}

        <button
          className="generate-btn"
          onClick={handleGenerate}
          disabled={loading || !configStatus.url || !configStatus.key}
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
              <button className="action-btn" onClick={copyAllKeys}>
                {copied === -3 ? (
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
                    复制密钥
                  </>
                )}
              </button>
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
                    复制JSON
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
                    保存文件
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
                    {license.tier_name}
                  </span>
                  <span className="info-item">
                    <strong>有效期:</strong> {getTierDays(license.tier)}
                  </span>
                  <span className="info-item">
                    <strong>状态:</strong> 未激活
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
        </>
      )}

      {/* License List Tab */}
      {activeTab === 'list' && (
        <div className="list-section">
          {/* Statistics */}
          {stats && (
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-value">{stats.total}</div>
                <div className="stat-label">总计</div>
              </div>
              <div className="stat-card" style={{ borderColor: '#6b7280' }}>
                <div className="stat-value" style={{ color: '#6b7280' }}>{stats.unused}</div>
                <div className="stat-label">未使用</div>
              </div>
              <div className="stat-card" style={{ borderColor: '#22c55e' }}>
                <div className="stat-value" style={{ color: '#22c55e' }}>{stats.active}</div>
                <div className="stat-label">已激活</div>
              </div>
              <div className="stat-card" style={{ borderColor: '#ef4444' }}>
                <div className="stat-value" style={{ color: '#ef4444' }}>{stats.expired}</div>
                <div className="stat-label">已过期</div>
              </div>
              <div className="stat-card" style={{ borderColor: '#f59e0b' }}>
                <div className="stat-value" style={{ color: '#f59e0b' }}>{stats.revoked}</div>
                <div className="stat-label">已撤销</div>
              </div>
            </div>
          )}

          {/* Filters */}
          <div className="filters">
            <div className="filter-group">
              <label>状态筛选</label>
              <div className="filter-buttons">
                {(['all', 'unused', 'active', 'expired', 'revoked'] as StatusFilter[]).map((s) => (
                  <button
                    key={s}
                    className={`filter-btn ${statusFilter === s ? 'active' : ''}`}
                    onClick={() => setStatusFilter(s)}
                    style={s !== 'all' ? { '--filter-color': getStatusColor(s) } as React.CSSProperties : undefined}
                  >
                    {s === 'all' ? '全部' : getStatusName(s)}
                  </button>
                ))}
              </div>
            </div>
            <div className="filter-group">
              <label>类型筛选</label>
              <div className="filter-buttons">
                <button
                  className={`filter-btn ${tierFilter === null ? 'active' : ''}`}
                  onClick={() => setTierFilter(null)}
                >
                  全部
                </button>
                {[1, 2, 3].map((t) => (
                  <button
                    key={t}
                    className={`filter-btn ${tierFilter === t ? 'active' : ''}`}
                    onClick={() => setTierFilter(t)}
                    style={{ '--filter-color': getTierColor(t) } as React.CSSProperties}
                  >
                    {getTierLabel(t)}
                  </button>
                ))}
              </div>
            </div>
            <button className="refresh-btn" onClick={fetchLicenseList} disabled={listLoading}>
              {listLoading ? (
                <span className="loading-spinner small"></span>
              ) : (
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M23 4v6h-6M1 20v-6h6" />
                  <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
                </svg>
              )}
              刷新
            </button>
          </div>

          {listError && <div className="error-message">{listError}</div>}

          {/* License List */}
          <div className="license-table">
            <div className="table-header">
              <span className="col-key">许可证密钥</span>
              <span className="col-tier">类型</span>
              <span className="col-status">状态</span>
              <span className="col-device">设备信息</span>
              <span className="col-date">激活时间</span>
              <span className="col-expires">到期时间</span>
            </div>

            {listLoading && licenseList.length === 0 ? (
              <div className="loading-placeholder">
                <span className="loading-spinner"></span>
                <span>加载中...</span>
              </div>
            ) : licenseList.length === 0 ? (
              <div className="empty-placeholder">
                <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" strokeWidth="1">
                  <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                </svg>
                <span>暂无许可证数据</span>
              </div>
            ) : (
              licenseList.map((lic) => (
                <div key={lic.id} className="table-row-wrapper">
                  <div
                    className={`table-row ${expandedLicense === lic.id ? 'expanded' : ''}`}
                    onClick={() => setExpandedLicense(expandedLicense === lic.id ? null : lic.id)}
                  >
                    <span className="col-key">
                      <code>{lic.license_key}</code>
                    </span>
                    <span className="col-tier">
                      <span className="tier-badge small" style={{ background: getTierColor(lic.tier) }}>
                        {lic.tier_name}
                      </span>
                    </span>
                    <span className="col-status">
                      <span className="status-badge" style={{ background: getStatusColor(lic.status) }}>
                        {getStatusName(lic.status)}
                      </span>
                    </span>
                    <span className="col-device">
                      {lic.hostname || lic.os_info || '-'}
                    </span>
                    <span className="col-date">
                      {formatDate(lic.activated_at)}
                    </span>
                    <span className="col-expires">
                      {lic.expires_at ? (
                        <>
                          {formatDate(lic.expires_at)}
                          {lic.days_remaining !== null && lic.days_remaining > 0 && (
                            <span className="days-remaining">({lic.days_remaining}天)</span>
                          )}
                        </>
                      ) : '-'}
                    </span>
                  </div>

                  {/* Expanded Details */}
                  {expandedLicense === lic.id && (
                    <div className="row-details">
                      <div className="detail-grid">
                        <div className="detail-item">
                          <label>许可证 ID</label>
                          <span>{lic.id}</span>
                        </div>
                        <div className="detail-item">
                          <label>机器 ID</label>
                          <span>{lic.machine_id || '-'}</span>
                        </div>
                        <div className="detail-item">
                          <label>操作系统</label>
                          <span>{lic.os_info || '-'} {lic.os_version || ''}</span>
                        </div>
                        <div className="detail-item">
                          <label>主机名</label>
                          <span>{lic.hostname || '-'}</span>
                        </div>
                        <div className="detail-item">
                          <label>创建时间</label>
                          <span>{formatDate(lic.created_at)}</span>
                        </div>
                        <div className="detail-item">
                          <label>最后验证</label>
                          <span>{formatDate(lic.last_verified_at)}</span>
                        </div>
                        <div className="detail-item">
                          <label>激活次数</label>
                          <span>{lic.activation_count} / {lic.max_activations}</span>
                        </div>
                        <div className="detail-item">
                          <label>备注</label>
                          <span>{lic.notes || '-'}</span>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="pagination">
              <button
                className="page-btn"
                disabled={currentPage <= 1}
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
              >
                上一页
              </button>
              <span className="page-info">
                第 {currentPage} 页 / 共 {totalPages} 页 (共 {totalCount} 条)
              </span>
              <button
                className="page-btn"
                disabled={currentPage >= totalPages}
                onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
              >
                下一页
              </button>
            </div>
          )}
        </div>
      )}

      <footer className="footer">
        <p>在线许可证生成 (Supabase) | 仅供授权使用</p>
      </footer>
    </div>
  )
}

export default App
