import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FilterTabs, type FilterTabsProps } from '@/components/FilterTabs'

const defaultProps: FilterTabsProps = {
  activeFilter: 'all',
  onFilterChange: vi.fn(),
}

function renderFilterTabs(overrides: Partial<FilterTabsProps> = {}) {
  return render(<FilterTabs {...defaultProps} {...overrides} />)
}

describe('FilterTabs', () => {
  it('renders ALL and UPDATED tabs', () => {
    renderFilterTabs()
    expect(screen.getByRole('button', { name: 'ALL' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'UPDATED' })).toBeInTheDocument()
  })

  it('marks ALL tab as pressed when activeFilter is all', () => {
    renderFilterTabs({ activeFilter: 'all' })
    expect(screen.getByRole('button', { name: 'ALL' })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
    expect(screen.getByRole('button', { name: 'UPDATED' })).toHaveAttribute(
      'aria-pressed',
      'false',
    )
  })

  it('marks UPDATED tab as pressed when activeFilter is updated', () => {
    renderFilterTabs({ activeFilter: 'updated' })
    expect(screen.getByRole('button', { name: 'ALL' })).toHaveAttribute(
      'aria-pressed',
      'false',
    )
    expect(screen.getByRole('button', { name: 'UPDATED' })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
  })

  it('calls onFilterChange with updated when UPDATED tab is clicked', () => {
    const onFilterChange = vi.fn()
    renderFilterTabs({ activeFilter: 'all', onFilterChange })
    screen.getByRole('button', { name: 'UPDATED' }).click()
    expect(onFilterChange).toHaveBeenCalledWith('updated')
  })

  it('calls onFilterChange with all when ALL tab is clicked', () => {
    const onFilterChange = vi.fn()
    renderFilterTabs({ activeFilter: 'updated', onFilterChange })
    screen.getByRole('button', { name: 'ALL' }).click()
    expect(onFilterChange).toHaveBeenCalledWith('all')
  })
})
