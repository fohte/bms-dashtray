import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { PlayHistoryList } from '@/components/PlayHistoryList'
import { makeRecord } from '@/test-helpers'

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
      expect(previousLamp.style.animation).toContain('lampFlash')
      expect(previousLamp.style.animation).toContain('50ms')
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
      expect(currentLamp.style.animation).toBe('')
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
      expect(previousLamp.style.animation).toContain('lampFlash')
      expect(previousLamp.style.animation).toContain('50ms')
      expect(currentLamp.style.animation).toContain('lampFlash')
      expect(currentLamp.style.animation).toContain('100ms')
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
      expect(previousLamp.style.animation).toBe('')
      expect(currentLamp.style.animation).toBe('')
    })
  })

  describe('flash animation on single lamp bar', () => {
    it('applies flash animation when clear has an alt color', () => {
      const record = makeRecord({
        clear: 1,
        previousClear: null,
        isRetired: false,
      })
      render(<PlayHistoryList records={[record]} />)

      const lampBar = screen.getByTestId('lamp-bar')
      expect(lampBar.style.animation).toContain('lampFlash')
      expect(lampBar.style.animation).toContain('50ms')
    })

    it('does not apply flash animation when clear has no alt color', () => {
      const record = makeRecord({
        clear: 6,
        previousClear: null,
        isRetired: false,
      })
      render(<PlayHistoryList records={[record]} />)

      const lampBar = screen.getByTestId('lamp-bar')
      expect(lampBar.style.animation).toBe('')
    })

    it('does not apply flash animation when isRetired is true', () => {
      const record = makeRecord({
        clear: 1,
        previousClear: null,
        isRetired: true,
      })
      render(<PlayHistoryList records={[record]} />)

      const lampBar = screen.getByTestId('lamp-bar')
      expect(lampBar.style.animation).toBe('')
    })
  })
})
