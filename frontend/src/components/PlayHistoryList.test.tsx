import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { PlayHistoryList } from '@/components/PlayHistoryList'
import { makeRecord } from '@/test-helpers'

function expectFlashAnimation(el: HTMLElement, expectedMs: string) {
  expect(el.style.animation).toContain('lampFlash')
  expect(el.style.animation).toContain(expectedMs)
}

function expectNoFlashAnimation(el: HTMLElement) {
  expect(el.style.animation).toBe('')
}

describe('PlayHistoryList lamp bar', () => {
  describe('flash animation on split lamp bar', () => {
    it.each([
      {
        name: 'previous flashes when previousClear has alt color, current does not',
        clear: 6,
        previousClear: 1,
        isRetired: false,
        expectedPreviousMs: '50ms',
        expectedCurrentMs: null,
      },
      {
        name: 'both halves flash when both have alt colors',
        clear: 7,
        previousClear: 1,
        isRetired: false,
        expectedPreviousMs: '50ms',
        expectedCurrentMs: '100ms',
      },
      {
        name: 'isRetired suppresses all flash animations',
        clear: 7,
        previousClear: 1,
        isRetired: true,
        expectedPreviousMs: null,
        expectedCurrentMs: null,
      },
    ])(
      '$name',
      ({
        clear,
        previousClear,
        isRetired,
        expectedPreviousMs,
        expectedCurrentMs,
      }) => {
        const record = makeRecord({ clear, previousClear, isRetired })
        render(<PlayHistoryList records={[record]} />)

        const previousLamp = screen.getByTestId('lamp-bar-previous')
        const currentLamp = screen.getByTestId('lamp-bar-current')
        if (expectedPreviousMs != null) {
          expectFlashAnimation(previousLamp, expectedPreviousMs)
        } else {
          expectNoFlashAnimation(previousLamp)
        }
        if (expectedCurrentMs != null) {
          expectFlashAnimation(currentLamp, expectedCurrentMs)
        } else {
          expectNoFlashAnimation(currentLamp)
        }
      },
    )
  })

  describe('flash animation on single lamp bar', () => {
    it.each([
      {
        name: 'applies flash when clear has an alt color',
        clear: 1,
        isRetired: false,
        expectFlash: true,
        expectedMs: '50ms',
      },
      {
        name: 'no flash when clear has no alt color',
        clear: 6,
        isRetired: false,
        expectFlash: false,
      },
      {
        name: 'no flash when isRetired is true',
        clear: 1,
        isRetired: true,
        expectFlash: false,
      },
    ])('$name', ({ clear, isRetired, expectFlash, expectedMs }) => {
      const record = makeRecord({
        clear,
        previousClear: null,
        isRetired,
      })
      render(<PlayHistoryList records={[record]} />)

      const lampBar = screen.getByTestId('lamp-bar')
      if (expectFlash && expectedMs != null) {
        expectFlashAnimation(lampBar, expectedMs)
      } else {
        expectNoFlashAnimation(lampBar)
      }
    })
  })
})
