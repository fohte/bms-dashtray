import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { SetupScreen, type SetupScreenProps } from '@/components/SetupScreen'

const defaultProps: SetupScreenProps = {
  selectedPath: null,
  dbFileStatuses: [],
  isValidating: false,
  error: null,
  players: [],
  selectedPlayer: null,
  onSelectFolder: vi.fn(),
  onSelectPlayer: vi.fn(),
  onStart: vi.fn(),
}

function renderSetupScreen(overrides: Partial<SetupScreenProps> = {}) {
  return render(<SetupScreen {...defaultProps} {...overrides} />)
}

describe('SetupScreen', () => {
  it('renders the title and description', () => {
    renderSetupScreen()
    expect(screen.getByText('bms-dashtray')).toBeInTheDocument()
    expect(screen.getByText(/beatoraja/)).toBeInTheDocument()
  })

  it('shows placeholder when no path is selected', () => {
    renderSetupScreen()
    expect(screen.getByText('Select folder...')).toBeInTheDocument()
  })

  it('shows selected path', () => {
    renderSetupScreen({ selectedPath: '/Users/player/beatoraja' })
    expect(screen.getByText('/Users/player/beatoraja')).toBeInTheDocument()
  })

  it('shows summary when all files are found', () => {
    renderSetupScreen({
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: true },
        { name: 'score.db', found: true },
        { name: 'scorelog.db', found: true },
      ],
    })
    expect(
      screen.getByText(
        'songdata.db, scoredatalog.db, score.db, scorelog.db found',
      ),
    ).toBeInTheDocument()
  })

  it('shows not-found files individually when some are missing', () => {
    renderSetupScreen({
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: false },
        { name: 'score.db', found: false },
        { name: 'scorelog.db', found: true },
      ],
    })
    expect(
      screen.getByText('scoredatalog.db が見つかりません'),
    ).toBeInTheDocument()
    expect(screen.getByText('score.db が見つかりません')).toBeInTheDocument()
    expect(screen.getByText('songdata.db found')).toBeInTheDocument()
    expect(screen.getByText('scorelog.db found')).toBeInTheDocument()
  })

  it('disables START button when no validation has been done', () => {
    renderSetupScreen()
    expect(screen.getByRole('button', { name: 'START' })).toBeDisabled()
  })

  it('enables START button when all files are found', () => {
    renderSetupScreen({
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: true },
        { name: 'score.db', found: true },
        { name: 'scorelog.db', found: true },
      ],
    })
    expect(screen.getByRole('button', { name: 'START' })).toBeEnabled()
  })

  it('disables START button when some files are missing', () => {
    renderSetupScreen({
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: false },
        { name: 'score.db', found: false },
        { name: 'scorelog.db', found: false },
      ],
    })
    expect(screen.getByRole('button', { name: 'START' })).toBeDisabled()
  })

  it('shows VALIDATING... text while validating', () => {
    renderSetupScreen({ isValidating: true })
    expect(screen.getByRole('button', { name: 'VALIDATING...' })).toBeDisabled()
  })

  it('calls onSelectFolder when BROWSE is clicked', () => {
    const onSelectFolder = vi.fn()
    renderSetupScreen({ onSelectFolder })
    screen.getByRole('button', { name: '...' }).click()
    expect(onSelectFolder).toHaveBeenCalledOnce()
  })

  it('disables START button when error is present even if all files are found', () => {
    renderSetupScreen({
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: true },
        { name: 'score.db', found: true },
        { name: 'scorelog.db', found: true },
      ],
      error: 'Permission denied',
    })
    expect(screen.getByRole('button', { name: 'START' })).toBeDisabled()
  })

  it('calls onStart when START is clicked and enabled', () => {
    const onStart = vi.fn()
    renderSetupScreen({
      onStart,
      dbFileStatuses: [
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: true },
        { name: 'score.db', found: true },
        { name: 'scorelog.db', found: true },
      ],
    })
    screen.getByRole('button', { name: 'START' }).click()
    expect(onStart).toHaveBeenCalledOnce()
  })
})
