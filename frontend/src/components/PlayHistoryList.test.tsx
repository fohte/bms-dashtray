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
    it('applies flash animation to the previous lamp when previousClear has an alt color', () => {
      // Failed(1) -> Hard(6): Failed has alt color, Hard does not
      const record = makeRecord({
        clear: 6,
        previousClear: 1,
        isRetired: false,
      })
      render(<PlayHistoryList records={[record]} />)

      const previousLamp = screen.getByTestId('lamp-bar-previous')
      expectFlashAnimation(previousLamp, '50ms')
    })

    it('does not apply flash animation to the current lamp when it has no alt color', () => {
      // Failed(1) -> Hard(6): Hard has no alt color
      const record = makeRecord({
        clear: 6,
        previousClear: 1,
        isRetired: false,
      })
      render(<PlayHistoryList records={[record]} />)

      const currentLamp = screen.getByTestId('lamp-bar-current')
      expectNoFlashAnimation(currentLamp)
    })

    it('applies flash animation to both halves when both have alt colors', () => {
      // Failed(1) -> ExHard(7): both have alt colors
      const record = makeRecord({
        clear: 7,
        previousClear: 1,
        isRetired: false,
      })
      render(<PlayHistoryList records={[record]} />)

      const previousLamp = screen.getByTestId('lamp-bar-previous')
      const currentLamp = screen.getByTestId('lamp-bar-current')
      expectFlashAnimation(previousLamp, '50ms')
      expectFlashAnimation(currentLamp, '100ms')
    })

    it('does not apply flash animation to either half when isRetired is true', () => {
      // ExHard(7) with previousClear Failed(1), retired — both have alt colors
      // normally, but isRetired should suppress all flash animations.
      const record = makeRecord({
        clear: 7,
        previousClear: 1,
        isRetired: true,
      })
      render(<PlayHistoryList records={[record]} />)

      const previousLamp = screen.getByTestId('lamp-bar-previous')
      const currentLamp = screen.getByTestId('lamp-bar-current')
      expectNoFlashAnimation(previousLamp)
      expectNoFlashAnimation(currentLamp)
    })
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
