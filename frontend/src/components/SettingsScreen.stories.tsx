import type { Meta, StoryObj } from '@storybook/react-vite'

import { SettingsScreen } from '@/components/SettingsScreen'

const meta = {
  title: 'Components/SettingsScreen',
  component: SettingsScreen,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof SettingsScreen>

export default meta
type Story = StoryObj<typeof meta>

const defaultConfig = {
  beatorajaRoot: '/Users/player/beatoraja',
  playerName: 'default',
  resetTime: '05:00',
  backgroundTransparent: false,
  fontSize: 13,
}

const noop = () => {}

const baseArgs = {
  onBack: noop,
  onChangeBeatorajaRoot: noop,
  onToggleBackgroundTransparent: noop,
  onChangeFontSize: noop,
  onChangeResetTime: noop,
  onResetHistory: noop,
}

export const Default: Story = {
  args: {
    ...baseArgs,
    config: defaultConfig,
  },
}

export const TransparentEnabled: Story = {
  args: {
    ...baseArgs,
    config: { ...defaultConfig, backgroundTransparent: true },
  },
}

export const LargeFontSize: Story = {
  args: {
    ...baseArgs,
    config: { ...defaultConfig, fontSize: 20 },
  },
}

export const CustomResetTime: Story = {
  args: {
    ...baseArgs,
    config: { ...defaultConfig, resetTime: '04:00' },
  },
}
