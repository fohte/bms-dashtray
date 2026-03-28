import type { ReactNode } from 'react'

export interface MainScreenProps {
  todayDate: string
  backgroundTransparent: boolean
  onOpenSettings: () => void
  onResetHistory: () => void
  children: ReactNode
}

export const MainScreen = ({
  todayDate,
  backgroundTransparent,
  onOpenSettings,
  onResetHistory,
  children,
}: MainScreenProps) => {
  const bgColor = backgroundTransparent ? 'transparent' : '#000000'

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: bgColor,
        color: '#ffffff',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px',
        }}
      >
        <span
          style={{
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: 'var(--font-size-lg)',
            fontWeight: 700,
            letterSpacing: '1.5px',
          }}
        >
          bms-dashtray
        </span>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
          }}
        >
          <span
            style={{
              fontFamily: "'JetBrains Mono', monospace",
              fontSize: 'var(--font-size-md)',
              fontWeight: 500,
              color: '#64748B',
            }}
          >
            {todayDate}
          </span>
          <button
            type="button"
            onClick={onOpenSettings}
            style={{
              background: 'none',
              border: 'none',
              color: '#888888',
              fontSize: 'var(--font-size-xl)',
              cursor: 'pointer',
              padding: '4px 8px',
            }}
            aria-label="Settings"
          >
            &#9881;
          </button>
        </div>
      </div>
      <div style={{ height: '1px', backgroundColor: '#1A1A1A' }} />
      {children}
      <div style={{ flex: 1 }} />
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          padding: '12px 16px',
        }}
      >
        <button
          type="button"
          onClick={onResetHistory}
          style={{
            background: 'none',
            border: '1px solid #475569',
            borderRadius: '4px',
            color: '#475569',
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: 'var(--font-size-md)',
            fontWeight: 600,
            letterSpacing: '1px',
            padding: '6px 12px',
            cursor: 'pointer',
          }}
        >
          RESET
        </button>
      </div>
    </div>
  )
}
