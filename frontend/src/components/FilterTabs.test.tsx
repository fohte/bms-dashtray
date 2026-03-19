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

  it.each([
    { activeFilter: 'all' as const, pressed: 'ALL', notPressed: 'UPDATED' },
    {
      activeFilter: 'updated' as const,
      pressed: 'UPDATED',
      notPressed: 'ALL',
    },
  ])(
    'marks $pressed tab as pressed when activeFilter is $activeFilter',
    ({ activeFilter, pressed, notPressed }) => {
      renderFilterTabs({ activeFilter })
      expect(screen.getByRole('button', { name: pressed })).toHaveAttribute(
        'aria-pressed',
        'true',
      )
      expect(screen.getByRole('button', { name: notPressed })).toHaveAttribute(
        'aria-pressed',
        'false',
      )
    },
  )

  it.each([
    {
      activeFilter: 'all' as const,
      clickTarget: 'UPDATED',
      expected: 'updated',
    },
    {
      activeFilter: 'updated' as const,
      clickTarget: 'ALL',
      expected: 'all',
    },
  ])(
    'calls onFilterChange with $expected when $clickTarget tab is clicked',
    ({ activeFilter, clickTarget, expected }) => {
      const onFilterChange = vi.fn()
      renderFilterTabs({ activeFilter, onFilterChange })
      screen.getByRole('button', { name: clickTarget }).click()
      expect(onFilterChange).toHaveBeenCalledWith(expected)
    },
  )
})
