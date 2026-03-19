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

export const Default: Story = {
  args: {
    config: defaultConfig,
    onBack: noop,
    onChangeBeatorajaRoot: noop,
    onToggleBackgroundTransparent: noop,
    onChangeFontSize: noop,
    onChangeResetTime: noop,
    onResetHistory: noop,
  },
}

export const TransparentEnabled: Story = {
  args: {
    config: { ...defaultConfig, backgroundTransparent: true },
    onBack: noop,
    onChangeBeatorajaRoot: noop,
    onToggleBackgroundTransparent: noop,
    onChangeFontSize: noop,
    onChangeResetTime: noop,
    onResetHistory: noop,
  },
}

export const LargeFontSize: Story = {
  args: {
    config: { ...defaultConfig, fontSize: 20 },
    onBack: noop,
    onChangeBeatorajaRoot: noop,
    onToggleBackgroundTransparent: noop,
    onChangeFontSize: noop,
    onChangeResetTime: noop,
    onResetHistory: noop,
  },
}

export const CustomResetTime: Story = {
  args: {
    config: { ...defaultConfig, resetTime: '04:00' },
    onBack: noop,
    onChangeBeatorajaRoot: noop,
    onToggleBackgroundTransparent: noop,
    onChangeFontSize: noop,
    onChangeResetTime: noop,
    onResetHistory: noop,
  },
}
